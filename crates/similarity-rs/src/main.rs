use anyhow::Result;
use clap::Parser;

mod check;
mod check_types;
mod parallel;
mod rust_parser;

#[derive(Parser)]
#[command(name = "similarity-rs")]
#[command(about = "Rust code similarity analyzer")]
#[command(version)]
struct Cli {
    /// Paths to analyze (files or directories)
    #[arg(default_value = ".")]
    paths: Vec<String>,

    /// Print code in output
    #[arg(short, long)]
    print: bool,

    /// Similarity threshold (0.0-1.0)
    #[arg(short, long, default_value = "0.85")]
    threshold: f64,

    /// File extensions to check
    #[arg(short, long, value_delimiter = ',')]
    extensions: Option<Vec<String>>,

    /// Minimum lines for functions to be considered
    #[arg(short, long, default_value = "3")]
    min_lines: Option<u32>,

    /// Minimum tokens for functions to be considered
    #[arg(long, default_value = "30")]
    min_tokens: Option<u32>,

    /// Rename cost for APTED algorithm
    #[arg(short, long, default_value = "0.3")]
    rename_cost: f64,

    /// Disable size penalty for very different sized functions
    #[arg(long)]
    no_size_penalty: bool,

    /// Filter functions by name (substring match)
    #[arg(long)]
    filter_function: Option<String>,

    /// Filter functions by body content (substring match)
    #[arg(long)]
    filter_function_body: Option<String>,

    /// Disable fast mode with bloom filter pre-filtering
    #[arg(long)]
    no_fast: bool,

    /// Exclude directories matching the given patterns (can be specified multiple times)
    #[arg(long)]
    exclude: Vec<String>,

    /// Skip test functions (functions starting with 'test_' or annotated with #[test])
    #[arg(long)]
    skip_test: bool,

    /// Enable experimental overlap detection mode
    #[arg(long = "experimental-overlap")]
    overlap: bool,

    /// Minimum window size for overlap detection (number of nodes)
    #[arg(long, default_value = "8")]
    overlap_min_window: u32,

    /// Maximum window size for overlap detection (number of nodes)
    #[arg(long, default_value = "25")]
    overlap_max_window: u32,

    /// Size tolerance for overlap detection (0.0-1.0)
    #[arg(long, default_value = "0.25")]
    overlap_size_tolerance: f64,

    /// Exit with code 1 if duplicates are found
    #[arg(long)]
    fail_on_duplicates: bool,

    /// Enable type similarity checking for structs and enums (experimental)
    #[arg(long = "experimental-types")]
    types: bool,

    /// Disable function similarity checking
    #[arg(long = "no-functions")]
    no_functions: bool,
    
    /// Use new generalized structure comparison framework (experimental)
    #[arg(long)]
    use_structure_comparison: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let functions_enabled = !cli.no_functions;
    let types_enabled = cli.types;
    let overlap_enabled = cli.overlap;

    // Validate that at least one analyzer is enabled
    if !functions_enabled && !types_enabled && !overlap_enabled {
        eprintln!("Error: At least one analyzer must be enabled. Use --experimental-types to enable type checking, --experimental-overlap for overlap detection, or remove --no-functions.");
        return Err(anyhow::anyhow!("No analyzer enabled"));
    }

    println!("Analyzing Rust code similarity...\n");

    let separator = "-".repeat(60);
    let mut total_duplicates = 0;

    // Run functions analysis
    if functions_enabled {
        println!("=== Function Similarity ===");
        let duplicate_count = check::check_paths(
            cli.paths.clone(),
            cli.threshold,
            cli.rename_cost,
            cli.extensions.as_ref(),
            cli.min_lines.unwrap_or(3),
            cli.min_tokens,
            cli.no_size_penalty,
            cli.print,
            !cli.no_fast,
            cli.filter_function.as_ref(),
            cli.filter_function_body.as_ref(),
            &cli.exclude,
            cli.skip_test,
        )?;
        total_duplicates += duplicate_count;
    }

    // Run types analysis if enabled
    if types_enabled && functions_enabled {
        println!("\n{separator}\n");
    }

    if types_enabled {
        println!("=== Type Similarity (Structs & Enums) ===");
        let type_duplicate_count = check_types::check_types(
            cli.paths.clone(),
            cli.threshold,
            cli.extensions.as_ref(),
            cli.print,
            &cli.exclude,
            cli.use_structure_comparison,
        )?;
        total_duplicates += type_duplicate_count;
    }

    // Run overlap analysis if enabled
    if overlap_enabled && (functions_enabled || types_enabled) {
        println!("\n{separator}\n");
    }

    if overlap_enabled {
        println!("=== Overlap Detection ===");
        let overlap_duplicate_count = check_overlaps(
            cli.paths,
            cli.threshold,
            cli.extensions.as_ref(),
            cli.print,
            cli.overlap_min_window,
            cli.overlap_max_window,
            cli.overlap_size_tolerance,
            &cli.exclude,
        )?;
        total_duplicates += overlap_duplicate_count;
    }

    // Exit with code 1 if duplicates found and --fail-on-duplicates is set
    if cli.fail_on_duplicates && total_duplicates > 0 {
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
                            // Get canonical path to avoid duplicates
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
            eprintln!("Warning: Path not found: {path_str}");
        }
    }

    if files.is_empty() {
        println!("No Rust files found in specified paths");
        return Ok(0);
    }

    println!("Checking {} files for overlapping code...\n", files.len());

    // Read all file contents
    let mut file_contents = HashMap::new();
    for file in &files {
        match fs::read_to_string(file) {
            Ok(content) => {
                let file_str = file.to_string_lossy().to_string();
                file_contents.insert(file_str, content);
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
            }
        }
    }

    // Set up overlap options
    let options = OverlapOptions { min_window_size, max_window_size, threshold, size_tolerance };

    // Create Rust parser
    let mut parser =
        RustParser::new().map_err(|e| anyhow::anyhow!("Failed to create Rust parser: {}", e))?;

    // Find overlaps
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
                // Extract and display the overlapping code
                if let Some(source_content) = file_contents.get(&overlap_with_files.source_file) {
                    if let Some(target_content) = file_contents.get(&overlap_with_files.target_file)
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
