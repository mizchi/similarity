use crate::apted::{compute_edit_distance, compute_edit_distance_with_cutoff, APTEDOptions};
use crate::tree::TreeNode;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct TSEDOptions {
    pub apted_options: APTEDOptions,
    pub min_lines: u32, // Minimum number of lines for a function to be considered
    pub min_tokens: Option<u32>, // Minimum number of tokens (AST nodes) for a function to be considered
    pub size_penalty: bool,      // Apply penalty for short functions
    pub skip_test: bool,         // Skip test functions (language-specific)
}

impl Default for TSEDOptions {
    fn default() -> Self {
        TSEDOptions {
            apted_options: APTEDOptions {
                rename_cost: 0.3, // Default from the TypeScript implementation
                delete_cost: 1.0,
                insert_cost: 1.0,
                compare_values: false, // TypeScript default: structural comparison only
            },
            min_lines: 5,       // Increased default to better filter trivial matches
            min_tokens: None,   // No token limit by default
            size_penalty: true, // Enable size penalty by default
            skip_test: false,   // Don't skip test functions by default
        }
    }
}

/// Calculate TSED (Tree Structure Edit Distance) similarity between two trees
/// Returns a value between 0.0 and 1.0, where 1.0 means identical
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn calculate_tsed(tree1: &Rc<TreeNode>, tree2: &Rc<TreeNode>, options: &TSEDOptions) -> f64 {
    let distance = compute_edit_distance(tree1, tree2, &options.apted_options);

    let size1 = tree1.get_subtree_size() as f64;
    let size2 = tree2.get_subtree_size() as f64;

    // TSED normalization: Use the larger tree size
    // This ensures that when comparing trees of different sizes,
    // the similarity reflects how much of the larger tree matches
    let max_size = size1.max(size2);

    // Calculate base TSED similarity
    let tsed_similarity = if max_size > 0.0 { (1.0 - distance / max_size).max(0.0) } else { 1.0 };

    // If distance is 0 but trees have different sizes, check more carefully
    // This can happen when compare_values is false and structure is similar
    let tsed_similarity = if distance == 0.0 && size1 != size2 {
        let size_ratio = size1.min(size2) / size1.max(size2);
        let size_diff = (size1 - size2).abs();

        // Apply penalty based on both ratio and absolute difference
        if size_diff > 10.0 {
            // Strong penalty for large absolute differences
            tsed_similarity * 0.5
        } else if size_ratio < 0.95 || size_diff > 3.0 {
            // Moderate penalty for noticeable differences
            tsed_similarity * size_ratio.powf(0.5)
        } else {
            tsed_similarity // Very minor differences are OK
        }
    } else {
        tsed_similarity
    };

    // For very small trees, even small differences should matter more
    let tsed_similarity = if options.size_penalty {
        if max_size < 10.0 && distance > 0.0 {
            tsed_similarity * 0.8 // Reduce similarity for small trees with any differences
        } else if max_size < 30.0 && distance > 0.0 {
            // For moderately small trees, apply a smaller penalty
            tsed_similarity * 0.9
        } else {
            tsed_similarity
        }
    } else {
        tsed_similarity
    };

    // Apply additional penalties for structural differences
    let mut similarity = tsed_similarity;

    // Size ratio penalty: penalize when trees have very different sizes
    let size_ratio = size1.min(size2) / size1.max(size2);

    if options.size_penalty {
        // For short functions, make differences more pronounced
        let min_size = size1.min(size2);

        if min_size < 30.0 {
            // Short function penalty: the shorter, the more sensitive to differences
            let short_function_factor = (min_size / 30.0).powf(0.5);
            similarity *= short_function_factor;

            // Additional penalty for very short functions
            if min_size < 10.0 {
                similarity *= 0.5; // Strong penalty for very short functions
            } else if min_size < 20.0 {
                similarity *= 0.7; // Moderate penalty for short functions
            }
        }

        // Size difference penalty
        if size_ratio < 0.5 {
            // If one tree is less than half the size of the other,
            // they're likely fundamentally different
            similarity *= size_ratio.powf(0.5);
        }
    }

    similarity
}

/// Calculate TSED with early termination when the result cannot reach the threshold.
/// Returns 0.0 immediately if the distance exceeds the budget, avoiding full computation.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn calculate_tsed_with_threshold(
    tree1: &Rc<TreeNode>,
    tree2: &Rc<TreeNode>,
    options: &TSEDOptions,
    threshold: f64,
) -> f64 {
    let size1 = tree1.get_subtree_size() as f64;
    let size2 = tree2.get_subtree_size() as f64;
    let max_size = size1.max(size2);

    if max_size == 0.0 {
        return 1.0;
    }

    // Maximum distance that could still yield similarity >= threshold
    // similarity = 1.0 - distance / max_size >= threshold
    // => distance <= max_size * (1.0 - threshold)
    let max_distance = max_size * (1.0 - threshold);

    let distance =
        compute_edit_distance_with_cutoff(tree1, tree2, &options.apted_options, max_distance);

    // If distance exceeded cutoff, similarity is definitely below threshold
    if distance >= f64::MAX / 2.0 {
        return 0.0;
    }

    // Apply the same normalization and penalties as calculate_tsed
    let tsed_similarity = (1.0 - distance / max_size).max(0.0);

    let tsed_similarity = if distance == 0.0 && size1 != size2 {
        let size_ratio = size1.min(size2) / size1.max(size2);
        let size_diff = (size1 - size2).abs();

        if size_diff > 10.0 {
            tsed_similarity * 0.5
        } else if size_ratio < 0.95 || size_diff > 3.0 {
            tsed_similarity * size_ratio.powf(0.5)
        } else {
            tsed_similarity
        }
    } else {
        tsed_similarity
    };

    let tsed_similarity = if options.size_penalty {
        if max_size < 10.0 && distance > 0.0 {
            tsed_similarity * 0.8
        } else if max_size < 30.0 && distance > 0.0 {
            tsed_similarity * 0.9
        } else {
            tsed_similarity
        }
    } else {
        tsed_similarity
    };

    let mut similarity = tsed_similarity;
    let size_ratio = size1.min(size2) / size1.max(size2);

    if options.size_penalty {
        let min_size = size1.min(size2);

        if min_size < 30.0 {
            let short_function_factor = (min_size / 30.0).powf(0.5);
            similarity *= short_function_factor;

            if min_size < 10.0 {
                similarity *= 0.5;
            } else if min_size < 20.0 {
                similarity *= 0.7;
            }
        }

        if size_ratio < 0.5 {
            similarity *= size_ratio.powf(0.5);
        }
    }

    similarity
}

/// Calculate TSED from TypeScript code strings
///
/// # Errors
///
/// Returns an error if parsing fails for either code string
pub fn calculate_tsed_from_code(
    code1: &str,
    code2: &str,
    filename1: &str,
    filename2: &str,
    options: &TSEDOptions,
) -> Result<f64, String> {
    use crate::parser::parse_and_convert_to_tree;

    let tree1 = parse_and_convert_to_tree(filename1, code1)?;
    let tree2 = parse_and_convert_to_tree(filename2, code2)?;

    Ok(calculate_tsed(&tree1, &tree2, options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_code() {
        let code = "function add(a: number, b: number) { return a + b; }";
        let options = TSEDOptions {
            size_penalty: false, // Disable for small test functions
            ..Default::default()
        };

        let similarity =
            calculate_tsed_from_code(code, code, "test1.ts", "test2.ts", &options).unwrap();
        assert!((similarity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_renamed_function() {
        let code1 = "function add(a: number, b: number) { return a + b; }";
        let code2 = "function sum(x: number, y: number) { return x + y; }";
        let options = TSEDOptions {
            size_penalty: false, // Disable for small test functions
            ..Default::default()
        };

        let similarity =
            calculate_tsed_from_code(code1, code2, "test1.ts", "test2.ts", &options).unwrap();
        // Should have high similarity due to low rename cost
        assert!(similarity > 0.8);
    }

    #[test]
    fn test_different_structure() {
        let code1 = "function test() { return 1; }";
        let code2 = "class Test { method() { return 1; } }";
        let options = TSEDOptions::default();

        let similarity =
            calculate_tsed_from_code(code1, code2, "test1.ts", "test2.ts", &options).unwrap();
        // Should have lower similarity due to structural differences
        assert!(similarity < 0.7);
    }
}
