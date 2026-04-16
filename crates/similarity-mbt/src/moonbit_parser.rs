#![allow(clippy::io_other_error)]

use similarity_core::language_parser::{
    GenericFunctionDef, GenericTypeDef, Language, LanguageParser,
};
use similarity_core::tree::TreeNode;
use std::error::Error;
use std::rc::Rc;
use tree_sitter::{Node, Parser};

pub struct MoonBitParser {
    parser: Parser,
}

impl MoonBitParser {
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_moonbit::LANGUAGE.into()).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to set MoonBit language: {e:?}"),
            )) as Box<dyn Error + Send + Sync>
        })?;

        Ok(Self { parser })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn convert_node(&self, node: Node, source_bytes: &[u8], id_counter: &mut usize) -> TreeNode {
        let current_id = *id_counter;
        *id_counter += 1;

        let kind = node.kind();
        let label = kind.to_string();
        let value = match kind {
            "lowercase_identifier"
            | "uppercase_identifier"
            | "dot_lowercase_identifier"
            | "integer_literal"
            | "float_literal"
            | "string_literal"
            | "char_literal"
            | "true"
            | "false" => node.utf8_text(source_bytes).unwrap_or("").to_string(),
            _ => String::new(),
        };

        let mut tree_node = TreeNode::new(label, value, current_id);
        let mut subtree_size: usize = 1;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let child_node = self.convert_node(child, source_bytes, id_counter);
            subtree_size += child_node.subtree_size.unwrap_or(1);
            tree_node.add_child(Rc::new(child_node));
        }

        tree_node.subtree_size = Some(subtree_size);
        tree_node
    }

    fn extract_functions_from_node(&self, node: Node, source: &str) -> Vec<GenericFunctionDef> {
        let mut functions = Vec::new();

        fn visit_node(node: Node, source: &str, functions: &mut Vec<GenericFunctionDef>) {
            match node.kind() {
                "function_definition" => {
                    if let Some(func) = extract_function_def(node, source, None) {
                        functions.push(func);
                    }
                }
                "impl_definition" => {
                    // Extract the type name being implemented for
                    let type_name = extract_impl_type_name(node, source);

                    // Find function_identifier + parameters + block_expression within impl
                    // impl definitions can contain inline method definitions
                    let func_id = find_child_by_kind(node, "function_identifier");
                    let params = find_child_by_kind(node, "parameters");
                    let body = find_child_by_kind(node, "block_expression");

                    if let (Some(func_id_node), Some(_params_node), Some(_body_node)) =
                        (func_id, params, body)
                    {
                        let name = extract_identifier_text(func_id_node, source);
                        let param_list =
                            params.map(|p| extract_params(p, source)).unwrap_or_default();

                        functions.push(GenericFunctionDef {
                            name,
                            start_line: node.start_position().row as u32 + 1,
                            end_line: node.end_position().row as u32 + 1,
                            body_start_line: body
                                .map(|n| n.start_position().row as u32 + 1)
                                .unwrap_or(0),
                            body_end_line: body
                                .map(|n| n.end_position().row as u32 + 1)
                                .unwrap_or(0),
                            parameters: param_list,
                            is_method: true,
                            class_name: type_name,
                            is_async: false,
                            is_generator: false,
                            decorators: Vec::new(),
                        });
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        visit_node(child, source, functions);
                    }
                }
            }
        }

        fn extract_function_def(
            node: Node,
            source: &str,
            class_name: Option<String>,
        ) -> Option<GenericFunctionDef> {
            let func_id = find_child_by_kind(node, "function_identifier")?;
            let name = extract_identifier_text(func_id, source);
            let params = find_child_by_kind(node, "parameters");
            let body = find_child_by_kind(node, "block_expression");
            let param_list = params.map(|p| extract_params(p, source)).unwrap_or_default();

            Some(GenericFunctionDef {
                name,
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
                body_start_line: body.map(|n| n.start_position().row as u32 + 1).unwrap_or(0),
                body_end_line: body.map(|n| n.end_position().row as u32 + 1).unwrap_or(0),
                parameters: param_list,
                is_method: class_name.is_some(),
                class_name,
                is_async: false,
                is_generator: false,
                decorators: Vec::new(),
            })
        }

        fn extract_impl_type_name(node: Node, source: &str) -> Option<String> {
            // In impl_definition: impl TraitName for TypeName with method_name(params) { body }
            // The second qualified_type_identifier (after "for") is the type name
            let mut cursor = node.walk();
            let mut found_for = false;
            for child in node.children(&mut cursor) {
                if child.kind() == "for" {
                    found_for = true;
                    continue;
                }
                if found_for && child.kind() == "qualified_type_identifier" {
                    return Some(child.utf8_text(source.as_bytes()).unwrap_or("").to_string());
                }
            }
            None
        }

        #[allow(clippy::manual_find)]
        fn find_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == kind {
                    return Some(child);
                }
            }
            None
        }

        fn extract_identifier_text(node: Node, source: &str) -> String {
            // function_identifier -> lowercase_identifier
            // identifier -> uppercase_identifier
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "lowercase_identifier" | "uppercase_identifier" => {
                        return child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    }
                    _ => {}
                }
            }
            node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
        }

        fn extract_params(params_node: Node, source: &str) -> Vec<String> {
            let mut params = Vec::new();
            let mut cursor = params_node.walk();

            for child in params_node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    // parameter -> positional_parameter / labelled_parameter / optional_parameter
                    let mut param_cursor = child.walk();
                    for param_child in child.children(&mut param_cursor) {
                        match param_child.kind() {
                            "positional_parameter"
                            | "labelled_parameter"
                            | "optional_parameter"
                            | "optional_parameter_with_default" => {
                                // Look for lowercase_identifier inside
                                let mut inner_cursor = param_child.walk();
                                for inner in param_child.children(&mut inner_cursor) {
                                    if inner.kind() == "lowercase_identifier" {
                                        if let Ok(text) = inner.utf8_text(source.as_bytes()) {
                                            params.push(text.to_string());
                                        }
                                        break;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            params
        }

        visit_node(node, source, &mut functions);
        functions
    }
}

impl LanguageParser for MoonBitParser {
    fn parse(
        &mut self,
        source: &str,
        _filename: &str,
    ) -> Result<Rc<TreeNode>, Box<dyn Error + Send + Sync>> {
        let tree = self.parser.parse(source, None).ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse MoonBit source",
            )) as Box<dyn Error + Send + Sync>
        })?;

        let root_node = tree.root_node();
        let mut id_counter = 0;
        Ok(Rc::new(self.convert_node(root_node, source.as_bytes(), &mut id_counter)))
    }

    fn extract_functions(
        &mut self,
        source: &str,
        _filename: &str,
    ) -> Result<Vec<GenericFunctionDef>, Box<dyn Error + Send + Sync>> {
        let tree = self.parser.parse(source, None).ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse MoonBit source",
            )) as Box<dyn Error + Send + Sync>
        })?;

        let root_node = tree.root_node();
        Ok(self.extract_functions_from_node(root_node, source))
    }

    fn extract_types(
        &mut self,
        source: &str,
        _filename: &str,
    ) -> Result<Vec<GenericTypeDef>, Box<dyn Error + Send + Sync>> {
        let tree = self.parser.parse(source, None).ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse MoonBit source",
            )) as Box<dyn Error + Send + Sync>
        })?;

        let root_node = tree.root_node();
        let mut types = Vec::new();

        fn visit_node_for_types(node: Node, source: &str, types: &mut Vec<GenericTypeDef>) {
            match node.kind() {
                "struct_definition" => {
                    if let Some(type_def) = extract_struct_def(node, source) {
                        types.push(type_def);
                    }
                }
                "enum_definition" | "error_type_definition" => {
                    if let Some(type_def) = extract_enum_def(node, source) {
                        types.push(type_def);
                    }
                }
                "trait_definition" => {
                    if let Some(type_def) = extract_trait_def(node, source) {
                        types.push(type_def);
                    }
                }
                "type_definition" => {
                    if let Some(type_def) = extract_type_alias(node, source) {
                        types.push(type_def);
                    }
                }
                _ => {}
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                visit_node_for_types(child, source, types);
            }
        }

        fn find_identifier(node: Node, source: &str) -> Option<String> {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    // identifier -> uppercase_identifier
                    let mut inner = child.walk();
                    for grandchild in child.children(&mut inner) {
                        if grandchild.kind() == "uppercase_identifier" {
                            return grandchild
                                .utf8_text(source.as_bytes())
                                .ok()
                                .map(|s| s.to_string());
                        }
                    }
                }
            }
            None
        }

        fn extract_struct_def(node: Node, source: &str) -> Option<GenericTypeDef> {
            let name = find_identifier(node, source)?;
            let mut fields = Vec::new();

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "struct_field_declaration" {
                    let mut field_cursor = child.walk();
                    for field_child in child.children(&mut field_cursor) {
                        if field_child.kind() == "lowercase_identifier" {
                            if let Ok(field_name) = field_child.utf8_text(source.as_bytes()) {
                                fields.push(field_name.to_string());
                            }
                        }
                    }
                }
            }

            Some(GenericTypeDef {
                name,
                kind: "struct".to_string(),
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
                fields,
            })
        }

        fn extract_enum_def(node: Node, source: &str) -> Option<GenericTypeDef> {
            let name = find_identifier(node, source)?;
            let mut fields = Vec::new();

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "enum_constructor" {
                    let mut constructor_cursor = child.walk();
                    for constructor_child in child.children(&mut constructor_cursor) {
                        if constructor_child.kind() == "uppercase_identifier" {
                            if let Ok(variant_name) = constructor_child.utf8_text(source.as_bytes())
                            {
                                fields.push(variant_name.to_string());
                            }
                        }
                    }
                }
            }

            Some(GenericTypeDef {
                name,
                kind: "enum".to_string(),
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
                fields,
            })
        }

        fn extract_trait_def(node: Node, source: &str) -> Option<GenericTypeDef> {
            let name = find_identifier(node, source)?;
            let mut fields = Vec::new();

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "trait_method_declaration" {
                    let mut method_cursor = child.walk();
                    for method_child in child.children(&mut method_cursor) {
                        if method_child.kind() == "function_identifier" {
                            let mut id_cursor = method_child.walk();
                            for id_child in method_child.children(&mut id_cursor) {
                                if id_child.kind() == "lowercase_identifier" {
                                    if let Ok(method_name) = id_child.utf8_text(source.as_bytes()) {
                                        fields.push(method_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Some(GenericTypeDef {
                name,
                kind: "trait".to_string(),
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
                fields,
            })
        }

        fn extract_type_alias(node: Node, source: &str) -> Option<GenericTypeDef> {
            let name = find_identifier(node, source)?;

            Some(GenericTypeDef {
                name,
                kind: "type".to_string(),
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
                fields: Vec::new(),
            })
        }

        visit_node_for_types(root_node, source, &mut types);
        Ok(types)
    }

    fn language(&self) -> Language {
        Language::MoonBit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moonbit_functions() {
        let mut parser = MoonBitParser::new().unwrap();
        let source = r#"
pub fn add(x : Int, y : Int) -> Int {
  x + y
}

fn helper(name : String) -> String {
  "Hello, " + name
}
"#;

        let functions = parser.extract_functions(source, "test.mbt").unwrap();
        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "add");
        assert_eq!(functions[0].parameters, vec!["x", "y"]);
        assert!(!functions[0].is_method);
        assert_eq!(functions[1].name, "helper");
        assert_eq!(functions[1].parameters, vec!["name"]);
    }

    #[test]
    fn test_moonbit_structs() {
        let mut parser = MoonBitParser::new().unwrap();
        let source = r#"
pub struct Point {
  x : Int
  y : Int
}

struct Color {
  r : Int
  g : Int
  b : Int
}
"#;

        let types = parser.extract_types(source, "test.mbt").unwrap();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].name, "Point");
        assert_eq!(types[0].kind, "struct");
        assert_eq!(types[0].fields, vec!["x", "y"]);
        assert_eq!(types[1].name, "Color");
        assert_eq!(types[1].fields, vec!["r", "g", "b"]);
    }

    #[test]
    fn test_moonbit_enums() {
        let mut parser = MoonBitParser::new().unwrap();
        let source = r#"
pub enum Color {
  Red
  Green
  Blue
  Custom(Int, Int, Int)
}
"#;

        let types = parser.extract_types(source, "test.mbt").unwrap();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name, "Color");
        assert_eq!(types[0].kind, "enum");
        assert_eq!(types[0].fields, vec!["Red", "Green", "Blue", "Custom"]);
    }

    #[test]
    fn test_moonbit_traits() {
        let mut parser = MoonBitParser::new().unwrap();
        let source = r#"
trait Show {
  to_string(Self) -> String
}

trait Eq {
  op_equal(Self, Self) -> Bool
}
"#;

        let types = parser.extract_types(source, "test.mbt").unwrap();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].name, "Show");
        assert_eq!(types[0].kind, "trait");
        assert_eq!(types[0].fields, vec!["to_string"]);
        assert_eq!(types[1].name, "Eq");
        assert_eq!(types[1].fields, vec!["op_equal"]);
    }

    #[test]
    fn test_moonbit_impl_methods() {
        let mut parser = MoonBitParser::new().unwrap();
        let source = r#"
pub struct Point {
  x : Int
  y : Int
}

impl Show for Point with to_string(self) {
  "Point(" + self.x.to_string() + ", " + self.y.to_string() + ")"
}
"#;

        let functions = parser.extract_functions(source, "test.mbt").unwrap();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "to_string");
        assert!(functions[0].is_method);
        assert_eq!(functions[0].class_name, Some("Point".to_string()));
        assert_eq!(functions[0].parameters, vec!["self"]);
    }

    #[test]
    fn test_moonbit_parse_tree() {
        let mut parser = MoonBitParser::new().unwrap();
        let source = r#"
fn add(x : Int, y : Int) -> Int {
  x + y
}
"#;
        let tree = parser.parse(source, "test.mbt");
        assert!(tree.is_ok());
    }
}
