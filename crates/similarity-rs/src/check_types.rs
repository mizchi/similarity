use anyhow::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;
use similarity_core::language_parser::{GenericTypeDef, LanguageParser};
use similarity_core::tsed::{calculate_tsed, TSEDOptions};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::rust_parser::RustParser;

/// Type fingerprint for grouping similar types
fn generate_type_fingerprint(type_def: &GenericTypeDef) -> String {
    let mut fingerprint_parts = Vec::new();

    // Add type kind
    fingerprint_parts.push(format!("kind:{}", type_def.kind));

    // Add field count
    fingerprint_parts.push(format!("fields:{}", type_def.fields.len()));

    // For enums, we care about the number of variants
    // For structs, we care about the number of fields
    if type_def.kind == "enum" {
        fingerprint_parts.push(format!("variants:{}", type_def.fields.len()));
    }

    fingerprint_parts.join(",")
}

/// Group types by their fingerprints for efficient comparison
fn group_types_by_fingerprint(types: &[ExtractedType]) -> HashMap<String, Vec<usize>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();

    for (index, extracted_type) in types.iter().enumerate() {
        let fingerprint = generate_type_fingerprint(&extracted_type.type_def);
        groups.entry(fingerprint).or_insert_with(Vec::new).push(index);
    }

    groups
}

/// Check if two fingerprints are similar enough to warrant detailed comparison
fn are_fingerprints_similar(fp1: &str, fp2: &str) -> bool {
    let parts1: HashMap<&str, &str> = fp1
        .split(',')
        .filter_map(|p| {
            let mut iter = p.split(':');
            Some((iter.next()?, iter.next()?))
        })
        .collect();

    let parts2: HashMap<&str, &str> = fp2
        .split(',')
        .filter_map(|p| {
            let mut iter = p.split(':');
            Some((iter.next()?, iter.next()?))
        })
        .collect();

    // Check if they have the same kind
    if let (Some(kind1), Some(kind2)) = (parts1.get("kind"), parts2.get("kind")) {
        if kind1 != kind2 {
            return false;
        }
    }

    // Check field count difference
    if let (Some(fields1), Some(fields2)) = (parts1.get("fields"), parts2.get("fields")) {
        if let (Ok(count1), Ok(count2)) = (fields1.parse::<usize>(), fields2.parse::<usize>()) {
            let diff = (count1 as isize - count2 as isize).abs();
            // Allow up to 2 field difference
            if diff > 2 {
                return false;
            }
        }
    }

    true
}

struct ExtractedType {
    type_def: GenericTypeDef,
    file_path: String,
    content: String,
}

/// Compare two types by converting them to AST and calculating similarity
fn compare_types(
    type1: &ExtractedType,
    type2: &ExtractedType,
    parser: &mut RustParser,
    options: &TSEDOptions,
) -> Result<f64> {
    // Skip comparing the same type in the same file
    if type1.type_def.name == type2.type_def.name && type1.file_path == type2.file_path {
        return Ok(0.0);
    }

    // Extract the type definition code from the source
    let type1_code = extract_type_code(&type1.content, &type1.type_def);
    let type2_code = extract_type_code(&type2.content, &type2.type_def);

    // Parse to AST
    let tree1 = parser
        .parse(&type1_code, &type1.file_path)
        .map_err(|e| anyhow::anyhow!("Failed to parse type1: {}", e))?;
    let tree2 = parser
        .parse(&type2_code, &type2.file_path)
        .map_err(|e| anyhow::anyhow!("Failed to parse type2: {}", e))?;

    // Calculate similarity
    let similarity = calculate_tsed(&tree1, &tree2, options);
    Ok(similarity)
}

/// Extract the code for a specific type definition
fn extract_type_code(content: &str, type_def: &GenericTypeDef) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start = (type_def.start_line as usize).saturating_sub(1);
    let end = (type_def.end_line as usize).min(lines.len());

    if start < lines.len() && end > start {
        lines[start..end].join("\n")
    } else {
        String::new()
    }
}

/// Get relative path for display
fn get_relative_path(file_path: &str) -> String {
    if let Ok(current_dir) = std::env::current_dir() {
        Path::new(file_path)
            .strip_prefix(&current_dir)
            .unwrap_or(Path::new(file_path))
            .to_string_lossy()
            .to_string()
    } else {
        file_path.to_string()
    }
}

/// Check for similar types (structs, enums) across files
pub fn check_types(
    paths: Vec<String>,
    threshold: f64,
    extensions: Option<&Vec<String>>,
    print: bool,
    exclude_patterns: &[String],
) -> Result<usize> {
    let default_extensions = vec!["rs".to_string()];
    let exts = extensions.unwrap_or(&default_extensions);

    // Collect all Rust files
    let mut files = Vec::new();
    let mut visited = HashSet::new();

    for path_str in &paths {
        let path = Path::new(path_str);

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if exts.iter().any(|e| e == ext_str) {
                        if let Ok(canonical) = path.canonicalize() {
                            if visited.insert(canonical.clone()) {
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

                // Check if path should be excluded
                for pattern in exclude_patterns {
                    if entry_path.to_string_lossy().contains(pattern) {
                        continue;
                    }
                }

                if let Some(ext) = entry_path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if exts.iter().any(|e| e == ext_str) {
                            if let Ok(canonical) = entry_path.canonicalize() {
                                if visited.insert(canonical.clone()) {
                                    files.push(entry_path.to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if files.is_empty() {
        println!("No Rust files found in specified paths");
        return Ok(0);
    }

    println!("Checking {} files for similar types...\n", files.len());

    // Extract all types from all files in parallel
    let extracted_types: Vec<ExtractedType> = files
        .par_iter()
        .flat_map(|file| {
            let content = fs::read_to_string(file).ok()?;
            let file_path = file.to_string_lossy().to_string();

            let mut parser = RustParser::new().ok()?;
            let types = parser.extract_types(&content, &file_path).ok()?;

            Some(
                types
                    .into_iter()
                    .map(move |type_def| ExtractedType {
                        type_def,
                        file_path: file_path.clone(),
                        content: content.clone(),
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect();

    if extracted_types.is_empty() {
        println!("No types (structs/enums) found in the specified files");
        return Ok(0);
    }

    println!("Found {} types to analyze\n", extracted_types.len());

    // Group types by fingerprint for optimization
    let fingerprint_groups = group_types_by_fingerprint(&extracted_types);

    // Set up comparison options
    let mut options = TSEDOptions::default();
    // Higher rename cost for types since field/variant names often differ
    options.apted_options.rename_cost = 0.8;
    options.apted_options.compare_values = true;

    // Find similar types
    let mut similar_pairs = Vec::new();
    let mut parser =
        RustParser::new().map_err(|e| anyhow::anyhow!("Failed to create parser: {}", e))?;

    // First, compare types within the same fingerprint group
    for (_fingerprint, indices) in &fingerprint_groups {
        if indices.len() < 2 {
            continue;
        }

        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                let idx1 = indices[i];
                let idx2 = indices[j];
                let type1 = &extracted_types[idx1];
                let type2 = &extracted_types[idx2];

                if let Ok(similarity) = compare_types(type1, type2, &mut parser, &options) {
                    if similarity >= threshold {
                        similar_pairs.push((idx1, idx2, similarity));
                    }
                }
            }
        }
    }

    // Then, compare types from different groups but with similar fingerprints
    let fingerprints: Vec<_> = fingerprint_groups.keys().collect();
    for i in 0..fingerprints.len() {
        for j in (i + 1)..fingerprints.len() {
            let fp1 = fingerprints[i];
            let fp2 = fingerprints[j];

            if are_fingerprints_similar(fp1, fp2) {
                for &idx1 in &fingerprint_groups[fp1] {
                    for &idx2 in &fingerprint_groups[fp2] {
                        let type1 = &extracted_types[idx1];
                        let type2 = &extracted_types[idx2];

                        if let Ok(similarity) = compare_types(type1, type2, &mut parser, &options) {
                            if similarity >= threshold {
                                similar_pairs.push((idx1, idx2, similarity));
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by similarity (descending)
    similar_pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    // Display results
    if similar_pairs.is_empty() {
        println!("No similar types found with threshold {:.0}%", threshold * 100.0);
    } else {
        println!("Similar types found:");
        println!("{}", "-".repeat(60));

        for (idx1, idx2, similarity) in &similar_pairs {
            let type1 = &extracted_types[*idx1];
            let type2 = &extracted_types[*idx2];

            println!("\nSimilarity: {:.2}%", similarity * 100.0);
            println!(
                "  {} {} | {}:{}",
                type1.type_def.kind,
                type1.type_def.name,
                get_relative_path(&type1.file_path),
                type1.type_def.start_line
            );
            println!(
                "  {} {} | {}:{}",
                type2.type_def.kind,
                type2.type_def.name,
                get_relative_path(&type2.file_path),
                type2.type_def.start_line
            );

            if print {
                println!("\n\x1b[36m--- Type 1 ---\x1b[0m");
                println!("{}", extract_type_code(&type1.content, &type1.type_def));
                println!("\n\x1b[36m--- Type 2 ---\x1b[0m");
                println!("{}", extract_type_code(&type2.content, &type2.type_def));
            }
        }

        println!("\n{}", "-".repeat(60));
        println!("Total similar type pairs found: {}", similar_pairs.len());
    }

    Ok(similar_pairs.len())
}
