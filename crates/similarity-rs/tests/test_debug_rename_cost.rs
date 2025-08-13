#![allow(clippy::uninlined_format_args)]

use similarity_core::{language_parser::LanguageParser, tsed::{calculate_tsed, TSEDOptions}};
use similarity_rs::rust_parser::RustParser;

#[test]
fn test_rename_cost_effect() {
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

    // Print AST structure
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

    println!("=== Tree 1 ===");
    print_tree(&tree1, 0);
    println!("\n=== Tree 2 ===");
    print_tree(&tree2, 0);

    // Test different rename_cost values
    for rename_cost in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 1.0] {
        let mut options = TSEDOptions::default();
        options.apted_options.rename_cost = rename_cost;
        options.apted_options.compare_values = true;
        
        let similarity = calculate_tsed(&tree1, &tree2, &options);
        println!("rename_cost = {:.1}: similarity = {:.2}%", rename_cost, similarity * 100.0);
    }
}