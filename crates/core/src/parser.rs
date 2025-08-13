use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPatternKind, BlockStatement, ClassElement, Expression, FormalParameter, FunctionBody,
    Program, PropertyKey, Statement, VariableDeclarator,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::rc::Rc;

use crate::tree::TreeNode;

/// Parse TypeScript code and convert to `TreeNode` structure
///
/// # Errors
///
/// Returns an error if parsing fails due to syntax errors
pub fn parse_and_convert_to_tree(
    filename: &str,
    source_text: &str,
) -> Result<Rc<TreeNode>, String> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(filename).unwrap_or(SourceType::tsx());
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    if !ret.errors.is_empty() {
        // Create a more readable error message
        let error_messages: Vec<String> =
            ret.errors.iter().map(|e| e.message.to_string()).collect();
        return Err(format!("Parse errors: {}", error_messages.join(", ")));
    }

    let mut id_counter = 0;
    Ok(ast_to_tree_node(&ret.program, &mut id_counter))
}

pub fn ast_to_tree_node(program: &Program, id_counter: &mut usize) -> Rc<TreeNode> {
    let mut root = TreeNode::new("Program".to_string(), "Program".to_string(), *id_counter);
    *id_counter += 1;

    for stmt in &program.body {
        if let Some(child) = statement_to_tree_node(stmt, id_counter) {
            root.add_child(child);
        }
    }

    Rc::new(root)
}

fn statement_to_tree_node(stmt: &Statement, id_counter: &mut usize) -> Option<Rc<TreeNode>> {
    match stmt {
        Statement::FunctionDeclaration(func) => {
            let label = func.id.as_ref().map_or("Function", |id| id.name.as_str()).to_string();
            let mut node = TreeNode::new(label, "FunctionDeclaration".to_string(), *id_counter);
            *id_counter += 1;

            // Add parameters
            for param in &func.params.items {
                if let Some(param_node) = formal_parameter_to_tree_node(param, id_counter) {
                    node.add_child(param_node);
                }
            }

            // Add body
            if let Some(body) = &func.body {
                if let Some(body_node) = function_body_to_tree_node(body, id_counter) {
                    node.add_child(body_node);
                }
            }

            Some(Rc::new(node))
        }
        Statement::ClassDeclaration(class) => {
            let label = class.id.as_ref().map_or("Class", |id| id.name.as_str()).to_string();
            let mut node = TreeNode::new(label, "ClassDeclaration".to_string(), *id_counter);
            *id_counter += 1;

            // Add class body elements
            for element in &class.body.body {
                if let Some(elem_node) = class_element_to_tree_node(element, id_counter) {
                    node.add_child(elem_node);
                }
            }

            Some(Rc::new(node))
        }
        Statement::VariableDeclaration(var_decl) => {
            let mut node = TreeNode::new(
                "VariableDeclaration".to_string(),
                "VariableDeclaration".to_string(),
                *id_counter,
            );
            *id_counter += 1;

            for decl in &var_decl.declarations {
                if let Some(decl_node) = variable_declarator_to_tree_node(decl, id_counter) {
                    node.add_child(decl_node);
                }
            }

            Some(Rc::new(node))
        }
        Statement::ExpressionStatement(expr_stmt) => {
            expression_to_tree_node(&expr_stmt.expression, id_counter)
        }
        Statement::BlockStatement(block) => block_statement_to_tree_node(block, id_counter),
        Statement::IfStatement(if_stmt) => {
            let mut node =
                TreeNode::new("IfStatement".to_string(), "IfStatement".to_string(), *id_counter);
            *id_counter += 1;

            // Add test expression
            if let Some(test_node) = expression_to_tree_node(&if_stmt.test, id_counter) {
                node.add_child(test_node);
            }

            // Add consequent
            if let Some(cons_node) = statement_to_tree_node(&if_stmt.consequent, id_counter) {
                node.add_child(cons_node);
            }

            // Add alternate if exists
            if let Some(alt) = &if_stmt.alternate {
                if let Some(alt_node) = statement_to_tree_node(alt, id_counter) {
                    node.add_child(alt_node);
                }
            }

            Some(Rc::new(node))
        }
        Statement::ReturnStatement(ret_stmt) => {
            let mut node = TreeNode::new(
                "ReturnStatement".to_string(),
                "ReturnStatement".to_string(),
                *id_counter,
            );
            *id_counter += 1;

            if let Some(arg) = &ret_stmt.argument {
                if let Some(arg_node) = expression_to_tree_node(arg, id_counter) {
                    node.add_child(arg_node);
                }
            }

            Some(Rc::new(node))
        }
        _ => {
            // For other statement types, create a generic node
            let node = TreeNode::new("Statement".to_string(), "Statement".to_string(), *id_counter);
            *id_counter += 1;
            Some(Rc::new(node))
        }
    }
}

fn expression_to_tree_node(expr: &Expression, id_counter: &mut usize) -> Option<Rc<TreeNode>> {
    match expr {
        Expression::Identifier(ident) => {
            let node = TreeNode::new(
                ident.name.as_str().to_string(),
                "Identifier".to_string(),
                *id_counter,
            );
            *id_counter += 1;
            Some(Rc::new(node))
        }
        Expression::StringLiteral(str_lit) => {
            let label = format!("\"{}\"", str_lit.value.as_str());
            let node = TreeNode::new(label, "StringLiteral".to_string(), *id_counter);
            *id_counter += 1;
            Some(Rc::new(node))
        }
        Expression::NumericLiteral(num_lit) => {
            let label = num_lit.value.to_string();
            let node = TreeNode::new(label, "NumericLiteral".to_string(), *id_counter);
            *id_counter += 1;
            Some(Rc::new(node))
        }
        Expression::BooleanLiteral(bool_lit) => {
            let label = bool_lit.value.to_string();
            let node = TreeNode::new(label, "BooleanLiteral".to_string(), *id_counter);
            *id_counter += 1;
            Some(Rc::new(node))
        }
        Expression::BinaryExpression(bin_expr) => {
            let mut node = TreeNode::new(
                format!("{:?}", bin_expr.operator),
                "BinaryExpression".to_string(),
                *id_counter,
            );
            *id_counter += 1;

            if let Some(left_node) = expression_to_tree_node(&bin_expr.left, id_counter) {
                node.add_child(left_node);
            }

            if let Some(right_node) = expression_to_tree_node(&bin_expr.right, id_counter) {
                node.add_child(right_node);
            }

            Some(Rc::new(node))
        }
        Expression::CallExpression(call_expr) => {
            let mut node = TreeNode::new(
                "CallExpression".to_string(),
                "CallExpression".to_string(),
                *id_counter,
            );
            *id_counter += 1;

            if let Some(callee_node) = expression_to_tree_node(&call_expr.callee, id_counter) {
                node.add_child(callee_node);
            }

            for arg in &call_expr.arguments {
                if let Some(expr) = arg.as_expression() {
                    if let Some(arg_node) = expression_to_tree_node(expr, id_counter) {
                        node.add_child(arg_node);
                    }
                }
            }

            Some(Rc::new(node))
        }
        Expression::ArrowFunctionExpression(arrow) => {
            let mut node = TreeNode::new(
                "ArrowFunction".to_string(),
                "ArrowFunctionExpression".to_string(),
                *id_counter,
            );
            *id_counter += 1;

            // Add parameters
            for param in &arrow.params.items {
                if let Some(param_node) = formal_parameter_to_tree_node(param, id_counter) {
                    node.add_child(param_node);
                }
            }

            // Add body
            if arrow.expression {
                // Expression body (e.g., => x + 1)
                if let Some(Statement::ExpressionStatement(expr_stmt)) =
                    arrow.body.statements.first()
                {
                    if let Some(expr_node) =
                        expression_to_tree_node(&expr_stmt.expression, id_counter)
                    {
                        node.add_child(expr_node);
                    }
                }
            } else {
                // Block body (e.g., => { return x + 1; })
                if let Some(body_node) = function_body_to_tree_node(&arrow.body, id_counter) {
                    node.add_child(body_node);
                }
            }

            Some(Rc::new(node))
        }
        _ => {
            // For other expression types, create a generic node
            let node =
                TreeNode::new("Expression".to_string(), "Expression".to_string(), *id_counter);
            *id_counter += 1;
            Some(Rc::new(node))
        }
    }
}

fn formal_parameter_to_tree_node(
    param: &FormalParameter,
    id_counter: &mut usize,
) -> Option<Rc<TreeNode>> {
    let label = match &param.pattern.kind {
        BindingPatternKind::BindingIdentifier(ident) => ident.name.as_str().to_string(),
        _ => "Parameter".to_string(),
    };
    let node = TreeNode::new(label, "Parameter".to_string(), *id_counter);
    *id_counter += 1;
    Some(Rc::new(node))
}

fn function_body_to_tree_node(body: &FunctionBody, id_counter: &mut usize) -> Option<Rc<TreeNode>> {
    let mut node =
        TreeNode::new("BlockStatement".to_string(), "BlockStatement".to_string(), *id_counter);
    *id_counter += 1;

    for stmt in &body.statements {
        if let Some(stmt_node) = statement_to_tree_node(stmt, id_counter) {
            node.add_child(stmt_node);
        }
    }

    Some(Rc::new(node))
}

fn block_statement_to_tree_node(
    block: &BlockStatement,
    id_counter: &mut usize,
) -> Option<Rc<TreeNode>> {
    let mut node =
        TreeNode::new("BlockStatement".to_string(), "BlockStatement".to_string(), *id_counter);
    *id_counter += 1;

    for stmt in &block.body {
        if let Some(stmt_node) = statement_to_tree_node(stmt, id_counter) {
            node.add_child(stmt_node);
        }
    }

    Some(Rc::new(node))
}

fn variable_declarator_to_tree_node(
    decl: &VariableDeclarator,
    id_counter: &mut usize,
) -> Option<Rc<TreeNode>> {
    let label = match &decl.id.kind {
        BindingPatternKind::BindingIdentifier(ident) => ident.name.as_str().to_string(),
        _ => "Variable".to_string(),
    };
    let mut node = TreeNode::new(label, "VariableDeclarator".to_string(), *id_counter);
    *id_counter += 1;

    if let Some(init) = &decl.init {
        if let Some(init_node) = expression_to_tree_node(init, id_counter) {
            node.add_child(init_node);
        }
    }

    Some(Rc::new(node))
}

fn class_element_to_tree_node(
    element: &ClassElement,
    id_counter: &mut usize,
) -> Option<Rc<TreeNode>> {
    match element {
        ClassElement::MethodDefinition(method) => {
            let label = match &method.key {
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str().to_string(),
                PropertyKey::PrivateIdentifier(ident) => format!("#{}", ident.name.as_str()),
                _ => "Method".to_string(),
            };
            let mut node = TreeNode::new(label, "MethodDefinition".to_string(), *id_counter);
            *id_counter += 1;

            // Add method body
            if let Some(body) = &method.value.body {
                if let Some(body_node) = function_body_to_tree_node(body, id_counter) {
                    node.add_child(body_node);
                }
            }

            Some(Rc::new(node))
        }
        ClassElement::PropertyDefinition(prop) => {
            let label = match &prop.key {
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str().to_string(),
                PropertyKey::PrivateIdentifier(ident) => format!("#{}", ident.name.as_str()),
                _ => "Property".to_string(),
            };
            let node = TreeNode::new(label, "PropertyDefinition".to_string(), *id_counter);
            *id_counter += 1;
            Some(Rc::new(node))
        }
        _ => None,
    }
}
