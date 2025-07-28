use similarity_core::tree::TreeNode;
use tree_sitter::{Node, Parser};
use std::rc::Rc;

pub fn parse_css_to_tree(content: &str, _file_path: &str) -> Result<Vec<Rc<TreeNode>>, String> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_css::LANGUAGE.into()).unwrap();
    
    let tree = parser
        .parse(content, None)
        .ok_or_else(|| "Failed to parse CSS".to_string())?;
        
    let root_node = tree.root_node();
    let mut trees = Vec::new();
    
    extract_rule_trees(&root_node, content, &mut trees);
    
    Ok(trees)
}

pub fn parse_scss_to_tree(content: &str, _file_path: &str) -> Result<Vec<Rc<TreeNode>>, String> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_scss::language()).unwrap();
    
    let tree = parser
        .parse(content, None)
        .ok_or_else(|| "Failed to parse SCSS".to_string())?;
        
    let root_node = tree.root_node();
    let mut trees = Vec::new();
    
    extract_rule_trees(&root_node, content, &mut trees);
    
    Ok(trees)
}

fn extract_rule_trees(node: &Node, source: &str, trees: &mut Vec<Rc<TreeNode>>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "rule_set" | "ruleset" => {
                if let Some(tree) = rule_to_tree(&child, source) {
                    trees.push(tree);
                }
            }
            "media_statement" | "supports_statement" | "at_rule" => {
                if let Some(tree) = at_rule_to_tree(&child, source) {
                    trees.push(tree);
                }
            }
            "mixin_statement" => {
                if let Some(tree) = mixin_to_tree(&child, source) {
                    trees.push(tree);
                }
            }
            _ => {
                extract_rule_trees(&child, source, trees);
            }
        }
    }
}

fn rule_to_tree(node: &Node, source: &str) -> Option<Rc<TreeNode>> {
    let selector_node = node.child_by_field_name("selectors")?;
    let selector_text = selector_node.utf8_text(source.as_bytes()).ok()?;
    
    let mut children = Vec::new();
    if let Some(block) = node.child_by_field_name("block") {
        extract_declarations(&block, source, &mut children);
    }
    
    let mut tree_node = TreeNode::new(
        "rule".to_string(),
        selector_text.to_string(),
        0
    );
    tree_node.children = children;
    Some(Rc::new(tree_node))
}

fn at_rule_to_tree(node: &Node, source: &str) -> Option<Rc<TreeNode>> {
    let at_keyword = node.child_by_field_name("at_keyword")
        .or_else(|| node.child(0))
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())?;
        
    let mut children = Vec::new();
    if let Some(block) = node.child_by_field_name("block") {
        extract_rule_trees(&block, source, &mut children);
    }
    
    let mut tree_node = TreeNode::new(
        "at-rule".to_string(),
        at_keyword.to_string(),
        0
    );
    tree_node.children = children;
    Some(Rc::new(tree_node))
}

fn mixin_to_tree(node: &Node, source: &str) -> Option<Rc<TreeNode>> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(source.as_bytes()).ok()?;
    
    let mut children = Vec::new();
    if let Some(block) = node.child_by_field_name("block") {
        extract_declarations(&block, source, &mut children);
    }
    
    let mut tree_node = TreeNode::new(
        "mixin".to_string(),
        format!("@mixin {name}"),
        0
    );
    tree_node.children = children;
    Some(Rc::new(tree_node))
}

fn extract_declarations(node: &Node, source: &str, declarations: &mut Vec<Rc<TreeNode>>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "declaration" {
            if let Some(tree) = declaration_to_tree(&child, source) {
                declarations.push(tree);
            }
        } else if child.kind() == "block" {
            extract_declarations(&child, source, declarations);
        }
    }
}

fn declaration_to_tree(node: &Node, source: &str) -> Option<Rc<TreeNode>> {
    let property = node.child_by_field_name("property")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())?;
        
    let value = node.child_by_field_name("value")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("");
        
    let important = node.child_by_field_name("important").is_some();
    let important_suffix = if important { " !important" } else { "" };
    
    let tree_node = TreeNode::new(
        "declaration".to_string(),
        format!("{property}: {value}{important_suffix}"),
        0
    );
    Some(Rc::new(tree_node))
}