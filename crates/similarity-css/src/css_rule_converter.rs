use crate::{CssParser, CssRule};
use similarity_core::language_parser::{GenericFunctionDef, LanguageParser};
use similarity_core::tree::TreeNode;
use std::rc::Rc;

/// Convert GenericFunctionDef to CssRule
pub fn convert_to_css_rule(func: &GenericFunctionDef, content: &str) -> CssRule {
    // For SCSS, we might already have declarations from flatten_scss_rules
    // Check if we need to extract declarations
    let declarations = if func.decorators.is_empty() {
        extract_declarations(func, content)
    } else {
        // Decorators can be used to pass declarations from flatten_scss_rules
        func.decorators
            .iter()
            .filter_map(|d| {
                let parts: Vec<&str> = d.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
                } else {
                    None
                }
            })
            .collect()
    };

    // Create a simple tree node for the rule
    let tree = create_rule_tree_node(&func.name, &declarations);

    CssRule {
        selector: func.name.clone(),
        declarations,
        tree,
        start_line: func.body_start_line as usize,
        end_line: func.body_end_line as usize,
    }
}

/// Extract CSS declarations from a function definition
fn extract_declarations(func: &GenericFunctionDef, content: &str) -> Vec<(String, String)> {
    // Get the content between start and end lines
    let lines: Vec<&str> = content.lines().collect();
    let start_idx = (func.body_start_line as usize).saturating_sub(1);
    let end_idx = (func.body_end_line as usize).min(lines.len());

    let mut declarations = Vec::new();

    for line_idx in start_idx..end_idx {
        if let Some(line) = lines.get(line_idx) {
            // Simple declaration parsing - look for property: value patterns
            if let Some(colon_pos) = line.find(':') {
                let property = line[..colon_pos].trim();
                let value_part = &line[colon_pos + 1..];

                // Remove trailing semicolon and comments
                let value = value_part
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .split("/*")
                    .next()
                    .unwrap_or("")
                    .trim();

                // Skip if it looks like a selector (contains { or })
                if !property.contains('{')
                    && !property.contains('}')
                    && !value.contains('{')
                    && !value.is_empty()
                {
                    declarations.push((property.to_string(), value.to_string()));
                }
            }
        }
    }

    declarations
}

/// Create a tree node for CSS rule
fn create_rule_tree_node(selector: &str, declarations: &[(String, String)]) -> Rc<TreeNode> {
    let mut rule_node = TreeNode::new(selector.to_string(), String::new(), 0);

    // Add declaration nodes as children
    for (i, (prop, value)) in declarations.iter().enumerate() {
        let decl_label = format!("{prop}: {value}");
        let decl_node = TreeNode::new(decl_label, value.clone(), i + 1);
        rule_node.add_child(Rc::new(decl_node));
    }

    Rc::new(rule_node)
}

/// Parse CSS content and convert to CssRule vector
pub fn parse_css_to_rules(
    content: &str,
    file_path: &str,
) -> Result<Vec<CssRule>, Box<dyn std::error::Error + Send + Sync>> {
    let mut parser = CssParser::new();
    let functions = parser.extract_functions(content, file_path)?;

    Ok(functions.iter().map(|func| convert_to_css_rule(func, content)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_declarations() {
        let content = r#".btn {
    color: blue;
    padding: 10px 20px;
    border: 1px solid black;
}"#;

        let func = GenericFunctionDef {
            name: ".btn".to_string(),
            start_line: 1,
            end_line: 5,
            body_start_line: 2,
            body_end_line: 4,
            parameters: vec![],
            is_method: false,
            class_name: None,
            is_async: false,
            is_generator: false,
            decorators: vec![],
        };

        let declarations = extract_declarations(&func, content);

        assert_eq!(declarations.len(), 3);
        assert_eq!(declarations[0], ("color".to_string(), "blue".to_string()));
        assert_eq!(declarations[1], ("padding".to_string(), "10px 20px".to_string()));
        assert_eq!(declarations[2], ("border".to_string(), "1px solid black".to_string()));
    }

    #[test]
    fn test_convert_to_css_rule() {
        let content = r#".card {
    background: white;
    padding: 20px;
}"#;

        let func = GenericFunctionDef {
            name: ".card".to_string(),
            start_line: 1,
            end_line: 4,
            body_start_line: 2,
            body_end_line: 3,
            parameters: vec![],
            is_method: false,
            class_name: None,
            is_async: false,
            is_generator: false,
            decorators: vec![],
        };

        let rule = convert_to_css_rule(&func, content);

        assert_eq!(rule.selector, ".card");
        assert_eq!(rule.declarations.len(), 2);
        assert_eq!(rule.start_line, 2);
        assert_eq!(rule.end_line, 3);
    }
}
