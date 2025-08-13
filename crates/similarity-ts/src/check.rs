#![allow(clippy::uninlined_format_args)]

use crate::parallel::{
    check_cross_file_duplicates_parallel, check_within_file_duplicates_parallel,
    load_files_parallel,
};
use ignore::WalkBuilder;
use similarity_core::TSEDOptions;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn create_exclude_matcher(exclude_patterns: &[String]) -> Option<globset::GlobSet> {
    if exclude_patterns.is_empty() {
        return None;
    }

    let mut builder = globset::GlobSetBuilder::new();
    for pattern in exclude_patterns {
        if let Ok(glob) = globset::Glob::new(pattern) {
            builder.add(glob);
        } else {
            eprintln!("Warning: Invalid glob pattern: {}", pattern);
        }
    }

    builder.build().ok()
}

/// Extract lines from file content within the specified range
fn extract_lines_from_content(content: &str, start_line: u32, end_line: u32) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start_idx = (start_line.saturating_sub(1)) as usize;
    let end_idx = std::cmp::min(end_line as usize, lines.len());

    if start_idx >= lines.len() {
        return String::new();
    }

    lines[start_idx..end_idx].join("\n")
}

/// Format function output in VSCode-compatible format
fn format_function_output(
    file_path: &str,
    function_name: &str,
    start_line: u32,
    end_line: u32,
) -> String {
    format!("{}:{}-{} {}", file_path, start_line, end_line, function_name)
}

/// Display code content for a function
fn show_function_code(file_path: &str, function_name: &str, start_line: u32, end_line: u32) {
    match fs::read_to_string(file_path) {
        Ok(content) => {
            let code = extract_lines_from_content(&content, start_line, end_line);
            println!(
                "\n\x1b[36m--- {}:{} (lines {}-{}) ---\x1b[0m",
                file_path, function_name, start_line, end_line
            );
            println!("{}", code);
        }
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
        }
    }
}

/// Structure to hold all similarity results
struct DuplicateResult {
    file1: PathBuf,
    file2: PathBuf,
    result: similarity_core::SimilarityResult,
}

impl DuplicateResult {
    fn priority(&self) -> f64 {
        // Score = Similarity × Average lines
        let avg_lines =
            (self.result.func1.line_count() + self.result.func2.line_count()) as f64 / 2.0;
        self.result.similarity * avg_lines
    }
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

            // Check function body filter
            if let Some(filter) = filter_function_body {
                // Need to read the file content to check body
                let mut match_found = false;

                // Check first function
                if let Ok(content) = fs::read_to_string(&dup.file1) {
                    let func1_body = extract_lines_from_content(
                        &content,
                        dup.result.func1.start_line,
                        dup.result.func1.end_line,
                    );
                    if func1_body.contains(filter) {
                        match_found = true;
                    }
                }

                // Check second function if no match yet
                if !match_found {
                    if let Ok(content) = fs::read_to_string(&dup.file2) {
                        let func2_body = extract_lines_from_content(
                            &content,
                            dup.result.func2.start_line,
                            dup.result.func2.end_line,
                        );
                        if func2_body.contains(filter) {
                            match_found = true;
                        }
                    }
                }

                if !match_found {
                    return false;
                }
            }

            true
        });
    }

    if all_results.is_empty() {
        println!("\nNo duplicate functions found matching the filters!");
        return 0;
    }

    // Sort by priority (impact * similarity)
    all_results.sort_by(|a, b| {
        b.priority().partial_cmp(&a.priority()).unwrap_or(std::cmp::Ordering::Equal)
    });

    println!("\nFound {} duplicate pairs:", all_results.len());
    println!("{}", "-".repeat(60));

    for dup in &all_results {
        // Get relative paths
        let (relative_path1, relative_path2) = if let Ok(current_dir) = std::env::current_dir() {
            (
                dup.file1
                    .strip_prefix(&current_dir)
                    .unwrap_or(&dup.file1)
                    .to_string_lossy()
                    .to_string(),
                dup.file2
                    .strip_prefix(&current_dir)
                    .unwrap_or(&dup.file2)
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            (dup.file1.to_string_lossy().to_string(), dup.file2.to_string_lossy().to_string())
        };

        // Calculate the line counts
        let line_count1 = dup.result.func1.line_count();
        let line_count2 = dup.result.func2.line_count();
        let min_lines = line_count1.min(line_count2);
        let max_lines = line_count1.max(line_count2);
        let avg_lines = (line_count1 + line_count2) as f64 / 2.0;
        let score = dup.result.similarity * avg_lines;

        println!(
            "\nSimilarity: {:.2}%, Score: {:.1} points (lines {}~{}, avg: {:.1})",
            dup.result.similarity * 100.0,
            score,
            min_lines,
            max_lines,
            avg_lines
        );
        println!(
            "  {}",
            format_function_output(
                &relative_path1,
                &dup.result.func1.name,
                dup.result.func1.start_line,
                dup.result.func1.end_line,
            )
        );
        println!(
            "  {}",
            format_function_output(
                &relative_path2,
                &dup.result.func2.name,
                dup.result.func2.start_line,
                dup.result.func2.end_line,
            )
        );

        if print {
            show_function_code(
                &relative_path1,
                &dup.result.func1.name,
                dup.result.func1.start_line,
                dup.result.func1.end_line,
            );
            show_function_code(
                &relative_path2,
                &dup.result.func2.name,
                dup.result.func2.start_line,
                dup.result.func2.end_line,
            );
        }
    }

    all_results.len()
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
    fast_mode: bool,
    filter_function: Option<&String>,
    filter_function_body: Option<&String>,
    exclude_patterns: &[String],
) -> anyhow::Result<usize> {
    let default_extensions = vec!["ts", "tsx", "js", "jsx", "mjs", "cjs", "mts", "cts"];
    let exts: Vec<&str> =
        extensions.map_or(default_extensions, |v| v.iter().map(String::as_str).collect());

    // Create exclude matcher
    let exclude_matcher = create_exclude_matcher(exclude_patterns);
    let mut files = Vec::new();
    let mut visited = HashSet::new();

    // Process each path
    for path_str in &paths {
        let path = Path::new(path_str);

        if path.is_file() {
            // If it's a file, check extension and add it
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if exts.contains(&ext_str) {
                        if let Ok(canonical) = path.canonicalize() {
                            if visited.insert(canonical.clone()) {
                                files.push(path.to_path_buf());
                            }
                        }
                    }
                }
            }
        } else if path.is_dir() {
            // If it's a directory, walk it respecting .gitignore
            let walker = WalkBuilder::new(path).follow_links(false).build();

            for entry in walker {
                let entry = entry?;
                let entry_path = entry.path();

                // Skip if not a file
                if !entry_path.is_file() {
                    continue;
                }

                // Check if path should be excluded
                if let Some(ref matcher) = exclude_matcher {
                    if matcher.is_match(entry_path) {
                        continue;
                    }
                }

                // Check extension
                if let Some(ext) = entry_path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if exts.contains(&ext_str) {
                            if let Ok(canonical) = entry_path.canonicalize() {
                                if visited.insert(canonical.clone()) {
                                    files.push(entry_path.to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!("Path does not exist or is not accessible: {}", path_str);
        }
    }

    // Sort files for consistent output
    files.sort();

    if files.is_empty() {
        println!("No TypeScript/JavaScript files found in the specified paths.");
        return Ok(0);
    }

    println!("Checking {} files for duplicates...", files.len());

    let mut options = TSEDOptions::default();
    options.apted_options.rename_cost = rename_cost;
    options.min_lines = min_lines;
    options.min_tokens = min_tokens;
    options.size_penalty = !no_size_penalty;

    let mut all_results = Vec::new();

    // Check within each file in parallel
    let within_file_results =
        check_within_file_duplicates_parallel(&files, threshold, &options, fast_mode);

    // Collect within-file duplicates
    for (file, similar_pairs) in within_file_results {
        for result in similar_pairs {
            all_results.push(DuplicateResult { file1: file.clone(), file2: file.clone(), result });
        }
    }

    // Check across files in parallel
    let file_data = load_files_parallel(&files);
    let cross_file_results =
        check_cross_file_duplicates_parallel(&file_data, threshold, &options, fast_mode);

    // Collect cross-file duplicates
    for (file1, result, file2) in cross_file_results {
        all_results.push(DuplicateResult {
            file1: PathBuf::from(file1),
            file2: PathBuf::from(file2),
            result,
        });
    }

    // Display all results together
    let duplicate_count =
        display_all_results(all_results, print, filter_function, filter_function_body);

    Ok(duplicate_count)
}
