#![allow(clippy::uninlined_format_args)]

use similarity_core::{language_parser::LanguageParser, tsed::{calculate_tsed, TSEDOptions}};
use similarity_rs::rust_parser::RustParser;

// ASTベースで検出可能なパターンに絞ったテストケース

// Pattern 1: 完全に同じコードのコピー（検出すべき）
#[test]
fn test_exact_copy() {
    let code1 = r#"
    let result = calculate_value(x);
    if result > 0 {
        process_result(result);
    }
    result
"#;

    let code2 = r#"
    let result = calculate_value(x);
    if result > 0 {
        process_result(result);
    }
    result
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let mut options = TSEDOptions::default();
    options.apted_options.compare_values = true;
    
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    println!("Pattern 1 - 完全コピー: {:.2}%", similarity * 100.0);
    assert!(similarity > 0.99, "完全に同じコードは99%以上の類似度を持つべき, got {}", similarity);
}

// Pattern 2: 変数名を一括置換したコピー（検出すべき）
#[test]
fn test_renamed_copy() {
    let code1 = r#"
    let result = calculate_value(x);
    if result > 0 {
        process_result(result);
    }
    result
"#;

    let code2 = r#"
    let output = calculate_value(y);
    if output > 0 {
        process_result(output);
    }
    output
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let mut options = TSEDOptions::default();
    options.apted_options.compare_values = true;
    options.apted_options.rename_cost = 0.1; // 変数名の違いには寛容
    
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    println!("Pattern 2 - 変数名置換: {:.2}%", similarity * 100.0);
    // compare_values=true でrename_cost=0.1のため、変数名の違いによる影響は小さい
    // 構造が同じで変数名のみ異なる場合、90%以上の類似度になる
    assert!(similarity > 0.9, 
            "変数名を置換したコピーは90%以上の類似度を持つべき, got {}", similarity);
}

// Pattern 3: 明らかに異なる処理（検出すべきでない）
#[test]
fn test_different_logic() {
    let code1 = r#"
    x + y
"#;

    let code2 = r#"
    let mut map = HashMap::new();
    for item in items {
        map.insert(item.key, item.value);
    }
    map
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let mut options = TSEDOptions::default();
    options.apted_options.compare_values = true;
    
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    println!("Pattern 3 - 異なる処理: {:.2}%", similarity * 100.0);
    assert!(similarity < 0.3, "明らかに異なる処理は30%未満の類似度であるべき, got {}", similarity);
}

// Pattern 4: 同じ条件分岐構造（構造的に類似）
#[test]
fn test_similar_control_flow() {
    let code1 = r#"
    if condition1 {
        action1();
    } else if condition2 {
        action2();
    } else {
        default_action();
    }
"#;

    let code2 = r#"
    if check1 {
        do_something();
    } else if check2 {
        do_other();
    } else {
        do_default();
    }
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let mut options = TSEDOptions::default();
    options.apted_options.compare_values = true;
    
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    println!("Pattern 4 - 同じ制御フロー: {:.2}%", similarity * 100.0);
    // 構造は同じだが、関数名が異なるため、中程度の類似度が現実的
    // rename_cost=0.1で構造が同じため高い類似度になる
    assert!(similarity > 0.9, "同じ制御フロー構造は90%以上の類似度を持つべき, got {}", similarity);
}

// Pattern 5: 行の順序を入れ替えたコード（構造的に異なる）
#[test]
fn test_reordered_lines() {
    let code1 = r#"
    let a = 1;
    let b = 2;
    let c = 3;
    a + b + c
"#;

    let code2 = r#"
    let b = 2;
    let c = 3;
    let a = 1;
    a + b + c
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let mut options = TSEDOptions::default();
    options.apted_options.compare_values = true;
    
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    println!("Pattern 5 - 行順序入れ替え: {:.2}%", similarity * 100.0);
    // 変数定義の順序が異なるが、最終的な計算式は同じ
    // ノードIDが正しく設定されたことで、より正確な編集距離が計算される
    // 変数定義の順序が異なるが、計算式は同じなので中程度の類似度
    assert!(similarity > 0.7 && similarity < 0.85, "行順序の入れ替えは70-85%の類似度であるべき, got {}", similarity);
}

// Pattern 6: match式の同じパターン（構造的に類似）
#[test]
fn test_match_patterns() {
    let code1 = r#"
    match value {
        Some(x) => x * 2,
        None => 0,
    }
"#;

    let code2 = r#"
    match data {
        Some(y) => y * 2,
        None => 0,
    }
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let mut options = TSEDOptions::default();
    options.apted_options.compare_values = true;
    
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    println!("Pattern 6 - match式: {:.2}%", similarity * 100.0);
    // 構造はほぼ同じ、変数名のみ異なる
    assert!(similarity > 0.7, "同じmatchパターンは70%以上の類似度を持つべき, got {}", similarity);
}

// Pattern 7: 大きさが大きく異なるコード（検出すべきでない）
#[test]
fn test_size_difference() {
    let code1 = r#"
    x + 1
"#;

    let code2 = r#"
    let mut result = 0;
    for i in 0..100 {
        if i % 2 == 0 {
            result += i;
        } else {
            result -= i;
        }
    }
    if result > 0 {
        println!("Positive");
    } else {
        println!("Negative");
    }
    result
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let mut options = TSEDOptions::default();
    options.apted_options.compare_values = true;
    options.size_penalty = true; // サイズペナルティを有効化
    
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    println!("Pattern 7 - サイズ差: {:.2}%", similarity * 100.0);
    assert!(similarity < 0.2, "大きさが大きく異なるコードは20%未満の類似度であるべき, got {}", similarity);
}

// Pattern 8: 同じ構造体フィールドアクセスパターン
#[test]
fn test_struct_field_access() {
    let code1 = r#"
    self.field1 = value1;
    self.field2 = value2;
    self.field1 + self.field2
"#;

    let code2 = r#"
    self.field1 = val1;
    self.field2 = val2;
    self.field1 + self.field2
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let mut options = TSEDOptions::default();
    options.apted_options.compare_values = true;
    
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    println!("Pattern 8 - 構造体アクセス: {:.2}%", similarity * 100.0);
    // フィールド名は同じ、値の変数名のみ異なる
    assert!(similarity > 0.6, "同じ構造体アクセスパターンは60%以上の類似度を持つべき, got {}", similarity);
}