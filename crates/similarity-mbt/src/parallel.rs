#![allow(clippy::uninlined_format_args)]

use crate::moonbit_parser::MoonBitParser;
use rayon::prelude::*;
use similarity_core::{
    cli_parallel::{FileData, SimilarityResult},
    language_parser::{GenericFunctionDef, LanguageParser},
    tree::TreeNode,
    tsed::{calculate_tsed, TSEDOptions},
};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

/// MoonBit file with its content and extracted functions
#[allow(dead_code)]
pub type MoonBitFileData = FileData<GenericFunctionDef>;

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

                            // Pre-parse all function bodies into TreeNodes and cache sizes
                            let parsed: Vec<Option<(Rc<TreeNode>, usize)>> = functions
                                .iter()
                                .map(|func| {
                                    let body = extract_function_body(&lines, func);
                                    parser.parse(&body, "body.mbt").ok().map(|tree| {
                                        let size = tree.get_subtree_size();
                                        (tree, size)
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
                                        (Some((tree1, size1)), Some((tree2, size2))) => {
                                            // Quick pre-filter: max possible similarity
                                            // is min_size/max_size (from edit distance lower bound).
                                            // Skip expensive APTED if it can't reach threshold.
                                            let (min_s, max_s) = if size1 <= size2 {
                                                (*size1, *size2)
                                            } else {
                                                (*size2, *size1)
                                            };
                                            if max_s == 0
                                                || (min_s as f64 / max_s as f64) < threshold
                                            {
                                                0.0
                                            } else {
                                                calculate_tsed(tree1, tree2, options)
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
