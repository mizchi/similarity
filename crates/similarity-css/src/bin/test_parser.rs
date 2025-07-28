fn main() {
    let scss_content = r#".m-0 { margin: 0; }
.m-1 { margin: 0.25rem; }
.m-2 { margin: 0.5rem; }"#;

    println!("Testing SCSS parser with single-line rules:");
    println!("{}", scss_content);
    println!("\n---\n");

    use similarity_css::scss_simple_flattener::simple_flatten_scss;
    
    match simple_flatten_scss(scss_content) {
        Ok(rules) => {
            println!("Found {} rules:", rules.len());
            for rule in &rules {
                println!("  - {} (lines {}-{}, {} declarations)", 
                    rule.selector, rule.start_line, rule.end_line, rule.declarations.len());
                for (prop, val) in &rule.declarations {
                    println!("    {}: {}", prop, val);
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}