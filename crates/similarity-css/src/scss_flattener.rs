/// SCSS nested rules flattener
/// Converts nested SCSS syntax to flat CSS rules
use tree_sitter::{Node, Parser};

#[derive(Debug, Clone)]
pub struct FlatRule {
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub start_line: u32,
    pub end_line: u32,
}

/// Flatten nested SCSS rules into flat CSS rules
pub fn flatten_scss_rules(
    content: &str,
) -> Result<Vec<FlatRule>, Box<dyn std::error::Error + Send + Sync>> {
    let mut parser = Parser::new();
    let language = tree_sitter_scss::language();
    parser.set_language(&language)?;

    let tree = parser.parse(content, None).ok_or("Failed to parse SCSS")?;

    let root_node = tree.root_node();
    let mut flat_rules = Vec::new();

    flatten_node(&root_node, content, &[], &mut flat_rules);

    Ok(flat_rules)
}

/// Recursively flatten SCSS nodes
fn flatten_node(
    node: &Node,
    source: &str,
    parent_selectors: &[String],
    flat_rules: &mut Vec<FlatRule>,
) {
    // Debug: print node kind
    #[cfg(test)]
    println!(
        "Processing node: {} at {}:{}",
        node.kind(),
        node.start_position().row,
        node.start_position().column
    );

    match node.kind() {
        "rule_set" | "ruleset" => {
            // Extract selectors from this rule
            let selectors = extract_selectors(node, source);

            // Combine with parent selectors
            let combined_selectors = combine_selectors(parent_selectors, &selectors);

            // Extract declarations
            let declarations = extract_declarations(node, source);

            #[cfg(test)]
            println!(
                "  Combined selectors: {combined_selectors:?}, declarations: {declarations:?}"
            );

            // Add flat rule if there are declarations
            if !declarations.is_empty() {
                for selector in &combined_selectors {
                    flat_rules.push(FlatRule {
                        selector: selector.clone(),
                        declarations: declarations.clone(),
                        start_line: node.start_position().row as u32 + 1,
                        end_line: node.end_position().row as u32 + 1,
                    });
                }
            } else {
                #[cfg(test)]
                println!("  No declarations found for selectors: {combined_selectors:?}");
            }

            // Process nested rules in the block
            if let Some(block_node) = node.child_by_field_name("block") {
                #[cfg(test)]
                println!("  Processing block with {} children", block_node.child_count());

                for nested in block_node.children(&mut block_node.walk()) {
                    #[cfg(test)]
                    println!(
                        "    Block child for nesting: {} at {}:{}",
                        nested.kind(),
                        nested.start_position().row,
                        nested.start_position().column
                    );

                    flatten_node(&nested, source, &combined_selectors, flat_rules);
                }
            } else {
                #[cfg(test)]
                println!("  No block found for node");
            }
        }
        "at_rule" => {
            // Handle @media, @supports, etc.
            if let Some(prelude_node) = node.child_by_field_name("prelude") {
                let at_rule_text = get_node_text(&prelude_node, source);
                let at_rule = format!(
                    "@{} {}",
                    node.child(0).map(|n| get_node_text(&n, source)).unwrap_or_default(),
                    at_rule_text
                );

                // Process block inside at-rule
                if let Some(block_node) = node.child_by_field_name("block") {
                    for child in block_node.children(&mut block_node.walk()) {
                        // For at-rules, we need to wrap the selectors
                        let mut wrapped_rules = Vec::new();
                        flatten_node(&child, source, parent_selectors, &mut wrapped_rules);

                        // Add at-rule context to each flattened rule
                        for rule in wrapped_rules {
                            flat_rules.push(FlatRule {
                                selector: format!("{} {{ {} }}", at_rule, rule.selector),
                                declarations: rule.declarations,
                                start_line: rule.start_line,
                                end_line: rule.end_line,
                            });
                        }
                    }
                }
            }
        }
        _ => {
            // Process other nodes recursively
            for child in node.children(&mut node.walk()) {
                flatten_node(&child, source, parent_selectors, flat_rules);
            }
        }
    }
}

/// Extract selectors from a rule_set node
fn extract_selectors(node: &Node, source: &str) -> Vec<String> {
    let mut selectors = Vec::new();

    // Debug: print available fields
    #[cfg(test)]
    {
        println!("  Looking for selectors in node: {}", node.kind());
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                println!(
                    "    Child {}: {} - {}",
                    i,
                    child.kind(),
                    get_node_text(&child, source).trim()
                );
            }
        }
    }

    // Try different field names and direct children
    if let Some(selector_node) = node.child_by_field_name("selectors") {
        // Handle comma-separated selectors
        for child in selector_node.children(&mut selector_node.walk()) {
            if child.kind() != "," {
                let selector_text = get_node_text(&child, source).trim().to_string();
                if !selector_text.is_empty() {
                    selectors.push(selector_text);
                }
            }
        }
    } else {
        // For SCSS, selectors might be direct children before the block
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "{" {
                    break; // Stop when we reach the block
                }
                if child.kind() != "rule_set" && child.kind() != "block" {
                    let text = get_node_text(&child, source).trim();
                    if !text.is_empty() && text != "," {
                        selectors.push(text.to_string());
                    }
                }
            }
        }
    }

    #[cfg(test)]
    println!("  Extracted selectors: {selectors:?}");

    selectors
}

/// Extract declarations from a rule_set node
fn extract_declarations(node: &Node, source: &str) -> Vec<(String, String)> {
    let mut declarations = Vec::new();

    // Recursive function to find all declarations
    fn find_declarations(node: &Node, source: &str, declarations: &mut Vec<(String, String)>) {
        if node.kind() == "declaration" {
            // Extract property name and value
            let mut property = String::new();
            let mut value = String::new();
            let mut found_colon = false;

            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    match child.kind() {
                        "property_name" => {
                            property = get_node_text(&child, source).trim().to_string();
                        }
                        ":" => {
                            found_colon = true;
                        }
                        _ if found_colon && !child.kind().contains("comment") => {
                            let text = get_node_text(&child, source).trim();
                            if !text.is_empty() && text != ";" {
                                if !value.is_empty() {
                                    value.push(' ');
                                }
                                value.push_str(text);
                            }
                        }
                        _ => {}
                    }
                }
            }

            if !property.is_empty() && !value.is_empty() {
                declarations.push((property, value));
            }
        } else {
            // Recursively search children
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    // Skip nested rule_sets to avoid extracting from nested rules
                    if child.kind() != "rule_set" && child.kind() != "ruleset" {
                        find_declarations(&child, source, declarations);
                    }
                }
            }
        }
    }

    find_declarations(node, source, &mut declarations);
    declarations
}

/// Combine parent selectors with current selectors
fn combine_selectors(parents: &[String], currents: &[String]) -> Vec<String> {
    if parents.is_empty() {
        return currents.to_vec();
    }

    if currents.is_empty() {
        return parents.to_vec();
    }

    let mut combined = Vec::new();

    for parent in parents {
        for current in currents {
            let combined_selector = if let Some(suffix) = current.strip_prefix('&') {
                // Handle parent selector reference
                format!("{parent}{suffix}")
            } else if current.starts_with(':') {
                // Handle pseudo-classes/elements
                format!("{parent}{current}")
            } else {
                // Normal nesting
                format!("{parent} {current}")
            };
            combined.push(combined_selector);
        }
    }

    combined
}

/// Get text content of a node
fn get_node_text<'a>(node: &Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Using scss_simple_flattener instead"]
    fn test_simple_nesting() {
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

        let rules = flatten_scss_rules(scss).unwrap();

        println!("Found {} rules:", rules.len());
        for rule in &rules {
            println!("  - {} ({} declarations)", rule.selector, rule.declarations.len());
            for (prop, val) in &rule.declarations {
                println!("    {prop}: {val}");
            }
        }

        assert_eq!(rules.len(), 3);
        assert!(rules.iter().any(|r| r.selector == ".card"));
        assert!(rules.iter().any(|r| r.selector == ".card__header"));
        assert!(rules.iter().any(|r| r.selector == ".card__body"));
    }

    #[test]
    #[ignore = "Using scss_simple_flattener instead"]
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

        let rules = flatten_scss_rules(scss).unwrap();

        assert!(rules.iter().any(|r| r.selector == ".nav"));
        assert!(rules.iter().any(|r| r.selector == ".nav ul"));
        assert!(rules.iter().any(|r| r.selector == ".nav ul li"));
        assert!(rules.iter().any(|r| r.selector == ".nav ul li a"));
        assert!(rules.iter().any(|r| r.selector == ".nav ul li a:hover"));
    }

    #[test]
    #[ignore = "Using scss_simple_flattener instead"]
    fn test_multiple_selectors() {
        let scss = r#"
.btn,
.button {
    padding: 10px;
    
    &.primary,
    &.secondary {
        font-weight: bold;
    }
}"#;

        let rules = flatten_scss_rules(scss).unwrap();

        // Should have rules for both .btn and .button
        assert!(rules.iter().any(|r| r.selector == ".btn"));
        assert!(rules.iter().any(|r| r.selector == ".button"));

        // Should have combined selectors
        assert!(rules.iter().any(|r| r.selector == ".btn.primary"));
        assert!(rules.iter().any(|r| r.selector == ".btn.secondary"));
        assert!(rules.iter().any(|r| r.selector == ".button.primary"));
        assert!(rules.iter().any(|r| r.selector == ".button.secondary"));
    }

    #[test]
    #[ignore = "Using scss_simple_flattener instead"]
    fn test_bem_nesting() {
        let scss = r#"
.block {
    display: block;
    
    &__element {
        color: black;
        
        &--modifier {
            color: red;
        }
    }
    
    &--modifier {
        display: flex;
        
        .block__element {
            color: blue;
        }
    }
}"#;

        let rules = flatten_scss_rules(scss).unwrap();

        assert!(rules.iter().any(|r| r.selector == ".block"));
        assert!(rules.iter().any(|r| r.selector == ".block__element"));
        assert!(rules.iter().any(|r| r.selector == ".block__element--modifier"));
        assert!(rules.iter().any(|r| r.selector == ".block--modifier"));
        assert!(rules.iter().any(|r| r.selector == ".block--modifier .block__element"));
    }

    #[test]
    #[ignore = "Using scss_simple_flattener instead"]
    fn test_media_queries() {
        let scss = r#"
.container {
    width: 100%;
    
    @media (min-width: 768px) {
        width: 750px;
    }
    
    @media (min-width: 1024px) {
        width: 970px;
    }
}"#;

        let rules = flatten_scss_rules(scss).unwrap();

        assert!(rules.iter().any(|r| r.selector == ".container"));
        assert!(rules
            .iter()
            .any(|r| r.selector.contains("@media") && r.selector.contains(".container")));
    }
}
