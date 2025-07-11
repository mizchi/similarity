#![allow(clippy::uninlined_format_args)]

use crate::parallel::check_within_file_duplicates_parallel;
use similarity_core::{
    cli_file_utils::collect_files,
    cli_output::{format_function_output, show_function_code},
    cli_parallel::SimilarityResult,
    language_parser::GenericFunctionDef,
    TSEDOptions,
};
use std::path::PathBuf;

/// Structure to hold all similarity results
struct DuplicateResult {
    file1: PathBuf,
    #[allow(dead_code)]
    file2: PathBuf,
    result: SimilarityResult<GenericFunctionDef>,
}

impl DuplicateResult {
    fn priority(&self) -> f64 {
        // Score = Similarity × Average lines
        let avg_lines = ((self.result.func1.end_line - self.result.func1.start_line + 1)
            + (self.result.func2.end_line - self.result.func2.start_line + 1))
            as f64
            / 2.0;
        self.result.similarity * avg_lines
    }
}

#[allow(clippy::too_many_arguments)]
pub fn check_paths(
    paths: Vec<String>,
    threshold: f64,
    rename_cost: f64,
    extensions: Option<&Vec<String>>,
    min_lines: u32,
    min_tokens: Option<u32>,
    no_size_penalty: bool,
    print: bool,
    _fast_mode: bool, // Rust doesn't support fast mode yet
    filter_function: Option<&String>,
    filter_function_body: Option<&String>,
    _exclude_patterns: &[String],
    skip_test: bool,
) -> anyhow::Result<usize> {
    let default_extensions = vec!["rs"];
    let exts: Vec<&str> =
        extensions.map_or(default_extensions, |v| v.iter().map(String::as_str).collect());

    let files = collect_files(&paths, &exts)?;

    if files.is_empty() {
        println!("No Rust files found in the specified paths.");
        return Ok(0);
    }

    println!("Checking {} files for duplicates...", files.len());

    let mut options = TSEDOptions::default();
    options.apted_options.rename_cost = rename_cost;
    options.apted_options.compare_values = true; // Rust: compare both node labels and values
    options.min_lines = min_lines;
    options.min_tokens = min_tokens;
    options.size_penalty = !no_size_penalty;
    options.skip_test = skip_test;

    let mut all_results = Vec::new();

    // Check within each file
    let within_file_results = check_within_file_duplicates_parallel(&files, threshold, &options);

    // Collect within-file duplicates
    for (file, similar_pairs) in within_file_results {
        for result in similar_pairs {
            all_results.push(DuplicateResult { file1: file.clone(), file2: file.clone(), result });
        }
    }

    // For now, we only support within-file duplicates for Rust
    // Cross-file support can be added later

    // Display results
    let duplicate_count = display_all_results(all_results, print, filter_function, filter_function_body);

    Ok(duplicate_count)
}

/// Display similarity results
fn display_all_results(
    mut all_results: Vec<DuplicateResult>,
    print: bool,
    filter_function: Option<&String>,
    filter_function_body: Option<&String>,
) -> usize {
    if all_results.is_empty() {
        println!("\nNo duplicate functions found!");
        return 0;
    }

    // Apply filters if specified
    if filter_function.is_some() || filter_function_body.is_some() {
        all_results.retain(|dup| {
            // Check function name filter
            if let Some(filter) = filter_function {
                if !dup.result.func1.name.contains(filter)
                    && !dup.result.func2.name.contains(filter)
                {
                    return false;
                }
            }

            // For body filter, we'd need to read the file content
            // This is a simplified version
            true
        });
    }

    // Sort by priority (higher similarity × larger functions first)
    all_results.sort_by(|a, b| {
        b.priority().partial_cmp(&a.priority()).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Group by file
    let mut file_groups = std::collections::HashMap::new();
    for dup in all_results {
        let file_path = dup.file1.to_string_lossy().to_string();
        file_groups.entry(file_path).or_insert_with(Vec::new).push(dup);
    }

    // Display results grouped by file
    let mut total_count = 0;
    for (file_path, duplicates) in file_groups {
        println!("\nDuplicates in {}:", file_path);
        println!("{}", "-".repeat(60));

        for dup in &duplicates {
            let func1 = &dup.result.func1;
            let func2 = &dup.result.func2;

            println!(
                "  {} <-> {}",
                format_function_output(
                    &file_path,
                    &format!(
                        "{} {}",
                        if func1.is_method { "method" } else { "function" },
                        &func1.name
                    ),
                    func1.start_line,
                    func1.end_line
                ),
                format_function_output(
                    &file_path,
                    &format!(
                        "{} {}",
                        if func2.is_method { "method" } else { "function" },
                        &func2.name
                    ),
                    func2.start_line,
                    func2.end_line
                )
            );
            println!("  Similarity: {:.2}%", dup.result.similarity * 100.0);

            if let (Some(class1), Some(class2)) = (&func1.class_name, &func2.class_name) {
                println!("  Classes: {} <-> {}", class1, class2);
            }

            if print {
                show_function_code(&file_path, &func1.name, func1.start_line, func1.end_line);
                show_function_code(&file_path, &func2.name, func2.start_line, func2.end_line);
                println!();
            }

            total_count += 1;
        }
    }

    println!("\nTotal duplicate pairs found: {}", total_count);

    total_count
}
