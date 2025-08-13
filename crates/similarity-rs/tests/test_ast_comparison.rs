#![allow(clippy::uninlined_format_args)]

use similarity_core::{
    language_parser::LanguageParser,
    tsed::{calculate_tsed, TSEDOptions},
};
use similarity_rs::rust_parser::RustParser;

#[test]
fn test_different_functions_should_have_low_similarity() {
    let code1 = r#"
    let result = x + 1;
    result * 2
"#;

    let code2 = r#"
    let mut sum = 0;
    for val in values {
        if val > 0 {
            sum += val;
        }
    }
    sum
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = TSEDOptions::default();
    let similarity = calculate_tsed(&tree1, &tree2, &options);

    println!("Similarity between addition and loop: {:.2}%", similarity * 100.0);

    // These are completely different - similarity should be low
    assert!(similarity < 0.5, "Different functions should have low similarity, got {}", similarity);
}

#[test]
fn test_similar_functions_should_have_high_similarity() {
    let code1 = r#"
    let result = x + 1;
    result * 2
"#;

    let code2 = r#"
    let temp = y + 1;
    temp * 2
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = TSEDOptions::default();
    let similarity = calculate_tsed(&tree1, &tree2, &options);

    println!("Similarity between similar functions: {:.2}%", similarity * 100.0);

    // These are very similar - similarity should be high
    assert!(similarity > 0.8, "Similar functions should have high similarity, got {}", similarity);
}

#[test]
fn test_ast_tree_structure() {
    let code = r#"
    let result = x + 1;
    result * 2
"#;

    let mut parser = RustParser::new().unwrap();
    let tree = parser.parse(code, "test.rs").unwrap();

    fn print_tree(node: &similarity_core::tree::TreeNode, depth: usize) {
        let indent = "  ".repeat(depth);
        if node.value.is_empty() {
            println!("{}{}", indent, node.label);
        } else {
            println!("{}{} = '{}'", indent, node.label, node.value);
        }
        for child in &node.children {
            print_tree(child, depth + 1);
        }
    }

    println!("=== AST Structure ===");
    print_tree(&tree, 0);

    // Check that the tree has reasonable structure
    assert!(tree.get_subtree_size() > 5, "Tree should have multiple nodes");
}
