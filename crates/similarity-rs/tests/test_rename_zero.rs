#![allow(clippy::uninlined_format_args)]

use similarity_core::{language_parser::LanguageParser, tsed::{calculate_tsed, TSEDOptions}};
use similarity_rs::rust_parser::RustParser;

#[test]
fn test_rename_cost_zero() {
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

    // rename_cost = 0.0, compare_values = true
    let mut options = TSEDOptions::default();
    options.apted_options.rename_cost = 0.0;
    options.apted_options.compare_values = true;
    
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    println!("With compare_values=true, rename_cost=0.0: {:.2}%", similarity * 100.0);

    // rename_cost = 0.0, compare_values = false (構造のみ比較)
    options.apted_options.compare_values = false;
    let similarity2 = calculate_tsed(&tree1, &tree2, &options);
    println!("With compare_values=false, rename_cost=0.0: {:.2}%", similarity2 * 100.0);
    
    // デフォルト設定
    let options_default = TSEDOptions::default();
    let similarity3 = calculate_tsed(&tree1, &tree2, &options_default);
    println!("With default settings: {:.2}%", similarity3 * 100.0);
}