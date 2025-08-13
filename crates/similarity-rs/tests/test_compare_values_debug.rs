#![allow(clippy::uninlined_format_args)]

use similarity_core::{
    language_parser::LanguageParser,
    tree::TreeNode,
    tsed::{calculate_tsed, TSEDOptions},
};
use similarity_rs::rust_parser::RustParser;
use std::rc::Rc;

fn print_tree(node: &Rc<TreeNode>, indent: usize) {
    let spaces = " ".repeat(indent);
    println!("{}[id={}] label='{}', value='{}'", 
             spaces, 
             node.id, 
             node.label, 
             node.value);
    
    for child in &node.children {
        print_tree(child, indent + 2);
    }
}

#[test]
fn test_compare_values_effect() {
    // シンプルな変数名だけが異なるコード
    let code1 = r#"
    let x = 1;
    x + 2
"#;

    let code2 = r#"
    let y = 1;
    y + 2
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    println!("\n=== Tree 1 Structure ===");
    print_tree(&tree1, 0);
    
    println!("\n=== Tree 2 Structure ===");
    print_tree(&tree2, 0);

    // Test 1: compare_values = false (構造のみ比較)
    let mut options_false = TSEDOptions::default();
    options_false.apted_options.compare_values = false;
    options_false.size_penalty = false; // サイズペナルティを無効化
    
    let similarity_false = calculate_tsed(&tree1, &tree2, &options_false);
    println!("\ncompare_values=false: {:.2}%", similarity_false * 100.0);

    // Test 2: compare_values = true (値も比較)
    let mut options_true = TSEDOptions::default();
    options_true.apted_options.compare_values = true;
    options_true.size_penalty = false; // サイズペナルティを無効化
    
    let similarity_true = calculate_tsed(&tree1, &tree2, &options_true);
    println!("compare_values=true: {:.2}%", similarity_true * 100.0);

    // Test 3: 低いコストで編集距離を直接計算
    use similarity_core::apted::compute_edit_distance;
    
    let dist_false = compute_edit_distance(&tree1, &tree2, &options_false.apted_options);
    let dist_true = compute_edit_distance(&tree1, &tree2, &options_true.apted_options);
    
    println!("\n=== Edit Distance Debug ===");
    println!("Distance with compare_values=false: {}", dist_false);
    println!("Distance with compare_values=true: {}", dist_true);
    
    let size1 = tree1.get_subtree_size();
    let size2 = tree2.get_subtree_size();
    println!("Tree1 size: {}", size1);
    println!("Tree2 size: {}", size2);
    
    // 期待: compare_values=true の場合、変数名の違いで距離が増える
    assert!(
        dist_true >= dist_false,
        "compare_values=true should have equal or greater distance"
    );
}

#[test]
fn test_identical_with_compare_values() {
    // 完全に同じコード
    let code = r#"
    let x = 1;
    x + 2
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code, "test1.rs").unwrap();
    let tree2 = parser.parse(code, "test2.rs").unwrap();

    // compare_values の値に関わらず同じ結果になるはず
    let mut options_false = TSEDOptions::default();
    options_false.apted_options.compare_values = false;
    options_false.size_penalty = false;
    
    let mut options_true = TSEDOptions::default();
    options_true.apted_options.compare_values = true;
    options_true.size_penalty = false;
    
    let similarity_false = calculate_tsed(&tree1, &tree2, &options_false);
    let similarity_true = calculate_tsed(&tree1, &tree2, &options_true);
    
    println!("\n=== Identical Code Test ===");
    println!("compare_values=false: {:.2}%", similarity_false * 100.0);
    println!("compare_values=true: {:.2}%", similarity_true * 100.0);
    
    assert!(
        (similarity_false - similarity_true).abs() < 0.001,
        "Identical code should have same similarity regardless of compare_values"
    );
}

#[test]
fn test_different_literals() {
    // リテラル値が異なるコード
    let code1 = r#"
    let x = 1;
    x + 2
"#;

    let code2 = r#"
    let x = 5;
    x + 10
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    println!("\n=== Different Literals Test ===");
    
    // compare_values = false の場合
    let mut options_false = TSEDOptions::default();
    options_false.apted_options.compare_values = false;
    options_false.size_penalty = false;
    
    let similarity_false = calculate_tsed(&tree1, &tree2, &options_false);
    println!("compare_values=false: {:.2}%", similarity_false * 100.0);

    // compare_values = true の場合
    let mut options_true = TSEDOptions::default();
    options_true.apted_options.compare_values = true;
    options_true.size_penalty = false;
    
    let similarity_true = calculate_tsed(&tree1, &tree2, &options_true);
    println!("compare_values=true: {:.2}%", similarity_true * 100.0);
    
    // 期待: compare_values=true の場合、リテラル値の違いで類似度が下がる
    assert!(
        similarity_true < similarity_false,
        "Different literals should have lower similarity with compare_values=true"
    );
}