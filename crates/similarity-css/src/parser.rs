use similarity_core::language_parser::{LanguageParser, GenericFunctionDef, GenericTypeDef, Language};
use similarity_core::tree::TreeNode;
use tree_sitter::{Node, Parser};
use std::rc::Rc;
use std::error::Error;
use crate::scss_simple_flattener::simple_flatten_scss;

pub struct CssParser {
    parser: Parser,
    is_scss: bool,
}

impl CssParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_css::LANGUAGE.into()).unwrap();
        Self {
            parser,
            is_scss: false,
        }
    }

    pub fn new_scss() -> Self {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_scss::language()).unwrap();
        Self {
            parser,
            is_scss: true,
        }
    }

    fn convert_node(&self, node: Node, source: &str, id_counter: &mut usize) -> TreeNode {
        let current_id = *id_counter;
        *id_counter += 1;

        let label = node.kind().to_string();
        let value = match node.kind() {
            "class_selector" | "id_selector" | "tag_name" | "property_name" | "plain_value" => {
                node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
            }
            _ => "".to_string(),
        };

        let mut tree_node = TreeNode::new(label, value, current_id);

        for child in node.children(&mut node.walk()) {
            let child_node = self.convert_node(child, source, id_counter);
            tree_node.add_child(Rc::new(child_node));
        }

        tree_node
    }
}

impl LanguageParser for CssParser {
    fn parse(&mut self, content: &str, _file_path: &str) -> Result<Rc<TreeNode>, Box<dyn Error + Send + Sync>> {
        let tree = self.parser
            .parse(content, None)
            .ok_or_else(|| Box::<dyn Error + Send + Sync>::from("Failed to parse CSS/SCSS"))?;

        let root_node = tree.root_node();
        let mut id_counter = 0;
        Ok(Rc::new(self.convert_node(root_node, content, &mut id_counter)))
    }

    fn extract_functions(
        &mut self,
        content: &str,
        _file_path: &str,
    ) -> Result<Vec<GenericFunctionDef>, Box<dyn Error + Send + Sync>> {
        // For SCSS, flatten nested rules first
        if self.is_scss {
            let flat_rules = simple_flatten_scss(content)?;
            let mut functions = Vec::new();
            
            for rule in flat_rules {
                // Pass declarations through decorators field (temporary solution)
                let decorators: Vec<String> = rule.declarations.iter()
                    .map(|(prop, value)| format!("{}: {}", prop, value))
                    .collect();
                
                functions.push(GenericFunctionDef {
                    name: rule.selector,
                    start_line: rule.start_line,
                    end_line: rule.end_line,
                    body_start_line: rule.start_line,
                    body_end_line: rule.end_line,
                    parameters: vec![],
                    is_method: false,
                    class_name: None,
                    is_async: false,
                    is_generator: false,
                    decorators,
                });
            }
            
            return Ok(functions);
        }
        
        // For regular CSS, use the original method
        let tree = self.parser
            .parse(content, None)
            .ok_or_else(|| Box::<dyn Error + Send + Sync>::from("Failed to parse CSS/SCSS"))?;

        let root_node = tree.root_node();
        let mut functions = Vec::new();
        
        extract_rules(&root_node, content, &mut functions);
        
        Ok(functions)
    }

    fn extract_types(
        &mut self,
        _content: &str,
        _file_path: &str,
    ) -> Result<Vec<GenericTypeDef>, Box<dyn Error + Send + Sync>> {
        // CSS doesn't have types
        Ok(Vec::new())
    }

    fn language(&self) -> Language {
        Language::Unknown
    }
}

fn extract_rules(node: &Node, source: &str, functions: &mut Vec<GenericFunctionDef>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "rule_set" | "ruleset" => {
                if let Some(selector) = child.child_by_field_name("selectors") {
                    let selector_text = selector.utf8_text(source.as_bytes()).unwrap_or("");
                    
                    functions.push(GenericFunctionDef {
                        name: selector_text.to_string(),
                        start_line: child.start_position().row as u32 + 1,
                        end_line: child.end_position().row as u32 + 1,
                        body_start_line: child.start_position().row as u32 + 1,
                        body_end_line: child.end_position().row as u32 + 1,
                        parameters: vec![],
                        is_method: false,
                        class_name: None,
                        is_async: false,
                        is_generator: false,
                        decorators: vec![],
                    });
                }
            }
            "media_statement" | "supports_statement" | "at_rule" => {
                let at_keyword = child.child_by_field_name("at_keyword")
                    .or_else(|| child.child(0))
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("@rule");
                    
                functions.push(GenericFunctionDef {
                    name: at_keyword.to_string(),
                    start_line: child.start_position().row as u32 + 1,
                    end_line: child.end_position().row as u32 + 1,
                    body_start_line: child.start_position().row as u32 + 1,
                    body_end_line: child.end_position().row as u32 + 1,
                    parameters: vec![],
                    is_method: false,
                    class_name: None,
                    is_async: false,
                    is_generator: false,
                    decorators: vec![],
                });
            }
            "mixin_statement" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("mixin");
                    
                    functions.push(GenericFunctionDef {
                        name: format!("@mixin {}", name),
                        start_line: child.start_position().row as u32 + 1,
                        end_line: child.end_position().row as u32 + 1,
                        body_start_line: child.start_position().row as u32 + 1,
                        body_end_line: child.end_position().row as u32 + 1,
                        parameters: vec![],
                        is_method: false,
                        class_name: None,
                        is_async: false,
                        is_generator: false,
                        decorators: vec![],
                    });
                }
            }
            _ => {
                extract_rules(&child, source, functions);
            }
        }
    }
}