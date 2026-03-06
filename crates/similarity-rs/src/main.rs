use anyhow::Result;
use clap::Parser;
use similarity_core::ConfigLoader;

mod check;
mod check_types;
mod config;
mod parallel;
mod rust_parser;

use config::{Cli, Config, ResolvedConfig};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let print = cli.print;
    let paths = cli.paths.clone();
    let cfg = Config::find_and_load();
    let r = ResolvedConfig::from(cli, cfg);

    let functions_enabled = !r.no_functions;

    if !functions_enabled && !r.types_enabled && !r.overlap {
        eprintln!("Error: At least one analyzer must be enabled. Use --experimental-types to enable type checking, --experimental-overlap for overlap detection, or remove --no-functions.");
        return Err(anyhow::anyhow!("No analyzer enabled"));
    }

    println!("Analyzing Rust code similarity...\n");

    let separator = "-".repeat(60);
    let mut total_duplicates = 0;

    if functions_enabled {
        println!("=== Function Similarity ===");
        total_duplicates += check::check_paths(
            paths.clone(),
            r.threshold,
            r.rename_cost,
            r.extensions.as_ref(),
            r.min_lines,
            r.min_tokens,
            r.no_size_penalty,
            print,
            !r.no_fast,
            r.filter_function.as_ref(),
            r.filter_function_body.as_ref(),
            &r.exclude,
            r.skip_test,
        )?;
    }

    if r.types_enabled && functions_enabled {
        println!("\n{separator}\n");
    }

    if r.types_enabled {
        println!("=== Type Similarity (Structs & Enums) ===");
        total_duplicates += check_types::check_types(
            paths.clone(),
            r.threshold,
            r.extensions.as_ref(),
            print,
            &r.exclude,
            r.use_structure_comparison,
        )?;
    }

    if r.overlap && (functions_enabled || r.types_enabled) {
        println!("\n{separator}\n");
    }

    if r.overlap {
        println!("=== Overlap Detection ===");
        total_duplicates += check_overlaps(
            paths,
            r.threshold,
            r.extensions.as_ref(),
            print,
            r.overlap_min_window,
            r.overlap_max_window,
            r.overlap_size_tolerance,
            &r.exclude,
        )?;
    }

    if r.fail_on_duplicates && total_duplicates > 0 {
        std::process::exit(1);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn check_overlaps(
    paths: Vec<String>,
    threshold: f64,
    extensions: Option<&Vec<String>>,
    print: bool,
    min_window_size: u32,
    max_window_size: u32,
    size_tolerance: f64,
    exclude_patterns: &[String],
) -> anyhow::Result<usize> {
    use crate::rust_parser::RustParser;
    use ignore::WalkBuilder;
    use similarity_core::{find_overlaps_across_files_generic, OverlapOptions};
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::Path;

    let default_extensions = vec!["rs"];
    let exts: Vec<&str> =
        extensions.map_or(default_extensions, |v| v.iter().map(String::as_str).collect());

    let exclude_matcher = create_exclude_matcher(exclude_patterns);
    let mut files = Vec::new();
    let mut visited = HashSet::new();

    for path_str in &paths {
        let path = Path::new(path_str);

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if exts.contains(&ext_str) {
                        if let Ok(canonical) = path.canonicalize() {
                            if visited.insert(canonical) {
                                files.push(path.to_path_buf());
                            }
                        }
                    }
                }
            }
        } else if path.is_dir() {
            let walker = WalkBuilder::new(path).follow_links(false).build();

            for entry in walker {
                let entry = entry?;
                let entry_path = entry.path();

                if !entry_path.is_file() {
                    continue;
                }

                if let Some(ref matcher) = exclude_matcher {
                    if matcher.is_match(entry_path) {
                        continue;
                    }
                }

                if let Some(ext) = entry_path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if exts.contains(&ext_str) {
                            if let Ok(canonical) = entry_path.canonicalize() {
                                if visited.insert(canonical) {
                                    files.push(entry_path.to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!("Warning: Path not found: {path_str}");
        }
    }

    if files.is_empty() {
        println!("No Rust files found in specified paths");
        return Ok(0);
    }

    println!("Checking {} files for overlapping code...\n", files.len());

    let mut file_contents = HashMap::new();
    for file in &files {
        match fs::read_to_string(file) {
            Ok(content) => {
                file_contents.insert(file.to_string_lossy().to_string(), content);
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
            }
        }
    }

    let options = OverlapOptions { min_window_size, max_window_size, threshold, size_tolerance };
    let mut parser =
        RustParser::new().map_err(|e| anyhow::anyhow!("Failed to create Rust parser: {}", e))?;

    let overlaps = find_overlaps_across_files_generic(&mut parser, &file_contents, &options)
        .map_err(|e| anyhow::anyhow!("Failed to find overlaps: {}", e))?;

    if overlaps.is_empty() {
        println!("\nNo code overlaps found!");
    } else {
        println!("\nCode overlaps found:");
        println!("{}", "-".repeat(60));

        for overlap_with_files in &overlaps {
            let overlap = &overlap_with_files.overlap;
            let source_path = get_relative_path(&overlap_with_files.source_file);
            let target_path = get_relative_path(&overlap_with_files.target_file);

            println!(
                "\nSimilarity: {:.2}% | {} nodes | {}",
                overlap.similarity * 100.0,
                overlap.node_count,
                overlap.node_type
            );
            println!(
                "  {}:{} | L{}-{} in function: {}",
                source_path,
                overlap.source_lines.0,
                overlap.source_lines.0,
                overlap.source_lines.1,
                overlap.source_function
            );
            println!(
                "  {}:{} | L{}-{} in function: {}",
                target_path,
                overlap.target_lines.0,
                overlap.target_lines.0,
                overlap.target_lines.1,
                overlap.target_function
            );

            if print {
                if let Some(source_content) = file_contents.get(&overlap_with_files.source_file) {
                    if let Some(target_content) =
                        file_contents.get(&overlap_with_files.target_file)
                    {
                        println!("\n\x1b[36m--- Source Code ---\x1b[0m");
                        if let Ok(source_segment) = extract_code_lines(
                            source_content,
                            overlap.source_lines.0,
                            overlap.source_lines.1,
                        ) {
                            println!("{source_segment}");
                        }

                        println!("\n\x1b[36m--- Target Code ---\x1b[0m");
                        if let Ok(target_segment) = extract_code_lines(
                            target_content,
                            overlap.target_lines.0,
                            overlap.target_lines.1,
                        ) {
                            println!("{target_segment}");
                        }
                    }
                }
            }
        }

        println!("\nTotal overlaps found: {}", overlaps.len());
    }

    Ok(overlaps.len())
}

fn create_exclude_matcher(exclude_patterns: &[String]) -> Option<globset::GlobSet> {
    if exclude_patterns.is_empty() {
        return None;
    }

    let mut builder = globset::GlobSetBuilder::new();
    for pattern in exclude_patterns {
        if let Ok(glob) = globset::Glob::new(pattern) {
            builder.add(glob);
        } else {
            eprintln!("Warning: Invalid glob pattern: {pattern}");
        }
    }

    builder.build().ok()
}

fn get_relative_path(file_path: &str) -> String {
    if let Ok(current_dir) = std::env::current_dir() {
        std::path::Path::new(file_path)
            .strip_prefix(&current_dir)
            .unwrap_or(std::path::Path::new(file_path))
            .to_string_lossy()
            .to_string()
    } else {
        file_path.to_string()
    }
}

fn extract_code_lines(code: &str, start_line: u32, end_line: u32) -> Result<String, String> {
    let lines: Vec<_> = code.lines().collect();

    if start_line as usize > lines.len() || end_line as usize > lines.len() {
        return Err("Line numbers out of bounds".to_string());
    }

    let start = (start_line as usize).saturating_sub(1);
    let end = (end_line as usize).min(lines.len());

    Ok(lines[start..end].join("\n"))
}
