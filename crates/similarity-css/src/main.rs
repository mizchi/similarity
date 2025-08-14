use clap::Parser as ClapParser;
use ignore::WalkBuilder;
use similarity_core::language_parser::LanguageParser;
use similarity_core::css_structure_adapter::{CssStructDef, CssBatchComparator};
use similarity_css::{convert_to_css_rule, CssParser, DuplicateAnalyzer};
use std::path::PathBuf;

#[derive(ClapParser, Debug)]
#[command(author, version, about = "Find similar CSS rules and declarations", long_about = None)]
struct Args {
    #[arg(help = "Target directory or file")]
    target: String,

    #[arg(short, long, default_value = "0.8", help = "Similarity threshold (0.0-1.0)")]
    threshold: f64,

    #[arg(
        short,
        long,
        default_value = "standard",
        help = "Output format (standard, vscode, json)"
    )]
    output: String,

    #[arg(long, help = "File extension to search for", default_value = "css")]
    extension: String,

    #[arg(long, help = "Process SCSS files instead of CSS")]
    scss: bool,

    #[arg(
        long,
        default_value = "0.3",
        help = "Size difference penalty factor. Higher values penalize size differences more"
    )]
    size_penalty: f64,

    #[arg(
        long,
        default_value = "3",
        help = "Minimum rule size (in declarations) to consider for comparison"
    )]
    min_size: usize,

    #[arg(
        long,
        help = "Use structure-based comparison instead of AST-based comparison"
    )]
    use_structure_comparison: bool,
}

fn find_files(path: &str, extension: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let target_path = std::path::Path::new(path);

    if target_path.is_file() {
        if target_path.extension().and_then(|s| s.to_str()) == Some(extension) {
            files.push(target_path.to_path_buf());
        }
    } else if target_path.is_dir() {
        let walker = WalkBuilder::new(target_path).follow_links(false).build();

        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some(extension) {
                files.push(path.to_path_buf());
            }
        }
    }

    files
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let extension = if args.scss { "scss" } else { &args.extension };
    let files = find_files(&args.target, extension);

    if files.is_empty() {
        eprintln!("No {extension} files found in the specified path");
        return Ok(());
    }

    // For now, just print files found
    println!("Found {} {} files", files.len(), extension);
    for file in &files {
        println!("  {}", file.display());
    }

    // Parse all CSS/SCSS files
    let mut all_rules = Vec::new();
    let mut parser = if args.scss { CssParser::new_scss() } else { CssParser::new() };

    for file in &files {
        let content = std::fs::read_to_string(file)?;
        let file_str = file.to_string_lossy();

        match parser.extract_functions(&content, &file_str) {
            Ok(functions) => {
                for func in functions {
                    let css_rule = convert_to_css_rule(&func, &content);
                    all_rules.push((file_str.to_string(), css_rule));
                }
            }
            Err(e) => {
                eprintln!("Error parsing {file_str}: {e}");
            }
        }
    }

    if all_rules.is_empty() {
        println!("\nNo CSS rules found to analyze");
        return Ok(());
    }

    println!("\nFound {} CSS rules to analyze", all_rules.len());

    if args.use_structure_comparison {
        // Use structure-based comparison
        println!("\nUsing structure-based comparison...");
        analyze_with_structure_comparison(&all_rules, args.threshold, &args.output)?;
    } else {
        // Analyze duplicates with traditional method
        let css_rules: Vec<_> = all_rules.iter().map(|(_, rule)| rule.clone()).collect();
        let analyzer = DuplicateAnalyzer::new(css_rules, args.threshold);
        let result = analyzer.analyze();

        // Output results
        match args.output.as_str() {
            "json" => {
                output_json(&result, &all_rules)?;
            }
            "vscode" => {
                output_vscode(&result, &all_rules);
            }
            _ => {
                output_standard(&result, &all_rules, args.threshold);
            }
        }
    }

    Ok(())
}

fn output_standard(
    result: &similarity_css::DuplicateAnalysisResult,
    all_rules: &[(String, similarity_css::CssRule)],
    threshold: f64,
) {
    println!("\n=== CSS Similarity Analysis Results ===");

    if !result.exact_duplicates.is_empty() {
        println!("\n## Exact Duplicates Found: {}", result.exact_duplicates.len());
        for (i, dup) in result.exact_duplicates.iter().enumerate() {
            let empty_string = String::new();
            let file1 = all_rules
                .iter()
                .find(|(_, r)| r.selector == dup.rule1.selector)
                .map(|(f, _)| f)
                .unwrap_or(&empty_string);
            let file2 = all_rules
                .iter()
                .find(|(_, r)| r.selector == dup.rule2.selector)
                .map(|(f, _)| f)
                .unwrap_or(&empty_string);

            println!("\n{}. {} and {}", i + 1, dup.rule1.selector, dup.rule2.selector);
            println!("   Files: {file1} and {file2}");
            println!(
                "   Lines: {}-{} and {}-{}",
                dup.rule1.start_line, dup.rule1.end_line, dup.rule2.start_line, dup.rule2.end_line
            );
        }
    }

    if !result.style_duplicates.is_empty() {
        println!("\n## Similar Styles Found: {}", result.style_duplicates.len());
        for (i, dup) in result.style_duplicates.iter().enumerate() {
            let empty_string = String::new();
            let file1 = all_rules
                .iter()
                .find(|(_, r)| r.selector == dup.rule1.selector)
                .map(|(f, _)| f)
                .unwrap_or(&empty_string);
            let file2 = all_rules
                .iter()
                .find(|(_, r)| r.selector == dup.rule2.selector)
                .map(|(f, _)| f)
                .unwrap_or(&empty_string);

            println!(
                "\n{}. {} and {} (similarity: {:.2}%)",
                i + 1,
                dup.rule1.selector,
                dup.rule2.selector,
                dup.similarity * 100.0
            );
            println!("   Files: {file1} and {file2}");
            println!(
                "   Lines: {}-{} and {}-{}",
                dup.rule1.start_line, dup.rule1.end_line, dup.rule2.start_line, dup.rule2.end_line
            );
        }
    }

    if !result.bem_variations.is_empty() {
        println!("\n## BEM Component Variations Found: {}", result.bem_variations.len());
        for (i, variation) in result.bem_variations.iter().enumerate() {
            println!("\n{}. BEM variation: {}", i + 1, variation.rule1.selector);
            println!("   Similar to: {}", variation.rule2.selector);
            println!("   Similarity: {:.2}%", variation.similarity * 100.0);
        }
    }

    if result.exact_duplicates.is_empty() && result.style_duplicates.is_empty() {
        println!("\nNo duplicates found with threshold >= {threshold}");
    }

    // Summary
    println!("\n## Summary");
    println!("Total rules analyzed: {}", all_rules.len());
    println!("Exact duplicates: {}", result.exact_duplicates.len());
    println!("Similar styles: {}", result.style_duplicates.len());
    println!("BEM components: {}", result.bem_variations.len());
}

fn output_vscode(
    result: &similarity_css::DuplicateAnalysisResult,
    all_rules: &[(String, similarity_css::CssRule)],
) {
    // VSCode problem matcher format
    let empty_string = String::new();
    for dup in &result.exact_duplicates {
        let file1 = all_rules
            .iter()
            .find(|(_, r)| r.selector == dup.rule1.selector)
            .map(|(f, _)| f)
            .unwrap_or(&empty_string);
        let file2 = all_rules
            .iter()
            .find(|(_, r)| r.selector == dup.rule2.selector)
            .map(|(f, _)| f)
            .unwrap_or(&empty_string);

        println!(
            "{}:{}:1: warning: Exact duplicate of {} at {}:{}",
            file1, dup.rule1.start_line, dup.rule2.selector, file2, dup.rule2.start_line
        );
    }

    for dup in &result.style_duplicates {
        let file1 = all_rules
            .iter()
            .find(|(_, r)| r.selector == dup.rule1.selector)
            .map(|(f, _)| f)
            .unwrap_or(&empty_string);
        let file2 = all_rules
            .iter()
            .find(|(_, r)| r.selector == dup.rule2.selector)
            .map(|(f, _)| f)
            .unwrap_or(&empty_string);

        println!(
            "{}:{}:1: warning: Similar to {} ({:.0}% similarity) at {}:{}",
            file1,
            dup.rule1.start_line,
            dup.rule2.selector,
            dup.similarity * 100.0,
            file2,
            dup.rule2.start_line
        );
    }
}

fn output_json(
    result: &similarity_css::DuplicateAnalysisResult,
    all_rules: &[(String, similarity_css::CssRule)],
) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::json;

    let mut duplicates = Vec::new();
    let empty_string = String::new();

    for dup in &result.exact_duplicates {
        let file1 = all_rules
            .iter()
            .find(|(_, r)| r.selector == dup.rule1.selector)
            .map(|(f, _)| f)
            .unwrap_or(&empty_string);
        let file2 = all_rules
            .iter()
            .find(|(_, r)| r.selector == dup.rule2.selector)
            .map(|(f, _)| f)
            .unwrap_or(&empty_string);

        duplicates.push(json!({
            "type": "exact",
            "rule1": {
                "selector": dup.rule1.selector,
                "file": file1,
                "start_line": dup.rule1.start_line,
                "end_line": dup.rule1.end_line,
            },
            "rule2": {
                "selector": dup.rule2.selector,
                "file": file2,
                "start_line": dup.rule2.start_line,
                "end_line": dup.rule2.end_line,
            }
        }));
    }

    for dup in &result.style_duplicates {
        let file1 = all_rules
            .iter()
            .find(|(_, r)| r.selector == dup.rule1.selector)
            .map(|(f, _)| f)
            .unwrap_or(&empty_string);
        let file2 = all_rules
            .iter()
            .find(|(_, r)| r.selector == dup.rule2.selector)
            .map(|(f, _)| f)
            .unwrap_or(&empty_string);

        duplicates.push(json!({
            "type": "similar",
            "similarity": dup.similarity,
            "rule1": {
                "selector": dup.rule1.selector,
                "file": file1,
                "start_line": dup.rule1.start_line,
                "end_line": dup.rule1.end_line,
            },
            "rule2": {
                "selector": dup.rule2.selector,
                "file": file2,
                "start_line": dup.rule2.start_line,
                "end_line": dup.rule2.end_line,
            }
        }));
    }

    // For BEM variations, just output count for now
    let bem_count = result.bem_variations.len();

    let output = json!({
        "duplicates": duplicates,
        "bem_variations_count": bem_count,
        "summary": {
            "total_rules": all_rules.len(),
            "exact_duplicates": result.exact_duplicates.len(),
            "similar_styles": result.style_duplicates.len(),
            "bem_components": bem_count,
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn analyze_with_structure_comparison(
    all_rules: &[(String, similarity_css::CssRule)],
    threshold: f64,
    output_format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert CSS rules to CssStructDef
    let mut css_structs = Vec::new();
    
    for (file_path, rule) in all_rules {
        let css_struct = CssStructDef {
            selector: rule.selector.clone(),
            declarations: rule.declarations.clone(),
            file_path: file_path.clone(),
            start_line: rule.start_line,
            end_line: rule.end_line,
            media_query: None,
            parent_selectors: vec![],
        };
        css_structs.push(css_struct);
    }
    
    // Use batch comparator for efficient comparison
    let mut batch_comparator = CssBatchComparator::new();
    batch_comparator.group_by_fingerprint(css_structs.clone());
    let similar_rules = batch_comparator.find_similar_rules(threshold);
    
    // Output results
    match output_format {
        "json" => {
            output_structure_json(&similar_rules)?;
        }
        "vscode" => {
            output_structure_vscode(&similar_rules);
        }
        _ => {
            output_structure_standard(&similar_rules, threshold);
        }
    }
    
    Ok(())
}

fn output_structure_standard(
    similar_rules: &[(similarity_core::structure_comparator::Structure, 
                     similarity_core::structure_comparator::Structure, f64)],
    threshold: f64,
) {
    println!("\n=== CSS Structure Similarity Analysis Results ===");
    
    if similar_rules.is_empty() {
        println!("\nNo similar CSS rules found with threshold >= {threshold}");
        return;
    }
    
    println!("\n## Similar CSS Rules Found: {}", similar_rules.len());
    
    for (i, (rule1, rule2, similarity)) in similar_rules.iter().enumerate() {
        println!(
            "\n{}. {} and {} (similarity: {:.2}%)",
            i + 1,
            rule1.identifier.name,
            rule2.identifier.name,
            similarity * 100.0
        );
        println!(
            "   Files: {} and {}",
            rule1.identifier.namespace.as_deref().unwrap_or("unknown"),
            rule2.identifier.namespace.as_deref().unwrap_or("unknown")
        );
        println!(
            "   Lines: {}-{} and {}-{}",
            rule1.metadata.location.start_line,
            rule1.metadata.location.end_line,
            rule2.metadata.location.start_line,
            rule2.metadata.location.end_line
        );
        println!("   Properties in common: {}", 
            rule1.members.iter()
                .filter(|m1| rule2.members.iter().any(|m2| m1.name == m2.name))
                .count()
        );
    }
    
    println!("\n## Summary");
    println!("Total similar rule pairs found: {}", similar_rules.len());
    println!("Similarity threshold: {threshold}");
}

fn output_structure_vscode(
    similar_rules: &[(similarity_core::structure_comparator::Structure, 
                     similarity_core::structure_comparator::Structure, f64)],
) {
    for (rule1, rule2, similarity) in similar_rules {
        let file1 = rule1.identifier.namespace.as_deref().unwrap_or("unknown");
        let file2 = rule2.identifier.namespace.as_deref().unwrap_or("unknown");
        
        println!(
            "{}:{}:1: warning: Similar to {} ({:.0}% similarity) at {}:{}",
            file1,
            rule1.metadata.location.start_line,
            rule2.identifier.name,
            similarity * 100.0,
            file2,
            rule2.metadata.location.start_line
        );
    }
}

fn output_structure_json(
    similar_rules: &[(similarity_core::structure_comparator::Structure, 
                     similarity_core::structure_comparator::Structure, f64)],
) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::json;
    
    let mut pairs = Vec::new();
    
    for (rule1, rule2, similarity) in similar_rules {
        pairs.push(json!({
            "similarity": similarity,
            "rule1": {
                "selector": rule1.identifier.name,
                "file": rule1.identifier.namespace.as_deref().unwrap_or("unknown"),
                "start_line": rule1.metadata.location.start_line,
                "end_line": rule1.metadata.location.end_line,
                "properties_count": rule1.members.len(),
            },
            "rule2": {
                "selector": rule2.identifier.name,
                "file": rule2.identifier.namespace.as_deref().unwrap_or("unknown"),
                "start_line": rule2.metadata.location.start_line,
                "end_line": rule2.metadata.location.end_line,
                "properties_count": rule2.members.len(),
            }
        }));
    }
    
    let output = json!({
        "similar_rules": pairs,
        "total_pairs": similar_rules.len(),
    });
    
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
