/// Simple SCSS flattener that uses text processing
use std::error::Error;

#[derive(Debug, Clone)]
pub struct SimpleFlatRule {
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub start_line: u32,
    pub end_line: u32,
}


/// Simple regex-based SCSS flattener
pub fn simple_flatten_scss(content: &str) -> Result<Vec<SimpleFlatRule>, Box<dyn Error + Send + Sync>> {
    let mut rules = Vec::new();
    let mut selector_stack: Vec<Vec<String>> = Vec::new();
    let mut in_rule = false;
    let mut current_declarations = Vec::new();
    let mut rule_start_line = 0;
    let mut pending_selector = String::new();
    
    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num as u32 + 1;
        let trimmed = line.trim();
        
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        
        // Count braces
        let open_braces = line.chars().filter(|&c| c == '{').count();
        let close_braces = line.chars().filter(|&c| c == '}').count();
        
        // Check if we're collecting a multi-line selector
        if trimmed.ends_with(",") && open_braces == 0 {
            // This line continues on the next line
            pending_selector.push_str(trimmed);
            pending_selector.push(' ');
            continue;
        }
        
        // Detect selector
        if open_braces > 0 && !trimmed.starts_with("@if") && !trimmed.starts_with("@else") {
            let selector_part = if !pending_selector.is_empty() {
                // Add the final part (before the opening brace)
                pending_selector.push_str(line.split('{').next().unwrap_or("").trim());
                let full_selector = pending_selector.clone();
                pending_selector.clear();
                full_selector
            } else {
                line.split('{').next().unwrap_or("").trim().to_string()
            };
            
            if !selector_part.is_empty() {
                // Save any pending rule
                if !current_declarations.is_empty() && !selector_stack.is_empty() {
                    if let Some(current_selectors) = selector_stack.last() {
                        for selector in current_selectors {
                            if !selector.starts_with('@') {
                                rules.push(SimpleFlatRule {
                                    selector: selector.clone(),
                                    declarations: current_declarations.clone(),
                                    start_line: rule_start_line,
                                    end_line: line_num - 1,
                                });
                            }
                        }
                    }
                }
                
                // Parse the new selector(s)
                let selectors: Vec<&str> = selector_part.split(',').map(|s| s.trim()).collect();
                let mut expanded_selectors = Vec::new();
                
                for selector in selectors {
                    if selector.starts_with('&') {
                        // Nested selector with parent reference
                        if let Some(parent_selectors) = selector_stack.last() {
                            for parent in parent_selectors {
                                if !parent.starts_with('@') {
                                    let combined = process_ampersand_selector(parent, selector);
                                    expanded_selectors.push(combined);
                                }
                            }
                        }
                    } else if selector.starts_with('@') {
                        // At-rule
                        expanded_selectors.push(selector.to_string());
                    } else {
                        // Regular selector
                        if let Some(parent_selectors) = selector_stack.last() {
                            for parent in parent_selectors {
                                if !parent.starts_with('@') {
                                    expanded_selectors.push(format!("{} {}", parent, selector));
                                } else {
                                    expanded_selectors.push(selector.to_string());
                                }
                            }
                        } else {
                            expanded_selectors.push(selector.to_string());
                        }
                    }
                }
                
                selector_stack.push(expanded_selectors);
                in_rule = true;
                rule_start_line = line_num;
                current_declarations.clear();
            }
        }
        
        // Parse declarations (handle single-line rules)
        if open_braces > 0 && close_braces > 0 {
            // Single-line rule like: .m-0 { margin: 0; }
            let content_between = line.split('{').nth(1)
                .and_then(|s| s.split('}').next())
                .unwrap_or("");
            
            for declaration in content_between.split(';') {
                let declaration = declaration.trim();
                if declaration.contains(':') {
                    let parts: Vec<&str> = declaration.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let property = parts[0].trim();
                        let value = parts[1].trim();
                        if !property.is_empty() && !value.is_empty() && !property.starts_with('@') {
                            current_declarations.push((property.to_string(), value.to_string()));
                        }
                    }
                }
            }
        } else if in_rule && trimmed.contains(':') && !trimmed.contains('{') {
            // Multi-line rule declarations
            let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
            if parts.len() == 2 {
                let property = parts[0].trim();
                let value = parts[1].trim_end_matches(';').trim();
                if !property.is_empty() && !value.is_empty() && !property.starts_with('@') {
                    current_declarations.push((property.to_string(), value.to_string()));
                }
            }
        }
        
        // Close rule
        if close_braces > 0 {
            for _ in 0..close_braces {
                if !current_declarations.is_empty() && !selector_stack.is_empty() {
                    if let Some(current_selectors) = selector_stack.last() {
                        for selector in current_selectors {
                            if !selector.starts_with('@') {
                                rules.push(SimpleFlatRule {
                                    selector: selector.clone(),
                                    declarations: current_declarations.clone(),
                                    start_line: rule_start_line,
                                    end_line: line_num,
                                });
                            }
                        }
                    }
                }
                
                if !selector_stack.is_empty() {
                    selector_stack.pop();
                }
                current_declarations.clear();
                in_rule = !selector_stack.is_empty();
                
                if !selector_stack.is_empty() {
                    rule_start_line = line_num + 1;
                }
            }
        }
    }
    
    Ok(rules)
}

fn process_ampersand_selector(parent: &str, selector: &str) -> String {
    if selector.starts_with("&::") || selector.starts_with("&:") {
        // Pseudo-elements and pseudo-classes
        format!("{}{}", parent, &selector[1..])
    } else if selector.starts_with("&.") {
        // Additional class
        format!("{}{}", parent, &selector[1..])
    } else if selector.starts_with("&__") || selector.starts_with("&--") || selector.starts_with("&-") {
        // BEM notation
        format!("{}{}", parent, &selector[1..])
    } else if selector.starts_with("&[") {
        // Attribute selector
        format!("{}{}", parent, &selector[1..])
    } else if selector == "&" {
        // Just ampersand
        parent.to_string()
    } else {
        // Space-separated nested selector
        format!("{} {}", parent, &selector[1..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_scss_flattening() {
        let scss = r#"
.card {
    background: white;
    padding: 20px;
    
    &__header {
        font-size: 18px;
        font-weight: bold;
    }
    
    &__body {
        font-size: 14px;
    }
}"#;
        
        let rules = simple_flatten_scss(scss).unwrap();
        
        println!("Found {} rules:", rules.len());
        for rule in &rules {
            println!("  - {} ({} declarations)", rule.selector, rule.declarations.len());
            for (prop, val) in &rule.declarations {
                println!("    {}: {}", prop, val);
            }
        }
        
        assert_eq!(rules.len(), 3);
        assert!(rules.iter().any(|r| r.selector == ".card"));
        assert!(rules.iter().any(|r| r.selector == ".card__header"));
        assert!(rules.iter().any(|r| r.selector == ".card__body"));
    }
    
    #[test]
    fn test_deep_nesting() {
        let scss = r#"
.nav {
    display: flex;
    
    ul {
        list-style: none;
        
        li {
            padding: 10px;
            
            a {
                color: blue;
                
                &:hover {
                    color: red;
                }
            }
        }
    }
}"#;
        
        let rules = simple_flatten_scss(scss).unwrap();
        
        assert!(rules.iter().any(|r| r.selector == ".nav"));
        assert!(rules.iter().any(|r| r.selector == ".nav ul"));
        assert!(rules.iter().any(|r| r.selector == ".nav ul li"));
        assert!(rules.iter().any(|r| r.selector == ".nav ul li a"));
        assert!(rules.iter().any(|r| r.selector == ".nav ul li a:hover"));
    }
    
    #[test]
    fn test_multiple_selectors() {
        let scss = r#"
.form-control {
    width: 100%;
    
    &[type="text"],
    &[type="email"],
    &[type="password"] {
        font-family: inherit;
    }
}"#;
        
        let rules = simple_flatten_scss(scss).unwrap();
        
        println!("Multiple selector test - Found {} rules:", rules.len());
        for rule in &rules {
            println!("  - {} ({} decls)", rule.selector, rule.declarations.len());
            for (k, v) in &rule.declarations {
                println!("    {}: {}", k, v);
            }
        }
        
        assert!(rules.iter().any(|r| r.selector == ".form-control"));
        assert!(rules.iter().any(|r| r.selector == ".form-control[type=\"text\"]"));
        assert!(rules.iter().any(|r| r.selector == ".form-control[type=\"email\"]"));
        assert!(rules.iter().any(|r| r.selector == ".form-control[type=\"password\"]"));
    }
    
    #[test]
    fn test_bem_notation() {
        let scss = r#"
.form {
    margin: 0;
    
    &-group {
        margin-bottom: 20px;
    }
    
    &__label {
        display: block;
    }
    
    &--inline {
        display: inline-block;
    }
}"#;
        
        let rules = simple_flatten_scss(scss).unwrap();
        
        println!("BEM notation test - Found {} rules:", rules.len());
        for rule in &rules {
            println!("  - {}", rule.selector);
        }
        
        assert!(rules.iter().any(|r| r.selector == ".form"));
        assert!(rules.iter().any(|r| r.selector == ".form-group"));
        assert!(rules.iter().any(|r| r.selector == ".form__label"));
        assert!(rules.iter().any(|r| r.selector == ".form--inline"));
    }
    
    #[test]
    fn test_complex_multi_selectors() {
        let scss = r#"
.form-group {
    input,
    textarea,
    select {
        width: 100%;
        
        &:focus {
            border-color: blue;
        }
        
        &.error {
            border-color: red;
            
            &:focus {
                border-color: darkred;
            }
        }
    }
}"#;
        
        let rules = simple_flatten_scss(scss).unwrap();
        
        println!("Complex multi-selector test - Found {} rules:", rules.len());
        for rule in &rules {
            println!("  - {}", rule.selector);
        }
        
        // Should have base selectors
        assert!(rules.iter().any(|r| r.selector == ".form-group input"));
        assert!(rules.iter().any(|r| r.selector == ".form-group textarea"));
        assert!(rules.iter().any(|r| r.selector == ".form-group select"));
        
        // Should have :focus variants
        assert!(rules.iter().any(|r| r.selector == ".form-group input:focus"));
        assert!(rules.iter().any(|r| r.selector == ".form-group textarea:focus"));
        assert!(rules.iter().any(|r| r.selector == ".form-group select:focus"));
        
        // Should have .error variants
        assert!(rules.iter().any(|r| r.selector == ".form-group input.error"));
        assert!(rules.iter().any(|r| r.selector == ".form-group textarea.error"));
        assert!(rules.iter().any(|r| r.selector == ".form-group select.error"));
        
        // Should have .error:focus variants
        assert!(rules.iter().any(|r| r.selector == ".form-group input.error:focus"));
        assert!(rules.iter().any(|r| r.selector == ".form-group textarea.error:focus"));
        assert!(rules.iter().any(|r| r.selector == ".form-group select.error:focus"));
    }
}