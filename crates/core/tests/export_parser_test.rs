use std::rc::Rc;

use similarity_core::{
    find_function_overlaps, parse_and_convert_to_tree, OverlapOptions, TreeNode,
};

fn contains_value(node: &Rc<TreeNode>, value: &str) -> bool {
    node.value == value || node.children.iter().any(|child| contains_value(child, value))
}

fn contains_label(node: &Rc<TreeNode>, label: &str) -> bool {
    node.label == label || node.children.iter().any(|child| contains_label(child, label))
}

#[test]
fn parse_exported_function_declaration_preserves_function_shape() {
    let code = r#"
export function greet(name) {
    return name + "!";
}
"#;

    let tree = parse_and_convert_to_tree("test.ts", code).unwrap();
    let function = tree.children.first().expect("expected exported function");

    assert_eq!(function.label, "greet");
    assert_eq!(function.value, "FunctionDeclaration");
    assert_eq!(function.children.first().map(|child| child.label.as_str()), Some("name"));
    assert!(contains_value(function, "ReturnStatement"));
}

#[test]
fn parse_export_default_class_preserves_methods() {
    let code = r#"
export default class Greeter {
    greet(name) {
        return `Hello, ${name}`;
    }
}
"#;

    let tree = parse_and_convert_to_tree("test.ts", code).unwrap();
    let class = tree.children.first().expect("expected exported class");

    assert_eq!(class.label, "Greeter");
    assert_eq!(class.value, "ClassDeclaration");
    assert!(contains_label(class, "greet"));
    assert!(contains_value(class, "ReturnStatement"));
}

#[test]
fn parse_exported_variable_arrow_function_preserves_body() {
    let code = r#"
export const loadValue = () => {
    return 42;
};
"#;

    let tree = parse_and_convert_to_tree("test.ts", code).unwrap();
    let declaration = tree.children.first().expect("expected exported variable declaration");
    let variable = declaration.children.first().expect("expected exported variable");

    assert_eq!(declaration.value, "VariableDeclaration");
    assert_eq!(variable.label, "loadValue");
    assert!(contains_value(variable, "ArrowFunctionExpression"));
    assert!(contains_value(variable, "ReturnStatement"));
}

#[test]
fn overlap_detection_handles_exported_functions() {
    let code = r#"
export function validateUser(user) {
    if (!user.email) {
        throw new Error("Email is required");
    }
    if (!user.email.includes("@")) {
        throw new Error("Invalid email format");
    }
    return user.email.toLowerCase();
}

export function validateAdmin(admin) {
    if (!admin.email) {
        throw new Error("Email is required");
    }
    if (!admin.email.includes("@")) {
        throw new Error("Invalid email format");
    }
    return admin.email.toLowerCase();
}
"#;

    let options = OverlapOptions {
        min_window_size: 3,
        max_window_size: 25,
        threshold: 0.5,
        size_tolerance: 0.5,
    };

    let overlaps = find_function_overlaps(code, code, &options).unwrap();

    assert!(
        overlaps.iter().any(|overlap| overlap.similarity > 0.9),
        "expected exported functions to be compared with their real ASTs"
    );
}
