#![allow(clippy::uninlined_format_args)]

use rayon::prelude::*;
use similarity_core::{
    cli_parallel::{FileData, SimilarityResult},
    language_parser::{GenericFunctionDef, LanguageParser},
    tsed::TSEDOptions,
};
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

                                        // Extract function bodies
                                        let lines: Vec<&str> = code.lines().collect();
                                        let body1 = extract_function_body(&lines, func1);
                                        let body2 = extract_function_body(&lines, func2);

                                        // Parse function bodies to trees
                                        let (tree1_opt, tree2_opt) = match (
                                            parser.parse(&body1, &format!("{}:func1", file_str)),
                                            parser.parse(&body2, &format!("{}:func2", file_str)),
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
                                                similarity_core::tsed::calculate_tsed(
                                                    &tree1, &tree2, options,
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

/// Extract function body only (excluding signature)
fn extract_function_body(lines: &[&str], func: &GenericFunctionDef) -> String {
    // Extract only the body (between body_start_line and body_end_line)
    // If body_start_line/body_end_line are not set, fall back to using the whole function
    let start_idx = if func.body_start_line > 0 {
        (func.body_start_line.saturating_sub(1)) as usize
    } else {
        (func.start_line.saturating_sub(1)) as usize
    };

    let end_idx = if func.body_end_line > 0 {
        std::cmp::min(func.body_end_line as usize, lines.len())
    } else {
        std::cmp::min(func.end_line as usize, lines.len())
    };

    if start_idx >= lines.len() {
        return String::new();
    }

    lines[start_idx..end_idx].join("\n")
}
