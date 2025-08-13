use similarity_core::language_parser::{
    GenericFunctionDef, GenericTypeDef, Language, LanguageParser,
};
use similarity_core::tree::TreeNode;
use std::error::Error;
use std::rc::Rc;
use tree_sitter::{Node, Parser};

pub struct PhpParser {
    parser: Parser,
}

impl PhpParser {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_php::LANGUAGE_PHP.into())?;

        Ok(Self { parser })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn convert_node(&self, node: Node, source: &str, id_counter: &mut usize) -> TreeNode {
        let current_id = *id_counter;
        *id_counter += 1;

        let label = node.kind().to_string();
        let value = match node.kind() {
            "name" | "variable_name" | "string" | "integer" | "float" | "true" | "false"
            | "null" => node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
            _ => "".to_string(),
        };

        let mut tree_node = TreeNode::new(label, value, current_id);

        for child in node.children(&mut node.walk()) {
            let child_node = self.convert_node(child, source, id_counter);
            tree_node.add_child(Rc::new(child_node));
        }

        tree_node
    }

    fn extract_functions_from_node(
        &self,
        node: Node,
        source: &str,
        class_name: Option<&str>,
        namespace: Option<&str>,
    ) -> Vec<GenericFunctionDef> {
        let mut functions = Vec::new();

        // Check if there's a namespace declaration at the root level
        let mut current_namespace: Option<String> = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "namespace_definition" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    if let Ok(ns_name) = name_node.utf8_text(source.as_bytes()) {
                        current_namespace = Some(ns_name.to_string());
                        break;
                    }
                }
            }
        }

        fn visit_node(
            node: Node,
            source: &str,
            functions: &mut Vec<GenericFunctionDef>,
            class_name: Option<&str>,
            namespace: Option<&str>,
        ) {
            match node.kind() {
                "function_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                            let parameters_node = node.child_by_field_name("parameters");
                            let body_node = node.child_by_field_name("body");

                            let params = extract_params(parameters_node, source);
                            let full_name = if let Some(ns) = namespace {
                                format!("{ns}\\{name}")
                            } else {
                                name.to_string()
                            };

                            functions.push(GenericFunctionDef {
                                name: full_name,
                                start_line: node.start_position().row as u32 + 1,
                                end_line: node.end_position().row as u32 + 1,
                                body_start_line: body_node
                                    .map(|n| n.start_position().row as u32 + 1)
                                    .unwrap_or(0),
                                body_end_line: body_node
                                    .map(|n| n.end_position().row as u32 + 1)
                                    .unwrap_or(0),
                                parameters: params,
                                is_method: class_name.is_some(),
                                class_name: class_name.map(|s| s.to_string()),
                                is_async: false, // PHP doesn't have async/await syntax
                                is_generator: is_generator_function(node, source),
                                decorators: Vec::new(), // PHP doesn't have decorators like Python
                            });
                        }
                    }
                }
                "method_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                            let parameters_node = node.child_by_field_name("parameters");
                            let body_node = node.child_by_field_name("body");

                            let params = extract_params(parameters_node, source);
                            let visibility = extract_visibility(node, source);
                            let is_static = is_static_method(node, source);
                            let is_abstract = is_abstract_method(node, source);

                            let method_name = format!("{}::{}", class_name.unwrap_or(""), name);

                            functions.push(GenericFunctionDef {
                                name: method_name,
                                start_line: node.start_position().row as u32 + 1,
                                end_line: node.end_position().row as u32 + 1,
                                body_start_line: body_node
                                    .map(|n| n.start_position().row as u32 + 1)
                                    .unwrap_or(0),
                                body_end_line: body_node
                                    .map(|n| n.end_position().row as u32 + 1)
                                    .unwrap_or(0),
                                parameters: params,
                                is_method: true,
                                class_name: class_name.map(|s| s.to_string()),
                                is_async: false,
                                is_generator: is_generator_function(node, source),
                                decorators: vec![
                                    visibility,
                                    if is_static { "static".to_string() } else { "".to_string() },
                                    if is_abstract {
                                        "abstract".to_string()
                                    } else {
                                        "".to_string()
                                    },
                                ]
                                .into_iter()
                                .filter(|s| !s.is_empty())
                                .collect(),
                            });
                        }
                    }
                }
                "class_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                            let mut subcursor = node.walk();
                            for child in node.children(&mut subcursor) {
                                visit_node(child, source, functions, Some(name), namespace);
                            }
                        }
                    }
                }
                "namespace_definition" => {
                    // For namespace without braces (like "namespace App\Controllers;")
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Ok(_ns_name) = name_node.utf8_text(source.as_bytes()) {
                            // Continue traversing sibling nodes with this namespace
                        }
                    }
                }
                "namespace_use_declaration" => {
                    // Skip use statements
                }
                _ => {
                    let mut subcursor = node.walk();
                    for child in node.children(&mut subcursor) {
                        visit_node(child, source, functions, class_name, namespace);
                    }
                }
            }
        }

        fn is_generator_function(node: Node, source: &str) -> bool {
            if let Some(body) = node.child_by_field_name("body") {
                if let Ok(body_text) = body.utf8_text(source.as_bytes()) {
                    return body_text.contains("yield");
                }
            }
            false
        }

        fn extract_visibility(node: Node, source: &str) -> String {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "visibility_modifier" {
                    if let Ok(visibility) = child.utf8_text(source.as_bytes()) {
                        return visibility.to_string();
                    }
                }
            }
            "public".to_string() // Default visibility in PHP
        }

        fn is_static_method(node: Node, _source: &str) -> bool {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "static_modifier" {
                    return true;
                }
            }
            false
        }

        fn is_abstract_method(node: Node, _source: &str) -> bool {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "abstract_modifier" {
                    return true;
                }
            }
            false
        }

        fn extract_params(params_node: Option<Node>, source: &str) -> Vec<String> {
            if let Some(node) = params_node {
                let mut params = Vec::new();
                let mut cursor = node.walk();

                for child in node.children(&mut cursor) {
                    match child.kind() {
                        "simple_parameter" => {
                            if let Some(var_node) = child.child_by_field_name("name") {
                                if let Ok(param_text) = var_node.utf8_text(source.as_bytes()) {
                                    params.push(param_text.to_string());
                                }
                            }
                        }
                        "typed_parameter" => {
                            if let Some(var_node) = child.child_by_field_name("name") {
                                if let Ok(param_text) = var_node.utf8_text(source.as_bytes()) {
                                    params.push(param_text.to_string());
                                }
                            }
                        }
                        "variadic_parameter" => {
                            if let Some(var_node) = child.child_by_field_name("name") {
                                if let Ok(param_text) = var_node.utf8_text(source.as_bytes()) {
                                    params.push(format!("..{param_text}"));
                                }
                            }
                        }
                        _ => {}
                    }
                }

                params
            } else {
                Vec::new()
            }
        }

        let final_namespace = current_namespace.as_deref().or(namespace);
        visit_node(node, source, &mut functions, class_name, final_namespace);
        functions
    }
}

impl LanguageParser for PhpParser {
    fn parse(
        &mut self,
        source: &str,
        _filename: &str,
    ) -> Result<Rc<TreeNode>, Box<dyn Error + Send + Sync>> {
        let tree =
            self.parser.parse(source, None).ok_or_else(|| -> Box<dyn Error + Send + Sync> {
                "Failed to parse PHP source".into()
            })?;

        let root_node = tree.root_node();
        let mut id_counter = 0;
        Ok(Rc::new(self.convert_node(root_node, source, &mut id_counter)))
    }

    fn extract_functions(
        &mut self,
        source: &str,
        _filename: &str,
    ) -> Result<Vec<GenericFunctionDef>, Box<dyn Error + Send + Sync>> {
        let tree =
            self.parser.parse(source, None).ok_or_else(|| -> Box<dyn Error + Send + Sync> {
                "Failed to parse PHP source".into()
            })?;

        let root_node = tree.root_node();
        Ok(self.extract_functions_from_node(root_node, source, None, None))
    }

    fn extract_types(
        &mut self,
        source: &str,
        _filename: &str,
    ) -> Result<Vec<GenericTypeDef>, Box<dyn Error + Send + Sync>> {
        let tree =
            self.parser.parse(source, None).ok_or_else(|| -> Box<dyn Error + Send + Sync> {
                "Failed to parse PHP source".into()
            })?;

        let root_node = tree.root_node();
        let mut types = Vec::new();

        fn visit_node_for_types(node: Node, source: &str, types: &mut Vec<GenericTypeDef>) {
            match node.kind() {
                "class_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                            types.push(GenericTypeDef {
                                name: name.to_string(),
                                kind: "class".to_string(),
                                start_line: node.start_position().row as u32 + 1,
                                end_line: node.end_position().row as u32 + 1,
                                fields: extract_class_properties(node, source),
                            });
                        }
                    }
                }
                "interface_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                            types.push(GenericTypeDef {
                                name: name.to_string(),
                                kind: "interface".to_string(),
                                start_line: node.start_position().row as u32 + 1,
                                end_line: node.end_position().row as u32 + 1,
                                fields: extract_interface_methods(node, source),
                            });
                        }
                    }
                }
                "trait_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                            types.push(GenericTypeDef {
                                name: name.to_string(),
                                kind: "trait".to_string(),
                                start_line: node.start_position().row as u32 + 1,
                                end_line: node.end_position().row as u32 + 1,
                                fields: extract_trait_methods(node, source),
                            });
                        }
                    }
                }
                _ => {}
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                visit_node_for_types(child, source, types);
            }
        }

        fn extract_class_properties(node: Node, source: &str) -> Vec<String> {
            let mut properties = Vec::new();

            if let Some(body) = node.child_by_field_name("body") {
                let mut cursor = body.walk();
                for child in body.children(&mut cursor) {
                    if child.kind() == "property_declaration" {
                        let mut prop_cursor = child.walk();
                        for prop_child in child.children(&mut prop_cursor) {
                            if prop_child.kind() == "variable_name" {
                                if let Ok(prop_name) = prop_child.utf8_text(source.as_bytes()) {
                                    properties.push(prop_name.to_string());
                                }
                            }
                        }
                    }
                }
            }

            properties
        }

        fn extract_interface_methods(node: Node, source: &str) -> Vec<String> {
            let mut methods = Vec::new();

            if let Some(body) = node.child_by_field_name("body") {
                let mut cursor = body.walk();
                for child in body.children(&mut cursor) {
                    if child.kind() == "method_declaration" {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(method_name) = name_node.utf8_text(source.as_bytes()) {
                                methods.push(method_name.to_string());
                            }
                        }
                    }
                }
            }

            methods
        }

        fn extract_trait_methods(node: Node, source: &str) -> Vec<String> {
            let mut methods = Vec::new();

            if let Some(body) = node.child_by_field_name("body") {
                let mut cursor = body.walk();
                for child in body.children(&mut cursor) {
                    if child.kind() == "method_declaration" {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            if let Ok(method_name) = name_node.utf8_text(source.as_bytes()) {
                                methods.push(method_name.to_string());
                            }
                        }
                    }
                }
            }

            methods
        }

        visit_node_for_types(root_node, source, &mut types);
        Ok(types)
    }

    fn language(&self) -> Language {
        Language::Php
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_functions() {
        let mut parser = PhpParser::new().unwrap();
        let source = r#"
<?php
function hello($name) {
    return "Hello, " . $name . "!";
}

function add($a, $b = 0) {
    return $a + $b;
}

class Calculator {
    public function __construct() {
        $this->result = 0;
    }
    
    public function add($x) {
        $this->result += $x;
        return $this->result;
    }
    
    private static function multiply($a, $b) {
        return $a * $b;
    }
}
"#;

        let functions = parser.extract_functions(source, "test.php").unwrap();
        assert!(functions.len() >= 4);

        let function_names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        assert!(function_names.contains(&"hello"));
        assert!(function_names.contains(&"add"));
        assert!(function_names.contains(&"Calculator::__construct"));
        assert!(function_names.contains(&"Calculator::add"));
        assert!(function_names.contains(&"Calculator::multiply"));
    }

    #[test]
    fn test_php_classes() {
        let mut parser = PhpParser::new().unwrap();
        let source = r#"
<?php
class User {
    public $name;
    private $email;
    
    public function __construct($name, $email) {
        $this->name = $name;
        $this->email = $email;
    }
}

interface UserInterface {
    public function getName();
}

trait Loggable {
    public function log($message) {
        echo $message;
    }
}
"#;

        let types = parser.extract_types(source, "test.php").unwrap();
        assert_eq!(types.len(), 3);
        assert_eq!(types[0].name, "User");
        assert_eq!(types[0].kind, "class");
        assert_eq!(types[1].name, "UserInterface");
        assert_eq!(types[1].kind, "interface");
        assert_eq!(types[2].name, "Loggable");
        assert_eq!(types[2].kind, "trait");
    }

    #[test]
    fn test_php_namespace() {
        let mut parser = PhpParser::new().unwrap();
        let source = r#"
<?php
namespace App\Controllers;

function processRequest() {
    return "processed";
}

class UserController {
    public function index() {
        return "user list";
    }
}
"#;

        let functions = parser.extract_functions(source, "test.php").unwrap();
        assert!(functions.len() >= 2);

        let function_names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        assert!(function_names.contains(&"App\\Controllers\\processRequest"));
        assert!(function_names.contains(&"UserController::index"));
    }

    #[test]
    fn test_php_class_detection() {
        let mut parser = PhpParser::new().unwrap();
        let source = r#"
<?php
class TestClass {
    public function method1() {
        return "test1";
    }
    
    public function method2() {
        return "test2";
    }
}

function standalone_function() {
    return "standalone";
}
"#;

        let functions = parser.extract_functions(source, "test.php").unwrap();
        assert_eq!(functions.len(), 3);

        // Check class methods have class_name set
        let method1 = functions.iter().find(|f| f.name.contains("method1")).unwrap();
        let method2 = functions.iter().find(|f| f.name.contains("method2")).unwrap();
        let standalone = functions.iter().find(|f| f.name == "standalone_function").unwrap();

        assert_eq!(method1.class_name, Some("TestClass".to_string()));
        assert_eq!(method2.class_name, Some("TestClass".to_string()));
        assert_eq!(standalone.class_name, None);

        assert!(method1.is_method);
        assert!(method2.is_method);
        assert!(!standalone.is_method);
    }
}
