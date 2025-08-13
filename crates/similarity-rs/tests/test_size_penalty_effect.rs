#![allow(clippy::uninlined_format_args)]

use similarity_core::{language_parser::LanguageParser, tsed::{calculate_tsed, TSEDOptions}};
use similarity_rs::rust_parser::RustParser;

#[test]
fn test_size_penalty_effect_on_short_code() {
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

    // Test with size_penalty = false
    let mut options_no_penalty = TSEDOptions::default();
    options_no_penalty.apted_options.compare_values = true;
    options_no_penalty.apted_options.rename_cost = 0.1;
    options_no_penalty.size_penalty = false;
    
    let similarity_no_penalty = calculate_tsed(&tree1, &tree2, &options_no_penalty);
    println!("\n=== Short code (2 lines) ===");
    println!("Without size penalty: {:.2}%", similarity_no_penalty * 100.0);

    // Test with size_penalty = true (default)
    let mut options_with_penalty = TSEDOptions::default();
    options_with_penalty.apted_options.compare_values = true;
    options_with_penalty.apted_options.rename_cost = 0.1;
    // size_penalty is true by default
    
    let similarity_with_penalty = calculate_tsed(&tree1, &tree2, &options_with_penalty);
    println!("With size penalty: {:.2}%", similarity_with_penalty * 100.0);
    
    // Size penalty should significantly reduce similarity for short functions
    assert!(similarity_with_penalty < similarity_no_penalty * 0.6, 
            "Size penalty should reduce similarity by at least 40% for short functions");
}

#[test]
fn test_size_penalty_effect_on_medium_code() {
    let code1 = r#"
    fn process_data(input: Vec<i32>) -> i32 {
        let mut sum = 0;
        for value in input {
            if value > 0 {
                sum += value;
            }
        }
        sum
    }
"#;

    let code2 = r#"
    fn calculate_total(data: Vec<i32>) -> i32 {
        let mut total = 0;
        for item in data {
            if item > 0 {
                total += item;
            }
        }
        total
    }
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    // Test with size_penalty = false
    let mut options_no_penalty = TSEDOptions::default();
    options_no_penalty.apted_options.compare_values = true;
    options_no_penalty.apted_options.rename_cost = 0.1;
    options_no_penalty.size_penalty = false;
    
    let similarity_no_penalty = calculate_tsed(&tree1, &tree2, &options_no_penalty);
    println!("\n=== Medium code (8 lines) ===");
    println!("Without size penalty: {:.2}%", similarity_no_penalty * 100.0);

    // Test with size_penalty = true
    let mut options_with_penalty = TSEDOptions::default();
    options_with_penalty.apted_options.compare_values = true;
    options_with_penalty.apted_options.rename_cost = 0.1;
    
    let similarity_with_penalty = calculate_tsed(&tree1, &tree2, &options_with_penalty);
    println!("With size penalty: {:.2}%", similarity_with_penalty * 100.0);
    
    // For medium-sized functions, penalty should be less severe
    assert!(similarity_with_penalty > similarity_no_penalty * 0.8, 
            "Size penalty should have less effect on medium-sized functions");
}

#[test]
fn test_realistic_expectations() {
    // Very short, structurally identical code
    let short1 = "x + 1";
    let short2 = "y + 1";
    
    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(short1, "test1.rs").unwrap();
    let tree2 = parser.parse(short2, "test2.rs").unwrap();
    
    let options = TSEDOptions::default(); // size_penalty = true by default
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("\n=== Realistic expectation for 'x + 1' vs 'y + 1' ===");
    println!("Similarity: {:.2}%", similarity * 100.0);
    
    // With size penalty, even identical structure should have low similarity for trivial code
    assert!(similarity < 0.5, "Trivial code should have low similarity even if structurally identical");
}