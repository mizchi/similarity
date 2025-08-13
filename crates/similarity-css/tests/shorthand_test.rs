use similarity_core::tree::TreeNode;
use similarity_css::{compare_css_rules, CssRule};
use std::rc::Rc;

fn create_test_rule_with_declarations(selector: &str, declarations: Vec<(&str, &str)>) -> CssRule {
    let mut children = Vec::new();
    let decls: Vec<(String, String)> =
        declarations.iter().map(|(p, v)| (p.to_string(), v.to_string())).collect();

    for (prop, val) in &decls {
        let decl = TreeNode::new("declaration".to_string(), format!("{prop}: {val}"), 0);
        children.push(Rc::new(decl));
    }

    let mut tree = TreeNode::new("rule".to_string(), selector.to_string(), 0);
    tree.children = children;

    CssRule {
        selector: selector.to_string(),
        declarations: decls,
        tree: Rc::new(tree),
        start_line: 1,
        end_line: 1,
    }
}

#[test]
fn test_margin_shorthand_expansion() {
    let rule1 = create_test_rule_with_declarations(".button", vec![("margin", "10px 20px")]);

    let rule2 = create_test_rule_with_declarations(
        ".btn",
        vec![
            ("margin-top", "10px"),
            ("margin-right", "20px"),
            ("margin-bottom", "10px"),
            ("margin-left", "20px"),
        ],
    );

    let results = compare_css_rules(&[rule1], &[rule2], 0.7);
    assert_eq!(results.len(), 1);
    // Should have high similarity due to expanded properties matching
    assert!(results[0].similarity > 0.8);
}

#[test]
fn test_padding_shorthand_variations() {
    let rule1 = create_test_rule_with_declarations(".card", vec![("padding", "20px")]);

    let rule2 =
        create_test_rule_with_declarations(".panel", vec![("padding", "20px 20px 20px 20px")]);

    let results = compare_css_rules(&[rule1], &[rule2], 0.7);
    assert_eq!(results.len(), 1);
    // These should be nearly identical after expansion
    assert!(results[0].similarity > 0.85);
}

#[test]
fn test_flex_shorthand() {
    let rule1 = create_test_rule_with_declarations(".flex-item", vec![("flex", "1")]);

    let rule2 = create_test_rule_with_declarations(
        ".flex-element",
        vec![("flex-grow", "1"), ("flex-shrink", "1"), ("flex-basis", "0%")],
    );

    let results = compare_css_rules(&[rule1], &[rule2], 0.7);
    assert_eq!(results.len(), 1);
    assert!(results[0].similarity > 0.8);
}

#[test]
fn test_border_shorthand() {
    let rule1 =
        create_test_rule_with_declarations(".bordered", vec![("border", "1px solid black")]);

    let rule2 = create_test_rule_with_declarations(
        ".with-border",
        vec![("border-width", "1px"), ("border-style", "solid"), ("border-color", "black")],
    );

    // Note: The expanded version would have all 4 sides
    // This tests partial matching
    let results = compare_css_rules(&[rule1], &[rule2], 0.5);
    assert!(!results.is_empty());
}

#[test]
fn test_gap_shorthand() {
    let rule1 = create_test_rule_with_declarations(".grid", vec![("gap", "10px 20px")]);

    let rule2 = create_test_rule_with_declarations(
        ".grid-container",
        vec![("row-gap", "10px"), ("column-gap", "20px")],
    );

    let results = compare_css_rules(&[rule1], &[rule2], 0.7);
    assert_eq!(results.len(), 1);
    assert!(results[0].similarity > 0.8);
}

#[test]
fn test_place_items_shorthand() {
    let rule1 = create_test_rule_with_declarations(".centered", vec![("place-items", "center")]);

    let rule2 = create_test_rule_with_declarations(
        ".center-content",
        vec![("align-items", "center"), ("justify-items", "center")],
    );

    let results = compare_css_rules(&[rule1], &[rule2], 0.7);
    assert_eq!(results.len(), 1);
    assert!(results[0].similarity > 0.8);
}

#[test]
fn test_overflow_shorthand() {
    let rule1 = create_test_rule_with_declarations(".scrollable", vec![("overflow", "hidden")]);

    let rule2 = create_test_rule_with_declarations(
        ".no-overflow",
        vec![("overflow-x", "hidden"), ("overflow-y", "hidden")],
    );

    let results = compare_css_rules(&[rule1], &[rule2], 0.7);
    assert_eq!(results.len(), 1);
    assert!(results[0].similarity > 0.8);
}

#[test]
fn test_mixed_shorthand_and_longhand() {
    let rule1 = create_test_rule_with_declarations(
        ".mixed",
        vec![
            ("margin", "10px 20px"),
            ("padding-top", "5px"),
            ("padding-bottom", "5px"),
            ("flex", "1"),
        ],
    );

    let rule2 = create_test_rule_with_declarations(
        ".combined",
        vec![
            ("margin-top", "10px"),
            ("margin-right", "20px"),
            ("margin-bottom", "10px"),
            ("margin-left", "20px"),
            ("padding", "5px 0"),
            ("flex-grow", "1"),
            ("flex-shrink", "1"),
            ("flex-basis", "0%"),
        ],
    );

    let results = compare_css_rules(&[rule1], &[rule2], 0.7);
    assert_eq!(results.len(), 1);
    // Should have high similarity as the expanded properties match
    assert!(results[0].similarity > 0.75);
}

#[test]
fn test_different_shorthand_same_result() {
    let rule1 = create_test_rule_with_declarations(
        ".spacing1",
        vec![("margin", "20px"), ("padding", "10px 15px")],
    );

    let rule2 = create_test_rule_with_declarations(
        ".spacing2",
        vec![("margin", "20px 20px"), ("padding", "10px 15px 10px 15px")],
    );

    let results = compare_css_rules(&[rule1], &[rule2], 0.9);
    assert_eq!(results.len(), 1);
    // These expand to identical properties
    assert!(results[0].similarity > 0.95);
}
