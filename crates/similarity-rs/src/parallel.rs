#![allow(clippy::uninlined_format_args)]

use rayon::prelude::*;
use similarity_core::{
    cli_parallel::{FileData, SimilarityResult},
    language_parser::{GenericFunctionDef, LanguageParser},
    tsed::TSEDOptions,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Rust file with its content and extracted functions
#[allow(dead_code)]
pub type RustFileData = FileData<GenericFunctionDef>;

/// Load and parse Rust files in parallel
#[allow(dead_code)]
pub fn load_files_parallel(files: &[PathBuf]) -> Vec<RustFileData> {
    files
        .par_iter()
        .filter_map(|file| {
            match fs::read_to_string(file) {
                Ok(content) => {
                    let filename = file.to_string_lossy();
                    // Create Rust parser
                    match similarity_rs::rust_parser::RustParser::new() {
                        Ok(mut parser) => {
                            // Extract functions
                            match parser.extract_functions(&content, &filename) {
                                Ok(functions) => {
                                    Some(FileData { path: file.clone(), content, functions })
                                }
                                Err(e) => {
                                    eprintln!("Error parsing {}: {}", file.display(), e);
                                    None
                                }
                            }
                        }
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
            }
        })
        .collect()
}

/// Check for duplicates within Rust files in parallel
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

                // Create Rust parser
                match similarity_rs::rust_parser::RustParser::new() {
                    Ok(mut parser) => {
                        // Extract functions
                        match parser.extract_functions(&code, &file_str) {
                            Ok(mut functions) => {
                                // Filter out test functions if skip_test is enabled
                                if options.skip_test {
                                    functions.retain(|f| {
                                        // Skip if function name starts with "test_"
                                        if f.name.starts_with("test_") {
                                            return false;
                                        }
                                        // Skip if function has #[test] attribute
                                        !f.decorators.iter().any(|d| d.contains("test"))
                                    });
                                }
                                let mut similar_pairs = Vec::new();

                                // Compare all pairs within the file
                                for i in 0..functions.len() {
                                    for j in (i + 1)..functions.len() {
                                        let func1 = &functions[i];
                                        let func2 = &functions[j];

                                        // Skip if functions don't meet minimum requirements
                                        if func1.end_line - func1.start_line + 1 < options.min_lines
                                            || func2.end_line - func2.start_line + 1
                                                < options.min_lines
                                        {
                                            continue;
                                        }

                                        // Extract full function source so Rust signatures
                                        // contribute to similarity, reducing false positives
                                        // on short functions with identical bodies.
                                        let lines: Vec<&str> = code.lines().collect();
                                        let source1 = extract_function_source(&lines, func1);
                                        let source2 = extract_function_source(&lines, func2);

                                        // Parse function source to trees
                                        let (tree1_opt, tree2_opt) = match (
                                            parser.parse(&source1, &format!("{}:func1", file_str)),
                                            parser.parse(&source2, &format!("{}:func2", file_str)),
                                        ) {
                                            (Ok(tree1), Ok(tree2)) => {
                                                // Skip if either tree is empty
                                                if tree1.get_subtree_size() == 0
                                                    || tree2.get_subtree_size() == 0
                                                {
                                                    (None, None)
                                                } else {
                                                    (Some(tree1), Some(tree2))
                                                }
                                            }
                                            _ => (None, None),
                                        };

                                        // Calculate similarity
                                        let similarity = match (tree1_opt, tree2_opt) {
                                            (Some(tree1), Some(tree2)) => {
                                                // Check minimum tokens if specified
                                                if let Some(min_tokens) = options.min_tokens {
                                                    let tokens1 = tree1.get_subtree_size() as u32;
                                                    let tokens2 = tree2.get_subtree_size() as u32;
                                                    if tokens1 < min_tokens || tokens2 < min_tokens
                                                    {
                                                        continue;
                                                    }
                                                }
                                                // For Rust, use TSED instead of enhanced similarity
                                                // to better handle short functions
                                                let body_similarity =
                                                    similarity_core::tsed::calculate_tsed(
                                                        &tree1, &tree2, options,
                                                    );
                                                blend_rust_similarity(
                                                    body_similarity,
                                                    &source1,
                                                    &source2,
                                                    func1,
                                                    func2,
                                                )
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
                        }
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        })
        .collect()
}

/// Extract full function source, including the signature.
fn extract_function_source(lines: &[&str], func: &GenericFunctionDef) -> String {
    let start_idx = (func.start_line.saturating_sub(1)) as usize;
    let end_idx = std::cmp::min(func.end_line as usize, lines.len());

    if start_idx >= lines.len() {
        return String::new();
    }

    lines[start_idx..end_idx].join("\n")
}

fn blend_rust_similarity(
    body_similarity: f64,
    source1: &str,
    source2: &str,
    func1: &GenericFunctionDef,
    func2: &GenericFunctionDef,
) -> f64 {
    let max_lines =
        (func1.end_line - func1.start_line + 1).max(func2.end_line - func2.start_line + 1);
    let signature_weight = if max_lines <= 8 { 0.35 } else { 0.2 };
    let signature_similarity = calculate_signature_similarity(source1, source2);

    (body_similarity * (1.0 - signature_weight) + signature_similarity * signature_weight)
        .clamp(0.0, 1.0)
}

fn calculate_signature_similarity(source1: &str, source2: &str) -> f64 {
    let tokens1 = tokenize_signature(extract_signature(source1));
    let tokens2 = tokenize_signature(extract_signature(source2));

    if tokens1.is_empty() || tokens2.is_empty() {
        return 0.0;
    }

    let counts1 = token_counts(tokens1);
    let counts2 = token_counts(tokens2);

    let mut intersection = 0usize;
    let mut union = 0usize;

    for (token, count1) in &counts1 {
        let count2 = counts2.get(token).copied().unwrap_or(0);
        intersection += (*count1).min(count2);
        union += (*count1).max(count2);
    }

    for (token, count2) in &counts2 {
        if !counts1.contains_key(token) {
            union += *count2;
        }
    }

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

fn extract_signature(source: &str) -> &str {
    source.split('{').next().unwrap_or(source).trim()
}

fn tokenize_signature(signature: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in signature.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn token_counts(tokens: Vec<String>) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for token in tokens {
        *counts.entry(token).or_insert(0) += 1;
    }
    counts
}
