use similarity_core::language_parser::LanguageParser;
use similarity_rs::rust_parser::RustParser;

#[test]
fn test_function_extraction() {
    let content = r#"fn f1() -> i32 { 1 }
fn f2() -> i32 { 1 }

fn longer_func1() -> i32 {
    let x = 1;
    let y = 2;
    let z = 3;
    x + y + z
}

fn longer_func2() -> i32 {
    let a = 1;
    let b = 2; 
    let c = 3;
    a + b + c
}"#;

    let mut parser = RustParser::new().unwrap();
    let functions = parser.extract_functions(content, "test.rs").unwrap();
    
    println!("\n=== Extracted Functions ===");
    for (i, func) in functions.iter().enumerate() {
        println!("[{}] {}: lines {}-{}, body {}-{}", 
            i,
            func.name, 
            func.start_line, 
            func.end_line,
            func.body_start_line,
            func.body_end_line);
        
        // Extract body
        let lines: Vec<&str> = content.lines().collect();
        let start_idx = if func.body_start_line > 0 {
            (func.body_start_line.saturating_sub(1)) as usize
        } else {
            (func.start_line.saturating_sub(1)) as usize
        };
        
        let end_idx = if func.body_end_line > 0 {
            std::cmp::min(func.body_end_line as usize, lines.len())
        } else {
            std::cmp::min(func.end_line as usize, lines.len())
        };
        
        let body = lines[start_idx..end_idx].join("\n");
        println!("  Start idx: {}, End idx: {}", start_idx, end_idx);
        println!("  Lines total: {}", lines.len());
        if start_idx < lines.len() {
            println!("  Line at start_idx: {:?}", lines[start_idx]);
        }
        println!("  Body: {}", body.replace('\n', "\\n"));
    }
    
    assert_eq!(functions.len(), 4);
    assert_eq!(functions[2].name, "longer_func1");
    assert_eq!(functions[3].name, "longer_func2");
    
    // Check that longer functions have correct line counts
    let func1_lines = functions[2].end_line - functions[2].start_line + 1;
    let func2_lines = functions[3].end_line - functions[3].start_line + 1;
    
    println!("\nlonger_func1 has {} lines", func1_lines);
    println!("longer_func2 has {} lines", func2_lines);
    
    assert!(func1_lines >= 5, "longer_func1 should have at least 5 lines");
    assert!(func2_lines >= 5, "longer_func2 should have at least 5 lines");
}