use crate::tree::TreeNode;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct APTEDOptions {
    pub rename_cost: f64,
    pub delete_cost: f64,
    pub insert_cost: f64,
    /// Whether to compare node values in addition to labels
    pub compare_values: bool,
}

impl Default for APTEDOptions {
    fn default() -> Self {
        APTEDOptions {
            rename_cost: 1.0,
            delete_cost: 1.0,
            insert_cost: 1.0,
            compare_values: true, // Default: compare both structure and values
        }
    }
}

/// Sentinel value indicating the distance exceeds the cutoff budget.
const DISTANCE_EXCEEDED: f64 = f64::MAX;

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn compute_edit_distance(
    tree1: &Rc<TreeNode>,
    tree2: &Rc<TreeNode>,
    options: &APTEDOptions,
) -> f64 {
    let mut memo: HashMap<(usize, usize), f64> = HashMap::new();
    compute_edit_distance_recursive(tree1, tree2, options, &mut memo)
}

/// Compute edit distance with early termination.
/// Returns `DISTANCE_EXCEEDED` if the distance exceeds `max_distance`,
/// otherwise returns the exact distance.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn compute_edit_distance_with_cutoff(
    tree1: &Rc<TreeNode>,
    tree2: &Rc<TreeNode>,
    options: &APTEDOptions,
    max_distance: f64,
) -> f64 {
    let mut memo: HashMap<(usize, usize), f64> = HashMap::new();
    compute_edit_distance_cutoff(tree1, tree2, options, &mut memo, max_distance)
}

fn node_rename_cost(node1: &TreeNode, node2: &TreeNode, options: &APTEDOptions) -> f64 {
    if options.compare_values {
        if node1.label == node2.label && node1.value == node2.value {
            0.0
        } else {
            options.rename_cost
        }
    } else if node1.label == node2.label {
        0.0
    } else {
        options.rename_cost
    }
}

fn compute_edit_distance_recursive(
    node1: &Rc<TreeNode>,
    node2: &Rc<TreeNode>,
    options: &APTEDOptions,
    memo: &mut HashMap<(usize, usize), f64>,
) -> f64 {
    let key = (node1.id, node2.id);

    if let Some(&cost) = memo.get(&key) {
        return cost;
    }

    // Base cases
    if node1.children.is_empty() && node2.children.is_empty() {
        let cost = node_rename_cost(node1, node2, options);
        memo.insert(key, cost);
        return cost;
    }

    // Calculate costs for all three operations
    let delete_all_cost = options.delete_cost * node1.get_subtree_size() as f64;
    let insert_all_cost = options.insert_cost * node2.get_subtree_size() as f64;

    // Calculate rename + optimal children alignment
    let mut rename_plus_cost = node_rename_cost(node1, node2, options);

    if !node1.children.is_empty() || !node2.children.is_empty() {
        // Compute all pairwise costs between children
        let mut child_cost_matrix: HashMap<(usize, usize), f64> = HashMap::new();

        for child1 in &node1.children {
            for child2 in &node2.children {
                let cost = compute_edit_distance_recursive(child1, child2, options, memo);
                child_cost_matrix.insert((child1.id, child2.id), cost);
            }
        }

        // Find optimal alignment
        let (alignment_cost, _) = compute_children_alignment(
            &node1.children,
            &node2.children,
            &child_cost_matrix,
            options,
        );

        rename_plus_cost += alignment_cost;
    }

    let min_cost = delete_all_cost.min(insert_all_cost).min(rename_plus_cost);
    memo.insert(key, min_cost);
    min_cost
}

#[allow(clippy::cast_precision_loss)]
fn compute_edit_distance_cutoff(
    node1: &Rc<TreeNode>,
    node2: &Rc<TreeNode>,
    options: &APTEDOptions,
    memo: &mut HashMap<(usize, usize), f64>,
    max_distance: f64,
) -> f64 {
    let key = (node1.id, node2.id);

    if let Some(&cost) = memo.get(&key) {
        return cost;
    }

    // Base cases
    if node1.children.is_empty() && node2.children.is_empty() {
        let cost = node_rename_cost(node1, node2, options);
        memo.insert(key, cost);
        return cost;
    }

    let size1 = node1.get_subtree_size() as f64;
    let size2 = node2.get_subtree_size() as f64;

    // Lower bound: at minimum, we need to insert/delete the size difference
    let min_op_cost = options.delete_cost.min(options.insert_cost);
    let lower_bound = (size1 - size2).abs() * min_op_cost;
    if lower_bound > max_distance {
        // Don't cache exceeded values - they depend on the budget
        return DISTANCE_EXCEEDED;
    }

    // Calculate costs for delete-all and insert-all operations
    let delete_all_cost = options.delete_cost * size1;
    let insert_all_cost = options.insert_cost * size2;
    let mut best = delete_all_cost.min(insert_all_cost);

    // Try rename + optimal children alignment
    let rename_cost = node_rename_cost(node1, node2, options);

    // Only compute alignment if it could improve on best
    // (alignment_cost >= 0, so rename_cost alone is the lower bound for this path)
    if rename_cost < best && (!node1.children.is_empty() || !node2.children.is_empty()) {
        // Budget for alignment = best - rename_cost (anything beyond that won't improve)
        let alignment_budget = best - rename_cost;

        // Compute pairwise child costs with tightened budget
        let mut child_cost_matrix: HashMap<(usize, usize), f64> = HashMap::new();

        for child1 in &node1.children {
            for child2 in &node2.children {
                let cost =
                    compute_edit_distance_cutoff(child1, child2, options, memo, alignment_budget);
                child_cost_matrix.insert((child1.id, child2.id), cost);
            }
        }

        // Find optimal alignment with budget
        let alignment_cost = compute_children_alignment_with_cutoff(
            &node1.children,
            &node2.children,
            &child_cost_matrix,
            options,
            alignment_budget,
        );

        if alignment_cost < DISTANCE_EXCEEDED {
            let rename_plus_cost = rename_cost + alignment_cost;
            best = best.min(rename_plus_cost);
        }
    }

    if best > max_distance {
        return DISTANCE_EXCEEDED;
    }

    memo.insert(key, best);
    best
}

fn compute_children_alignment(
    children1: &[Rc<TreeNode>],
    children2: &[Rc<TreeNode>],
    cost_matrix: &HashMap<(usize, usize), f64>,
    options: &APTEDOptions,
) -> (f64, HashMap<usize, Option<usize>>) {
    let m = children1.len();
    let n = children2.len();

    // dp[i][j] = minimum cost to align first i children of node1 with first j children of node2
    let mut dp = vec![vec![0.0; n + 1]; m + 1];

    // Initialize base cases
    for i in 1..=m {
        dp[i][0] = dp[i - 1][0] + options.delete_cost * children1[i - 1].get_subtree_size() as f64;
    }
    for j in 1..=n {
        dp[0][j] = dp[0][j - 1] + options.insert_cost * children2[j - 1].get_subtree_size() as f64;
    }

    // Fill DP table
    for i in 1..=m {
        for j in 1..=n {
            let child1 = &children1[i - 1];
            let child2 = &children2[j - 1];
            let edit_cost = cost_matrix.get(&(child1.id, child2.id)).unwrap_or(&0.0);

            dp[i][j] = (dp[i - 1][j] + options.delete_cost * child1.get_subtree_size() as f64)
                .min(dp[i][j - 1] + options.insert_cost * child2.get_subtree_size() as f64)
                .min(dp[i - 1][j - 1] + edit_cost);
        }
    }

    // Backtrack to find alignment
    let mut alignment = HashMap::new();
    let mut i = m;
    let mut j = n;

    while i > 0 || j > 0 {
        if i == 0 {
            j -= 1;
        } else if j == 0 {
            alignment.insert(children1[i - 1].id, None);
            i -= 1;
        } else {
            let child1 = &children1[i - 1];
            let child2 = &children2[j - 1];
            let edit_cost = cost_matrix.get(&(child1.id, child2.id)).unwrap_or(&0.0);

            let delete_cost = dp[i - 1][j] + options.delete_cost * child1.get_subtree_size() as f64;
            let insert_cost = dp[i][j - 1] + options.insert_cost * child2.get_subtree_size() as f64;
            let match_cost = dp[i - 1][j - 1] + edit_cost;

            if match_cost <= delete_cost && match_cost <= insert_cost {
                alignment.insert(child1.id, Some(child2.id));
                i -= 1;
                j -= 1;
            } else if delete_cost <= insert_cost {
                alignment.insert(child1.id, None);
                i -= 1;
            } else {
                j -= 1;
            }
        }
    }

    (dp[m][n], alignment)
}

/// Children alignment with early termination when cost exceeds budget.
#[allow(clippy::cast_precision_loss)]
fn compute_children_alignment_with_cutoff(
    children1: &[Rc<TreeNode>],
    children2: &[Rc<TreeNode>],
    cost_matrix: &HashMap<(usize, usize), f64>,
    options: &APTEDOptions,
    max_cost: f64,
) -> f64 {
    let m = children1.len();
    let n = children2.len();

    let mut dp = vec![vec![0.0; n + 1]; m + 1];

    for i in 1..=m {
        dp[i][0] = dp[i - 1][0] + options.delete_cost * children1[i - 1].get_subtree_size() as f64;
    }
    for j in 1..=n {
        dp[0][j] = dp[0][j - 1] + options.insert_cost * children2[j - 1].get_subtree_size() as f64;
    }

    for i in 1..=m {
        // Track the minimum value in this row for early termination
        let mut row_min = f64::MAX;
        for j in 1..=n {
            let child1 = &children1[i - 1];
            let child2 = &children2[j - 1];
            let edit_cost = cost_matrix.get(&(child1.id, child2.id)).copied().unwrap_or(0.0);

            // If edit_cost is DISTANCE_EXCEEDED, use delete+insert as fallback
            let edit_cost = if edit_cost >= DISTANCE_EXCEEDED {
                options.delete_cost * child1.get_subtree_size() as f64
                    + options.insert_cost * child2.get_subtree_size() as f64
            } else {
                edit_cost
            };

            dp[i][j] = (dp[i - 1][j] + options.delete_cost * child1.get_subtree_size() as f64)
                .min(dp[i][j - 1] + options.insert_cost * child2.get_subtree_size() as f64)
                .min(dp[i - 1][j - 1] + edit_cost);

            row_min = row_min.min(dp[i][j]);
        }

        // If the minimum in this row already exceeds the budget,
        // subsequent rows can only grow, so early terminate
        if row_min > max_cost {
            return DISTANCE_EXCEEDED;
        }
    }

    dp[m][n]
}
