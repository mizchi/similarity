#![allow(clippy::uninlined_format_args)]

use crate::moonbit_parser::MoonBitParser;
use rayon::prelude::*;
use similarity_core::{
    cli_parallel::{FileData, SimilarityResult},
    language_parser::{GenericFunctionDef, LanguageParser},
    tree::TreeNode,
    tsed::{calculate_tsed, TSEDOptions},
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

/// MoonBit file with its content and extracted functions
#[allow(dead_code)]
pub type MoonBitFileData = FileData<GenericFunctionDef>;

/// Lightweight fingerprint: histogram of node kinds in the tree
type NodeFingerprint = HashMap<u16, u16>;

/// Pre-parsed function data with tree, size, and fingerprint
struct ParsedFunc {
    tree: Rc<TreeNode>,
    size: usize,
    fingerprint: NodeFingerprint,
}

/// Build a fingerprint from a TreeNode by counting occurrences of each node kind.
/// Uses a u16 hash of the label to keep the fingerprint compact.
fn build_fingerprint(node: &TreeNode) -> NodeFingerprint {
    let mut fp = HashMap::new();
    build_fingerprint_recursive(node, &mut fp);
    fp
}

fn build_fingerprint_recursive(node: &TreeNode, fp: &mut NodeFingerprint) {
    let key = hash_label(&node.label);
    *fp.entry(key).or_insert(0) += 1;
    for child in &node.children {
        build_fingerprint_recursive(child, fp);
    }
}

fn hash_label(label: &str) -> u16 {
    let mut h: u32 = 0;
    for b in label.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    h as u16
}

/// Quick Jaccard-like similarity between two fingerprints.
/// Returns a value between 0.0 and 1.0.
fn fingerprint_similarity(a: &NodeFingerprint, b: &NodeFingerprint) -> f64 {
    let mut intersection: u32 = 0;
    let mut union: u32 = 0;

    for (key, &count_a) in a {
        let count_b = b.get(key).copied().unwrap_or(0);
        intersection += count_a.min(count_b) as u32;
        union += count_a.max(count_b) as u32;
    }
    for (key, &count_b) in b {
        if !a.contains_key(key) {
            union += count_b as u32;
        }
    }

    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Load and parse MoonBit files in parallel
#[allow(dead_code)]
pub fn load_files_parallel(files: &[PathBuf]) -> Vec<MoonBitFileData> {
    files
        .par_iter()
        .filter_map(|file| match fs::read_to_string(file) {
            Ok(content) => {
                let filename = file.to_string_lossy();
                match MoonBitParser::new() {
                    Ok(mut parser) => match parser.extract_functions(&content, &filename) {
                        Ok(functions) => Some(FileData { path: file.clone(), content, functions }),
                        Err(e) => {
                            eprintln!("Error parsing {}: {}", file.display(), e);
                            None
                        }
                    },
                    Err(e) => {
                        eprintln!("Error creating parser for {}: {}", file.display(), e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
                None
            }
        })
        .collect()
}

/// Check for duplicates within MoonBit files in parallel
pub fn check_within_file_duplicates_parallel(
    files: &[PathBuf],
    threshold: f64,
    options: &TSEDOptions,
) -> Vec<(PathBuf, Vec<SimilarityResult<GenericFunctionDef>>)> {
    files
        .par_iter()
        .filter_map(|file| match fs::read_to_string(file) {
            Ok(code) => {
                let file_str = file.to_string_lossy();

                match MoonBitParser::new() {
                    Ok(mut parser) => match parser.extract_functions(&code, &file_str) {
                        Ok(functions) => {
                            if functions.is_empty() {
                                return None;
                            }

                            // Pre-split lines once
                            let lines: Vec<&str> = code.lines().collect();

                            // Pre-parse all function bodies with size and fingerprint
                            let parsed: Vec<Option<ParsedFunc>> = functions
                                .iter()
                                .map(|func| {
                                    let body = extract_function_body(&lines, func);
                                    parser.parse(&body, "body.mbt").ok().map(|tree| {
                                        let size = tree.get_subtree_size();
                                        let fingerprint = build_fingerprint(&tree);
                                        ParsedFunc { tree, size, fingerprint }
                                    })
                                })
                                .collect();

                            let mut similar_pairs = Vec::new();

                            for i in 0..functions.len() {
                                for j in (i + 1)..functions.len() {
                                    let func1 = &functions[i];
                                    let func2 = &functions[j];

                                    if func1.end_line - func1.start_line + 1 < options.min_lines
                                        || func2.end_line - func2.start_line + 1 < options.min_lines
                                    {
                                        continue;
                                    }

                                    let similarity = match (parsed[i].as_ref(), parsed[j].as_ref())
                                    {
                                        (Some(p1), Some(p2)) => {
                                            // Skip if below min_tokens threshold
                                            if let Some(min_tokens) = options.min_tokens {
                                                if (p1.size as u32) < min_tokens
                                                    || (p2.size as u32) < min_tokens
                                                {
                                                    continue;
                                                }
                                            }

                                            // Pre-filter 1: tree size ratio
                                            let (min_s, max_s) = if p1.size <= p2.size {
                                                (p1.size, p2.size)
                                            } else {
                                                (p2.size, p1.size)
                                            };
                                            if max_s == 0
                                                || (min_s as f64 / max_s as f64) < threshold
                                            {
                                                0.0
                                            } else if fingerprint_similarity(
                                                &p1.fingerprint,
                                                &p2.fingerprint,
                                            ) < threshold
                                            {
                                                // Pre-filter 2: node kind histogram
                                                0.0
                                            } else {
                                                calculate_tsed(&p1.tree, &p2.tree, options)
                                            }
                                        }
                                        _ => 0.0,
                                    };

                                    if similarity >= threshold {
                                        similar_pairs.push(SimilarityResult::new(
                                            func1.clone(),
                                            func2.clone(),
                                            similarity,
                                        ));
                                    }
                                }
                            }

                            if similar_pairs.is_empty() {
                                None
                            } else {
                                Some((file.clone(), similar_pairs))
                            }
                        }
                        Err(_) => None,
                    },
                    Err(_) => None,
                }
            }
            Err(_) => None,
        })
        .collect()
}

/// Extract function body from lines
fn extract_function_body(lines: &[&str], func: &GenericFunctionDef) -> String {
    let start_idx = (func.body_start_line.saturating_sub(1)) as usize;
    let end_idx = std::cmp::min(func.body_end_line as usize, lines.len());

    if start_idx >= lines.len() {
        return String::new();
    }

    lines[start_idx..end_idx].join("\n")
}
