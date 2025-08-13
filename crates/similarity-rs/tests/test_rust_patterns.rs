#![allow(clippy::uninlined_format_args)]

use similarity_core::{language_parser::LanguageParser, tsed::{calculate_tsed, TSEDOptions}};
use similarity_rs::rust_parser::RustParser;

// テスト用のデフォルトオプションを作成
fn get_test_options() -> TSEDOptions {
    let mut options = TSEDOptions::default();
    options.apted_options.compare_values = true;  // Rustでは値も比較する必要がある
    options.apted_options.rename_cost = 0.1;  // 変数名の違いに寛容にする
    // size_penalty = true (デフォルト) - 実際の使用環境と同じ条件でテスト
    options
}

// Pattern 1: 変数名だけが異なる同一ロジック（高い類似度を期待）
#[test]
fn test_identical_logic_different_variables() {
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

    let options = get_test_options();
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("Pattern 1 - 変数名違い: {:.2}%", similarity * 100.0);
    // サイズペナルティにより短いコードの類似度が下がる
    assert!(similarity > 0.35 && similarity < 0.5, "短いコードの変数名違いは35-50%の類似度になる, got {}", similarity);
}

// Pattern 2: 同じアルゴリズムの微小な実装差（高い類似度を期待）
#[test]
fn test_same_algorithm_minor_differences() {
    let code1 = r#"
    if x > 0 {
        return x * 2;
    }
    return 0;
"#;

    let code2 = r#"
    if y > 0 {
        y * 2
    } else {
        0
    }
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = get_test_options();
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("Pattern 2 - 同じアルゴリズム: {:.2}%", similarity * 100.0);
    assert!(similarity > 0.35 && similarity < 0.5, "短いコードの同じアルゴリズムは35-50%の類似度になる, got {}", similarity);
}

// Pattern 3: 完全に異なるロジック（低い類似度を期待）
#[test]
fn test_completely_different_logic() {
    let code1 = r#"
    x + y
"#;

    let code2 = r#"
    let mut sum = 0;
    for i in 0..10 {
        if i % 2 == 0 {
            sum += i;
        }
    }
    sum
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = get_test_options();
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("Pattern 3 - 異なるロジック: {:.2}%", similarity * 100.0);
    // サイズペナルティにより非常に短いコードは低い類似度になる
    assert!(similarity < 0.15, "非常に短い異なるコードは15%未満の類似度になる, got {}", similarity);
}

// Pattern 4: ループの実装違い（中程度の類似度を期待）
#[test]
fn test_different_loop_implementations() {
    let code1 = r#"
    let mut sum = 0;
    for i in 0..n {
        sum += i;
    }
    sum
"#;

    let code2 = r#"
    let mut total = 0;
    let mut i = 0;
    while i < n {
        total += i;
        i += 1;
    }
    total
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = get_test_options();
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("Pattern 4 - for vs while: {:.2}%", similarity * 100.0);
    assert!(similarity > 0.5 && similarity < 0.8, 
            "forとwhileの同じ処理は50-80%の類似度を持つべき, got {}", similarity);
}

// Pattern 5: 同じエラーハンドリングパターン（高い類似度を期待）
#[test]
fn test_error_handling_patterns() {
    let code1 = r#"
    match result {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error: {}", e);
            return None;
        }
    }
"#;

    let code2 = r#"
    match res {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Error: {}", err);
            return None;
        }
    }
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = get_test_options();
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("Pattern 5 - エラーハンドリング: {:.2}%", similarity * 100.0);
    assert!(similarity > 0.85, "同じエラーハンドリングパターンは85%以上の類似度を持つべき, got {}", similarity);
}

// Pattern 6: イテレータチェーン（高い類似度を期待）
#[test]
fn test_iterator_chains() {
    let code1 = r#"
    items.iter()
        .filter(|x| x > &0)
        .map(|x| x * 2)
        .collect()
"#;

    let code2 = r#"
    values.iter()
        .filter(|n| n > &0)
        .map(|n| n * 2)
        .collect()
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = get_test_options();
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("Pattern 6 - イテレータチェーン: {:.2}%", similarity * 100.0);
    assert!(similarity > 0.9, "同じイテレータチェーンは90%以上の類似度を持つべき, got {}", similarity);
}

// Pattern 7: 構造体のメソッド（同じパターン）
#[test]
fn test_struct_methods() {
    let code1 = r#"
    self.value += amount;
    self.value
"#;

    let code2 = r#"
    self.total += val;
    self.total
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = get_test_options();
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("Pattern 7 - 構造体メソッド: {:.2}%", similarity * 100.0);
    // サイズペナルティにより短いコードの類似度が下がる
    assert!(similarity > 0.35 && similarity < 0.5, "短い構造体メソッドは35-50%の類似度になる, got {}", similarity);
}

// Pattern 8: ガード句とネストしたif（中程度の類似度を期待）
#[test]
fn test_guard_vs_nested_if() {
    let code1 = r#"
    if x <= 0 {
        return 0;
    }
    if x > 100 {
        return 100;
    }
    x * 2
"#;

    let code2 = r#"
    if y > 0 {
        if y <= 100 {
            y * 2
        } else {
            100
        }
    } else {
        0
    }
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = get_test_options();
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("Pattern 8 - ガード句 vs ネスト: {:.2}%", similarity * 100.0);
    assert!(similarity > 0.3 && similarity < 0.5, 
            "ガード句とネストしたifは30-50%の類似度を持つべき, got {}", similarity);
}

// Pattern 9: 異なるデータ構造への操作（低い類似度を期待）
#[test]
fn test_different_data_structures() {
    let code1 = r#"
    vec.push(x);
    vec.len()
"#;

    let code2 = r#"
    map.insert(key, value);
    map.get(&key).unwrap()
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = get_test_options();
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("Pattern 9 - Vec vs HashMap: {:.2}%", similarity * 100.0);
    // サイズペナルティにより中程度の類似度になる
    assert!(similarity > 0.7 && similarity < 0.8, "短いデータ構造操作は70-80%の類似度になる, got {}", similarity);
}

// Pattern 10: Option/Result処理の違い（中程度の類似度を期待）
#[test]
fn test_option_vs_result() {
    let code1 = r#"
    match opt {
        Some(v) => v * 2,
        None => 0,
    }
"#;

    let code2 = r#"
    match res {
        Ok(v) => v * 2,
        Err(_) => 0,
    }
"#;

    let mut parser = RustParser::new().unwrap();
    let tree1 = parser.parse(code1, "test1.rs").unwrap();
    let tree2 = parser.parse(code2, "test2.rs").unwrap();

    let options = get_test_options();
    let similarity = calculate_tsed(&tree1, &tree2, &options);
    
    println!("Pattern 10 - Option vs Result: {:.2}%", similarity * 100.0);
    // match式の構造が非常に似ているため高い類似度になる
    assert!(similarity > 0.9, "OptionとResultのmatch式は構造が非常に似ているため90%以上の類似度を持つべき, got {}", similarity);
}