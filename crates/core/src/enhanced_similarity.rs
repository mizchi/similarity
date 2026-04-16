use crate::apted::{compute_edit_distance, APTEDOptions};
use crate::tree::TreeNode;
use std::rc::Rc;

/// Enhanced similarity calculation that considers multiple factors
pub struct EnhancedSimilarityOptions {
    /// Weight for structural similarity (0.0-1.0)
    pub structural_weight: f64,
    /// Weight for size similarity (0.0-1.0)
    pub size_weight: f64,
    /// Weight for node type distribution (0.0-1.0)
    pub type_distribution_weight: f64,
    /// Minimum size ratio to avoid penalizing small vs large functions
    pub min_size_ratio: f64,
    /// APTED options
    pub apted_options: APTEDOptions,
}

impl Default for EnhancedSimilarityOptions {
    fn default() -> Self {
        Self {
            structural_weight: 0.4,
            size_weight: 0.3,
            type_distribution_weight: 0.3,
            min_size_ratio: 0.5,
            apted_options: APTEDOptions::default(),
        }
    }
}

/// Calculate enhanced similarity between two trees
pub fn calculate_enhanced_similarity(
    tree1: &Rc<TreeNode>,
    tree2: &Rc<TreeNode>,
    options: &EnhancedSimilarityOptions,
) -> f64 {
    // 1. Structural similarity using APTED
    let distance = compute_edit_distance(tree1, tree2, &options.apted_options);
    let max_size = tree1.get_subtree_size().max(tree2.get_subtree_size()) as f64;
    let structural_similarity = if max_size > 0.0 { 1.0 - (distance / max_size) } else { 1.0 };

    // 2. Size similarity
    let size1 = tree1.get_subtree_size() as f64;
    let size2 = tree2.get_subtree_size() as f64;
    let size_ratio = size1.min(size2) / size1.max(size2).max(1.0);
    let size_similarity = if size_ratio < options.min_size_ratio {
        size_ratio / options.min_size_ratio // Penalize very different sizes
    } else {
        1.0
    };

    // 3. Node type distribution similarity
    let dist1 = get_node_type_distribution(tree1);
    let dist2 = get_node_type_distribution(tree2);
    let type_similarity = calculate_distribution_similarity(&dist1, &dist2);

    // 4. Apply additional penalties for very different trees
    let mut penalty_factor = 1.0;

    // Penalize if one tree is more than 2x the size of the other
    if size_ratio < 0.5 {
        penalty_factor *= 0.8;
    }

    // Penalize if structural similarity is very high but semantic similarity is very low
    let semantic_sim = calculate_semantic_similarity(tree1, tree2);
    if structural_similarity > 0.7 && semantic_sim < 0.2 {
        penalty_factor *= 0.7;
    }

    // Penalize if the trees have very different complexity
    let complexity_ratio = calculate_complexity_ratio(tree1, tree2);
    if complexity_ratio < 0.5 {
        penalty_factor *= 0.85;
    }

    // Weighted combination
    let total_weight =
        options.structural_weight + options.size_weight + options.type_distribution_weight;
    let combined_similarity = (structural_similarity * options.structural_weight
        + size_similarity * options.size_weight
        + type_similarity * options.type_distribution_weight)
        / total_weight;

    combined_similarity * penalty_factor
}

/// Get distribution of node types in a tree
fn get_node_type_distribution(tree: &TreeNode) -> std::collections::HashMap<String, usize> {
    let mut distribution = std::collections::HashMap::new();
    count_node_types(tree, &mut distribution);
    distribution
}

fn count_node_types(node: &TreeNode, distribution: &mut std::collections::HashMap<String, usize>) {
    *distribution.entry(node.label.clone()).or_insert(0) += 1;

    for child in &node.children {
        count_node_types(child, distribution);
    }
}

/// Calculate similarity between two node type distributions
fn calculate_distribution_similarity(
    dist1: &std::collections::HashMap<String, usize>,
    dist2: &std::collections::HashMap<String, usize>,
) -> f64 {
    let all_types: std::collections::HashSet<_> =
        dist1.keys().chain(dist2.keys()).cloned().collect();

    if all_types.is_empty() {
        return 1.0;
    }

    let mut intersection = 0;
    let mut union = 0;

    for node_type in all_types {
        let count1 = dist1.get(&node_type).copied().unwrap_or(0);
        let count2 = dist2.get(&node_type).copied().unwrap_or(0);

        intersection += count1.min(count2);
        union += count1.max(count2);
    }

    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Calculate semantic similarity based on key nodes
pub fn calculate_semantic_similarity(tree1: &TreeNode, tree2: &TreeNode) -> f64 {
    // Extract key semantic features
    let features1 = extract_semantic_features(tree1);
    let features2 = extract_semantic_features(tree2);

    // Compare features
    let mut matches = 0;
    let mut total = 0;

    // Compare identifiers
    let common_identifiers = features1.identifiers.intersection(&features2.identifiers).count();
    let total_identifiers = features1.identifiers.union(&features2.identifiers).count();
    if total_identifiers > 0 {
        matches += common_identifiers;
        total += total_identifiers;
    }

    // Compare operators
    let common_operators = features1.operators.intersection(&features2.operators).count();
    let total_operators = features1.operators.union(&features2.operators).count();
    if total_operators > 0 {
        matches += common_operators;
        total += total_operators;
    }

    // Compare control flow
    let common_control = features1.control_flow.intersection(&features2.control_flow).count();
    let total_control = features1.control_flow.union(&features2.control_flow).count();
    if total_control > 0 {
        matches += common_control;
        total += total_control;
    }

    if total == 0 {
        0.0 // No semantic features to compare
    } else {
        matches as f64 / total as f64
    }
}

#[derive(Debug)]
struct SemanticFeatures {
    identifiers: std::collections::HashSet<String>,
    operators: std::collections::HashSet<String>,
    control_flow: std::collections::HashSet<String>,
    function_calls: std::collections::HashSet<String>,
}

fn extract_semantic_features(tree: &TreeNode) -> SemanticFeatures {
    let mut features = SemanticFeatures {
        identifiers: std::collections::HashSet::new(),
        operators: std::collections::HashSet::new(),
        control_flow: std::collections::HashSet::new(),
        function_calls: std::collections::HashSet::new(),
    };

    extract_features_recursive(tree, &mut features);
    features
}

/// Calculate complexity ratio between two trees
fn calculate_complexity_ratio(tree1: &TreeNode, tree2: &TreeNode) -> f64 {
    let complexity1 = calculate_tree_complexity(tree1);
    let complexity2 = calculate_tree_complexity(tree2);

    if complexity1 == 0 && complexity2 == 0 {
        return 1.0;
    }

    let min_complexity = complexity1.min(complexity2) as f64;
    let max_complexity = complexity1.max(complexity2) as f64;

    min_complexity / max_complexity
}

/// Calculate complexity of a tree based on depth and branching
fn calculate_tree_complexity(tree: &TreeNode) -> usize {
    calculate_complexity_recursive(tree, 0)
}

fn calculate_complexity_recursive(node: &TreeNode, depth: usize) -> usize {
    let mut complexity = depth + 1; // Depth contributes to complexity

    // Control flow nodes contribute more to complexity
    match node.label.as_str() {
        "if_expression" | "if_statement" | "loop_expression" | "while_expression"
        | "for_expression" | "match_expression" => {
            complexity += 3;
        }
        "function_item" | "closure_expression" => {
            complexity += 2;
        }
        _ => {}
    }

    for child in &node.children {
        complexity += calculate_complexity_recursive(child, depth + 1);
    }

    complexity
}

fn extract_features_recursive(node: &TreeNode, features: &mut SemanticFeatures) {
    match node.label.as_str() {
        "identifier" if !node.value.is_empty() => {
            features.identifiers.insert(node.value.clone());
        }
        "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||" | "!"
        | "&" | "|" | "^" | "<<" | ">>" => {
            features.operators.insert(node.label.clone());
        }
        "if_expression" | "if_statement" => {
            features.control_flow.insert("if".to_string());
        }
        "loop_expression" | "while_expression" | "for_expression" => {
            features.control_flow.insert("loop".to_string());
        }
        "return_expression" | "return_statement" => {
            features.control_flow.insert("return".to_string());
        }
        "call_expression" => {
            features.control_flow.insert("call".to_string());
            // Try to extract function name if available
            if let Some(first_child) = node.children.first() {
                if first_child.label == "identifier" && !first_child.value.is_empty() {
                    features.function_calls.insert(first_child.value.clone());
                }
            }
        }
        _ => {}
    }

    for child in &node.children {
        extract_features_recursive(child, features);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_similarity() {
        let options = EnhancedSimilarityOptions::default();

        // Create two trees of different sizes
        let mut tree1 = TreeNode::new("root".to_string(), "".to_string(), 1);
        tree1.add_child(Rc::new(TreeNode::new("child".to_string(), "".to_string(), 2)));

        let mut tree2 = TreeNode::new("root".to_string(), "".to_string(), 3);
        for i in 0..10 {
            tree2.add_child(Rc::new(TreeNode::new("child".to_string(), "".to_string(), 4 + i)));
        }

        let tree1_rc = Rc::new(tree1);
        let tree2_rc = Rc::new(tree2);

        let similarity = calculate_enhanced_similarity(&tree1_rc, &tree2_rc, &options);

        // Should have low similarity due to size difference
        assert!(
            similarity < 0.5,
            "Expected low similarity due to size difference, got {}",
            similarity
        );
    }
}
