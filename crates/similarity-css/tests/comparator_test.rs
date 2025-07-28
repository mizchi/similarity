use similarity_core::tree::TreeNode;
use similarity_css::css_comparator::{CssRule, compare_css_rules};
use std::rc::Rc;

fn create_test_rule(selector: &str, declarations: Vec<(&str, &str)>) -> CssRule {
    let mut children = Vec::new();
    for (prop, val) in &declarations {
        let decl = TreeNode::new(
            "declaration".to_string(),
            format!("{}: {}", prop, val),
            0
        );
        children.push(Rc::new(decl));
    }
    
    let mut tree = TreeNode::new(
        "rule".to_string(),
        selector.to_string(),
        0
    );
    tree.children = children;
    
    CssRule {
        selector: selector.to_string(),
        declarations: declarations.into_iter()
            .map(|(p, v)| (p.to_string(), v.to_string()))
            .collect(),
        tree: Rc::new(tree),
        start_line: 1,
        end_line: 10,
    }
}

#[test]
fn test_identical_rules() {
    let rule1 = create_test_rule(".button", vec![
        ("background-color", "blue"),
        ("color", "white"),
        ("padding", "10px"),
    ]);
    
    let rule2 = create_test_rule(".button", vec![
        ("background-color", "blue"),
        ("color", "white"),
        ("padding", "10px"),
    ]);
    
    let results = compare_css_rules(&[rule1], &[rule2], 0.8);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].similarity, 1.0);
}

#[test]
fn test_similar_rules_different_selectors() {
    let rule1 = create_test_rule(".button", vec![
        ("background-color", "blue"),
        ("color", "white"),
        ("padding", "10px"),
    ]);
    
    let rule2 = create_test_rule(".btn", vec![
        ("background-color", "blue"),
        ("color", "white"),
        ("padding", "10px"),
    ]);
    
    let results = compare_css_rules(&[rule1], &[rule2], 0.3);
    assert_eq!(results.len(), 1);
    assert!(results[0].similarity > 0.3);
    assert!(results[0].similarity < 1.0);
}

#[test]
fn test_similar_values() {
    let rule1 = create_test_rule(".button", vec![
        ("color", "white"),
        ("padding", "10px 20px"),
    ]);
    
    let rule2 = create_test_rule(".button", vec![
        ("color", "#fff"),
        ("padding", "10px 20px"),
    ]);
    
    let results = compare_css_rules(&[rule1], &[rule2], 0.8);
    assert_eq!(results.len(), 1);
    assert!(results[0].similarity > 0.9);
}

#[test]
fn test_different_rules() {
    let rule1 = create_test_rule(".button", vec![
        ("background-color", "blue"),
        ("color", "white"),
    ]);
    
    let rule2 = create_test_rule(".header", vec![
        ("display", "flex"),
        ("justify-content", "center"),
    ]);
    
    let results = compare_css_rules(&[rule1], &[rule2], 0.8);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_threshold_filtering() {
    let rule1 = create_test_rule(".button", vec![
        ("background", "blue"),
        ("color", "white"),
        ("padding", "10px"),
    ]);
    
    let rule2 = create_test_rule(".btn", vec![
        ("background", "blue"),
        ("color", "white"),
    ]);
    
    let high_results = compare_css_rules(&[rule1.clone()], &[rule2.clone()], 0.95);
    let low_results = compare_css_rules(&[rule1], &[rule2], 0.1);
    
    assert_eq!(high_results.len(), 0);
    assert_eq!(low_results.len(), 1);
}