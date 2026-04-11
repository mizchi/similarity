use crate::ast_fingerprint::AstFingerprint;
use crate::compare_functions;
use crate::function_extractor::{extract_functions, FunctionDefinition, SimilarityResult};
use crate::tsed::TSEDOptions;

/// Fast similarity options
#[derive(Debug, Clone)]
pub struct FastSimilarityOptions {
    /// Minimum fingerprint similarity to consider detailed comparison
    pub fingerprint_threshold: f64,
    /// Final similarity threshold
    pub similarity_threshold: f64,
    /// Options for detailed comparison
    pub tsed_options: TSEDOptions,
    /// Enable debug statistics
    pub debug_stats: bool,
}

impl Default for FastSimilarityOptions {
    fn default() -> Self {
        Self {
            fingerprint_threshold: 0.5,
            similarity_threshold: 0.7,
            tsed_options: TSEDOptions::default(),
            debug_stats: false,
        }
    }
}

/// Function with precomputed fingerprint
#[derive(Debug)]
struct FingerprintedFunction {
    function: FunctionDefinition,
    fingerprint: AstFingerprint,
}

/// Find similar functions using fingerprint pre-filtering
pub fn find_similar_functions_fast(
    filename: &str,
    source_text: &str,
    options: &FastSimilarityOptions,
) -> Result<Vec<SimilarityResult>, String> {
    // Extract functions
    let mut functions = extract_functions(filename, source_text)?;
    functions.retain(|function| !function.has_ignore_directive);

    // Create fingerprints
    let mut fingerprinted = Vec::new();
    for func in functions {
        // Skip short functions
        if let Some(min_tokens) = options.tsed_options.min_tokens {
            // If min_tokens is specified, use token count instead of line count
            let tokens = func.node_count.unwrap_or(0);
            if tokens < min_tokens {
                continue;
            }
        } else {
            // Otherwise use line count
            if func.line_count() < options.tsed_options.min_lines {
                continue;
            }
        }

        // Extract function body
        let start = func.body_span.start as usize;
        let end = func.body_span.end as usize;
        let body = &source_text[start..end];

        let fingerprint = match AstFingerprint::from_source(body) {
            Ok(fp) => fp,
            Err(_) => continue, // Skip functions with parse errors
        };
        fingerprinted.push(FingerprintedFunction { function: func, fingerprint });
    }

    let mut similar_pairs = Vec::new();
    let mut comparisons_made = 0;
    let mut comparisons_skipped = 0;

    // Compare all pairs
    for i in 0..fingerprinted.len() {
        for j in (i + 1)..fingerprinted.len() {
            let func1 = &fingerprinted[i];
            let func2 = &fingerprinted[j];

            // Quick fingerprint check
            if !func1
                .fingerprint
                .might_be_similar(&func2.fingerprint, options.fingerprint_threshold)
            {
                comparisons_skipped += 1;
                continue;
            }

            // More detailed fingerprint similarity
            let fp_similarity = func1.fingerprint.similarity(&func2.fingerprint);
            if fp_similarity < options.fingerprint_threshold {
                comparisons_skipped += 1;
                continue;
            }

            // Full comparison
            comparisons_made += 1;
            let similarity = compare_functions(
                &func1.function,
                &func2.function,
                source_text,
                source_text,
                &options.tsed_options,
            )?;

            if similarity >= options.similarity_threshold {
                similar_pairs.push(SimilarityResult::new(
                    func1.function.clone(),
                    func2.function.clone(),
                    similarity,
                ));
            }
        }
    }

    if options.debug_stats {
        let total = comparisons_made + comparisons_skipped;
        if total > 0 {
            let skip_rate = (comparisons_skipped as f64 / total as f64) * 100.0;
            eprintln!(
                "Fast comparison: {comparisons_made} detailed, {comparisons_skipped} skipped ({skip_rate:.1}% skip rate)"
            );
        }
    }

    // Sort by priority
    similar_pairs.sort_by(|a, b| {
        b.impact
            .cmp(&a.impact)
            .then(b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal))
    });

    Ok(similar_pairs)
}

/// Find similar functions across multiple files using fingerprint pre-filtering
pub fn find_similar_functions_across_files_fast(
    files: &[(String, String)],
    options: &FastSimilarityOptions,
) -> Result<Vec<(String, SimilarityResult, String)>, String> {
    let mut all_functions = Vec::new();

    // Extract functions with fingerprints from all files
    for (filename, source) in files {
        let mut functions = extract_functions(filename, source)?;
        functions.retain(|function| !function.has_ignore_directive);
        for func in functions {
            if let Some(min_tokens) = options.tsed_options.min_tokens {
                // If min_tokens is specified, use token count instead of line count
                let tokens = func.node_count.unwrap_or(0);
                if tokens < min_tokens {
                    continue;
                }
            } else {
                // Otherwise use line count
                if func.line_count() < options.tsed_options.min_lines {
                    continue;
                }
            }

            let start = func.body_span.start as usize;
            let end = func.body_span.end as usize;
            let body = &source[start..end];
            let fingerprint = match AstFingerprint::from_source(body) {
                Ok(fp) => fp,
                Err(_) => continue, // Skip functions with parse errors
            };

            all_functions.push((
                filename.clone(),
                source.clone(),
                FingerprintedFunction { function: func, fingerprint },
            ));
        }
    }

    let mut similar_pairs = Vec::new();
    let mut comparisons_made = 0;
    let mut comparisons_skipped = 0;

    // Compare all pairs across files
    for i in 0..all_functions.len() {
        for j in (i + 1)..all_functions.len() {
            let (file1, source1, func1) = &all_functions[i];
            let (file2, source2, func2) = &all_functions[j];

            // Skip same file
            if file1 == file2 {
                continue;
            }

            // Quick fingerprint check
            if !func1
                .fingerprint
                .might_be_similar(&func2.fingerprint, options.fingerprint_threshold)
            {
                comparisons_skipped += 1;
                continue;
            }

            // Detailed fingerprint similarity
            let fp_similarity = func1.fingerprint.similarity(&func2.fingerprint);
            if fp_similarity < options.fingerprint_threshold {
                comparisons_skipped += 1;
                continue;
            }

            // Full comparison
            comparisons_made += 1;
            let similarity = compare_functions(
                &func1.function,
                &func2.function,
                source1,
                source2,
                &options.tsed_options,
            )?;

            if similarity >= options.similarity_threshold {
                similar_pairs.push((
                    file1.clone(),
                    SimilarityResult::new(
                        func1.function.clone(),
                        func2.function.clone(),
                        similarity,
                    ),
                    file2.clone(),
                ));
            }
        }
    }

    if options.debug_stats {
        let total = comparisons_made + comparisons_skipped;
        if total > 0 {
            let skip_rate = (comparisons_skipped as f64 / total as f64) * 100.0;
            eprintln!(
                "Fast cross-file comparison: {comparisons_made} detailed, {comparisons_skipped} skipped ({skip_rate:.1}% skip rate)"
            );
        }
    }

    // Sort by priority
    similar_pairs.sort_by(|(_, a, _), (_, b, _)| {
        b.impact
            .cmp(&a.impact)
            .then(b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal))
    });

    Ok(similar_pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_similarity() {
        let code = r#"
            export function add(a: number, b: number): number {
                const result = a + b;
                return result;
            }
            
            export function sum(x: number, y: number): number {
                const result = x + y;
                return result;
            }
            
            export function multiply(a: number, b: number): number {
                const result = a * b;
                return result;
            }
        "#;

        let options = FastSimilarityOptions {
            fingerprint_threshold: 0.2,
            similarity_threshold: 0.25,
            tsed_options: TSEDOptions {
                min_lines: 1,
                size_penalty: false, // Disable for test with small functions
                ..Default::default()
            },
            debug_stats: true,
        };

        let result = find_similar_functions_fast("test.ts", code, &options);
        assert!(result.is_ok());

        let pairs = result.unwrap();

        // Print debug info to see what's happening
        if pairs.is_empty() {
            // Try to get all comparisons regardless of threshold
            let debug_options = FastSimilarityOptions {
                fingerprint_threshold: 0.0,
                similarity_threshold: 0.0,
                tsed_options: TSEDOptions { min_lines: 1, ..Default::default() },
                debug_stats: false,
            };
            let debug_result =
                find_similar_functions_fast("test.ts", code, &debug_options).unwrap();
            for pair in &debug_result {
                println!(
                    "{} ~ {}: {:.2}%",
                    pair.func1.name,
                    pair.func2.name,
                    pair.similarity * 100.0
                );
            }
        }

        assert!(!pairs.is_empty(), "No similar pairs found");

        // add and sum should be found as similar
        let found_add_sum = pairs.iter().any(|p| {
            (p.func1.name == "add" && p.func2.name == "sum")
                || (p.func1.name == "sum" && p.func2.name == "add")
        });
        assert!(found_add_sum);
    }

    #[test]
    fn test_identical_functions_always_pass_fingerprint() {
        // Test that identical functions always pass the fingerprint check
        let code1 = "function test(a: number, b: number): number { return a + b; }";
        let code2 = "function test(a: number, b: number): number { return a + b; }";

        let fp1 = AstFingerprint::from_source(code1).expect("Failed to create fingerprint");
        let fp2 = AstFingerprint::from_source(code2).expect("Failed to create fingerprint");

        // Should always pass fingerprint check for any threshold
        assert!(fp1.might_be_similar(&fp2, 0.9));
        assert!(fp1.might_be_similar(&fp2, 0.5));
        assert!(fp1.might_be_similar(&fp2, 0.1));

        // Similarity should be 100%
        assert_eq!(fp1.similarity(&fp2), 1.0);
    }
}
