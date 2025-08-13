use oxc_ast::ast::*;
use oxc_span::Span;

use crate::parser::parse_and_convert_to_tree;
use crate::tsed::{calculate_tsed, TSEDOptions};

type CrossFileSimilarityResult = Vec<(String, SimilarityResult, String)>;

#[derive(Debug, Clone)]
pub struct SimilarityResult {
    pub func1: FunctionDefinition,
    pub func2: FunctionDefinition,
    pub similarity: f64,
    pub impact: u32, // Total lines that could be removed
}

impl SimilarityResult {
    pub fn new(func1: FunctionDefinition, func2: FunctionDefinition, similarity: f64) -> Self {
        // Impact is the smaller function's line count (since we'd remove the duplicate)
        let impact = func1.line_count().min(func2.line_count());
        SimilarityResult { func1, func2, similarity, impact }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub function_type: FunctionType,
    pub parameters: Vec<String>,
    pub body_span: Span,
    pub start_line: u32,
    pub end_line: u32,
    pub class_name: Option<String>,
    pub parent_function: Option<String>,
    pub node_count: Option<u32>,
}

impl FunctionDefinition {
    pub fn line_count(&self) -> u32 {
        self.end_line - self.start_line + 1
    }

    /// Check if this function is a parent or child of another function
    pub fn is_parent_child_relationship(&self, other: &FunctionDefinition) -> bool {
        // Check if 'other' is inside 'self' (self is parent of other)
        let other_inside_self = self.start_line <= other.start_line
            && self.end_line >= other.end_line
            && self.body_span.start < other.body_span.start
            && self.body_span.end > other.body_span.end;

        // Check if 'self' is inside 'other' (other is parent of self)
        let self_inside_other = other.start_line <= self.start_line
            && other.end_line >= self.end_line
            && other.body_span.start < self.body_span.start
            && other.body_span.end > self.body_span.end;

        other_inside_self || self_inside_other
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    Function,
    Method,
    Arrow,
    Constructor,
}

/// Extract all functions from TypeScript/JavaScript code
pub fn extract_functions(
    filename: &str,
    source_text: &str,
) -> Result<Vec<FunctionDefinition>, String> {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let allocator = Allocator::default();
    let source_type = SourceType::from_path(filename).unwrap_or(SourceType::tsx());
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    if !ret.errors.is_empty() {
        // Create a more readable error message
        let error_messages: Vec<String> =
            ret.errors.iter().map(|e| e.message.to_string()).collect();
        return Err(format!("Parse errors: {}", error_messages.join(", ")));
    }

    let mut functions = Vec::new();
    let mut context = ExtractionContext {
        functions: &mut functions,
        source_text,
        class_name: None,
        parent_function: None,
    };

    extract_from_program(&ret.program, &mut context);
    Ok(functions)
}

struct ExtractionContext<'a> {
    functions: &'a mut Vec<FunctionDefinition>,
    source_text: &'a str,
    class_name: Option<String>,
    parent_function: Option<String>,
}

fn extract_from_program(program: &Program, ctx: &mut ExtractionContext) {
    for stmt in &program.body {
        extract_from_statement(stmt, ctx);
    }
}

fn extract_from_statement(stmt: &Statement, ctx: &mut ExtractionContext) {
    match stmt {
        Statement::FunctionDeclaration(func) => {
            if let Some(name) = &func.id {
                let func_name = name.name.to_string();
                let params = extract_parameters(&func.params);
                ctx.functions.push(FunctionDefinition {
                    name: func_name.clone(),
                    function_type: FunctionType::Function,
                    parameters: params,
                    body_span: func.span,
                    start_line: get_line_number(func.span.start, ctx.source_text),
                    end_line: get_line_number(func.span.end, ctx.source_text),
                    class_name: None,
                    parent_function: ctx.parent_function.clone(),
                    node_count: count_function_nodes(func.span, ctx.source_text),
                });

                // Extract nested functions within the function body
                if let Some(body) = &func.body {
                    let saved_parent = ctx.parent_function.clone();
                    ctx.parent_function = Some(func_name);
                    extract_from_function_body(body, ctx);
                    ctx.parent_function = saved_parent;
                }
            }
        }
        Statement::ClassDeclaration(class) => {
            let class_name = class.id.as_ref().map(|id| id.name.to_string());
            let saved_class_name = ctx.class_name.clone();
            ctx.class_name = class_name.clone();

            for element in &class.body.body {
                if let ClassElement::MethodDefinition(method) = element {
                    let method_name = match &method.key {
                        PropertyKey::StaticIdentifier(ident) => ident.name.to_string(),
                        PropertyKey::PrivateIdentifier(ident) => format!("#{}", ident.name),
                        _ => "anonymous".to_string(),
                    };

                    let params = extract_parameters(&method.value.params);
                    let function_type = if method.kind == MethodDefinitionKind::Constructor {
                        FunctionType::Constructor
                    } else {
                        FunctionType::Method
                    };

                    let method_full_name = if let Some(ref class) = class_name {
                        format!("{class}.{method_name}")
                    } else {
                        method_name.clone()
                    };

                    ctx.functions.push(FunctionDefinition {
                        name: method_name.clone(),
                        function_type,
                        parameters: params,
                        body_span: method.span,
                        start_line: get_line_number(method.span.start, ctx.source_text),
                        end_line: get_line_number(method.span.end, ctx.source_text),
                        class_name: class_name.clone(),
                        parent_function: ctx.parent_function.clone(),
                        node_count: count_function_nodes(method.span, ctx.source_text),
                    });

                    // Extract nested functions within method body
                    if let Some(body) = &method.value.body {
                        let saved_parent = ctx.parent_function.clone();
                        ctx.parent_function = Some(method_full_name);
                        extract_from_function_body(body, ctx);
                        ctx.parent_function = saved_parent;
                    }
                }
            }

            ctx.class_name = saved_class_name;
        }
        Statement::VariableDeclaration(var_decl) => {
            for decl in &var_decl.declarations {
                if let Some(Expression::ArrowFunctionExpression(arrow)) = &decl.init {
                    if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
                        let params = extract_parameters(&arrow.params);
                        let arrow_name = ident.name.to_string();
                        ctx.functions.push(FunctionDefinition {
                            name: arrow_name.clone(),
                            function_type: FunctionType::Arrow,
                            parameters: params,
                            body_span: arrow.span,
                            start_line: get_line_number(arrow.span.start, ctx.source_text),
                            end_line: get_line_number(arrow.span.end, ctx.source_text),
                            class_name: None,
                            parent_function: ctx.parent_function.clone(),
                            node_count: count_function_nodes(arrow.span, ctx.source_text),
                        });

                        // Extract nested functions within arrow function body
                        if !arrow.expression {
                            let saved_parent = ctx.parent_function.clone();
                            ctx.parent_function = Some(arrow_name);
                            extract_from_function_body(&arrow.body, ctx);
                            ctx.parent_function = saved_parent;
                        }
                    }
                }
            }
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(decl) = &export.declaration {
                extract_from_declaration(decl, ctx);
            }
        }
        Statement::ExportDefaultDeclaration(export) => {
            if let ExportDefaultDeclarationKind::FunctionDeclaration(func) = &export.declaration {
                let name = func
                    .id
                    .as_ref()
                    .map(|id| id.name.to_string())
                    .unwrap_or_else(|| "default".to_string());
                let params = extract_parameters(&func.params);
                let func_name = name.clone();
                ctx.functions.push(FunctionDefinition {
                    name: func_name.clone(),
                    function_type: FunctionType::Function,
                    parameters: params,
                    body_span: func.span,
                    start_line: get_line_number(func.span.start, ctx.source_text),
                    end_line: get_line_number(func.span.end, ctx.source_text),
                    class_name: None,
                    parent_function: ctx.parent_function.clone(),
                    node_count: count_function_nodes(func.span, ctx.source_text),
                });

                // Extract nested functions within the function body
                if let Some(body) = &func.body {
                    let saved_parent = ctx.parent_function.clone();
                    ctx.parent_function = Some(func_name);
                    extract_from_function_body(body, ctx);
                    ctx.parent_function = saved_parent;
                }
            }
        }
        _ => {}
    }
}

fn extract_from_declaration(decl: &Declaration, ctx: &mut ExtractionContext) {
    match decl {
        Declaration::FunctionDeclaration(func) => {
            if let Some(name) = &func.id {
                let func_name = name.name.to_string();
                let params = extract_parameters(&func.params);
                ctx.functions.push(FunctionDefinition {
                    name: func_name.clone(),
                    function_type: FunctionType::Function,
                    parameters: params,
                    body_span: func.span,
                    start_line: get_line_number(func.span.start, ctx.source_text),
                    end_line: get_line_number(func.span.end, ctx.source_text),
                    class_name: None,
                    parent_function: ctx.parent_function.clone(),
                    node_count: count_function_nodes(func.span, ctx.source_text),
                });

                // Extract nested functions within the function body
                if let Some(body) = &func.body {
                    let saved_parent = ctx.parent_function.clone();
                    ctx.parent_function = Some(func_name);
                    extract_from_function_body(body, ctx);
                    ctx.parent_function = saved_parent;
                }
            }
        }
        Declaration::ClassDeclaration(class) => {
            let class_name = class.id.as_ref().map(|id| id.name.to_string());
            let saved_class_name = ctx.class_name.clone();
            ctx.class_name = class_name.clone();

            for element in &class.body.body {
                if let ClassElement::MethodDefinition(method) = element {
                    let method_name = match &method.key {
                        PropertyKey::StaticIdentifier(ident) => ident.name.to_string(),
                        PropertyKey::PrivateIdentifier(ident) => format!("#{}", ident.name),
                        _ => "anonymous".to_string(),
                    };

                    let params = extract_parameters(&method.value.params);
                    let function_type = if method.kind == MethodDefinitionKind::Constructor {
                        FunctionType::Constructor
                    } else {
                        FunctionType::Method
                    };

                    let method_full_name = if let Some(ref class) = class_name {
                        format!("{class}.{method_name}")
                    } else {
                        method_name.clone()
                    };

                    ctx.functions.push(FunctionDefinition {
                        name: method_name.clone(),
                        function_type,
                        parameters: params,
                        body_span: method.span,
                        start_line: get_line_number(method.span.start, ctx.source_text),
                        end_line: get_line_number(method.span.end, ctx.source_text),
                        class_name: class_name.clone(),
                        parent_function: ctx.parent_function.clone(),
                        node_count: count_function_nodes(method.span, ctx.source_text),
                    });

                    // Extract nested functions within method body
                    if let Some(body) = &method.value.body {
                        let saved_parent = ctx.parent_function.clone();
                        ctx.parent_function = Some(method_full_name);
                        extract_from_function_body(body, ctx);
                        ctx.parent_function = saved_parent;
                    }
                }
            }

            ctx.class_name = saved_class_name;
        }
        Declaration::VariableDeclaration(var) => {
            for decl in &var.declarations {
                if let Some(Expression::ArrowFunctionExpression(arrow)) = &decl.init {
                    if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
                        let params = extract_parameters(&arrow.params);
                        let arrow_name = ident.name.to_string();
                        ctx.functions.push(FunctionDefinition {
                            name: arrow_name.clone(),
                            function_type: FunctionType::Arrow,
                            parameters: params,
                            body_span: arrow.span,
                            start_line: get_line_number(arrow.span.start, ctx.source_text),
                            end_line: get_line_number(arrow.span.end, ctx.source_text),
                            class_name: None,
                            parent_function: ctx.parent_function.clone(),
                            node_count: count_function_nodes(arrow.span, ctx.source_text),
                        });

                        // Extract nested functions within arrow function body
                        if !arrow.expression {
                            let saved_parent = ctx.parent_function.clone();
                            ctx.parent_function = Some(arrow_name);
                            extract_from_function_body(&arrow.body, ctx);
                            ctx.parent_function = saved_parent;
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn extract_parameters(params: &oxc_ast::ast::FormalParameters) -> Vec<String> {
    params
        .items
        .iter()
        .filter_map(|param| match &param.pattern.kind {
            BindingPatternKind::BindingIdentifier(ident) => Some(ident.name.to_string()),
            _ => None,
        })
        .collect()
}

fn extract_from_function_body(body: &FunctionBody, ctx: &mut ExtractionContext) {
    for stmt in &body.statements {
        extract_from_statement(stmt, ctx);
    }
}

fn get_line_number(offset: u32, source_text: &str) -> u32 {
    let mut line = 1;
    let mut current_offset = 0;

    for ch in source_text.chars() {
        if current_offset >= offset as usize {
            break;
        }
        if ch == '\n' {
            line += 1;
        }
        current_offset += ch.len_utf8();
    }

    line
}

/// Compare similarity between two functions
pub fn compare_functions(
    func1: &FunctionDefinition,
    func2: &FunctionDefinition,
    source1: &str,
    source2: &str,
    options: &TSEDOptions,
) -> Result<f64, String> {
    // Extract function body text
    let body1 = extract_body_text(func1, source1);
    let body2 = extract_body_text(func2, source2);

    // Parse and compare
    let tree1 = parse_and_convert_to_tree("func1.ts", &body1)?;
    let tree2 = parse_and_convert_to_tree("func2.ts", &body2)?;

    let mut similarity = calculate_tsed(&tree1, &tree2, options);

    // Apply size penalty for short functions if enabled
    if options.size_penalty {
        let avg_lines = (func1.line_count() + func2.line_count()) as f64 / 2.0;
        if avg_lines < 10.0 {
            // Apply penalty: shorter functions get more penalty
            let penalty = avg_lines / 10.0;
            similarity *= penalty;
        }
    }

    Ok(similarity)
}

fn extract_body_text(func: &FunctionDefinition, source: &str) -> String {
    let start = func.body_span.start as usize;
    let end = func.body_span.end as usize;
    source[start..end].to_string()
}

/// Count the number of AST nodes in a function body
fn count_function_nodes(body_span: Span, source_text: &str) -> Option<u32> {
    let start = body_span.start as usize;
    let end = body_span.end as usize;
    if start >= end || end > source_text.len() {
        return None;
    }

    let body_text = &source_text[start..end];

    // For now, try to parse the text as-is
    // If it fails, try wrapping it in a way that makes it valid TypeScript
    match parse_and_convert_to_tree("temp.ts", body_text) {
        Ok(tree) => Some(tree.get_subtree_size() as u32),
        Err(_) => {
            // If direct parsing fails, try wrapping in a minimal context
            // This handles cases like "constructor(private x: number) {}" or method definitions
            let wrapped = if body_text.starts_with("constructor") {
                format!("class C {{ {body_text} }}")
            } else if body_text.contains("(") && body_text.contains(")") && body_text.contains("{")
            {
                // Likely a method or function - wrap it appropriately
                if body_text.trim().starts_with(|c: char| c.is_alphabetic() || c == '_' || c == '#')
                {
                    // Method-like syntax
                    format!("class C {{ {body_text} }}")
                } else {
                    // Arrow function or other expression
                    format!("const x = {body_text}")
                }
            } else {
                // Default fallback
                body_text.to_string()
            };

            match parse_and_convert_to_tree("temp.ts", &wrapped) {
                Ok(tree) => {
                    // Subtract nodes added by wrapping (approximation)
                    let base_nodes = if wrapped.starts_with("class C") {
                        3 // class node + body node + wrapping
                    } else if wrapped.starts_with("const x") {
                        2 // const declaration + wrapping
                    } else {
                        0
                    };
                    Some((tree.get_subtree_size().saturating_sub(base_nodes)) as u32)
                }
                Err(_) => {
                    // If all else fails, make a rough estimate based on the text
                    // Count common syntax elements as a fallback
                    let node_count =
                        body_text.matches(['{', '}', '(', ')', ';']).count() as u32 + 1;
                    Some(node_count.max(1))
                }
            }
        }
    }
}

/// Find similar functions within the same file
pub fn find_similar_functions_in_file(
    filename: &str,
    source_text: &str,
    threshold: f64,
    options: &TSEDOptions,
) -> Result<Vec<SimilarityResult>, String> {
    let functions = extract_functions(filename, source_text)?;
    let mut similar_pairs = Vec::new();

    // Compare all pairs
    for i in 0..functions.len() {
        for j in (i + 1)..functions.len() {
            // Skip if either function is too short
            if let Some(min_tokens) = options.min_tokens {
                // If min_tokens is specified, use token count instead of line count
                let tokens_i = functions[i].node_count.unwrap_or(0);
                let tokens_j = functions[j].node_count.unwrap_or(0);
                if tokens_i < min_tokens || tokens_j < min_tokens {
                    continue;
                }
            } else {
                // Otherwise use line count
                if functions[i].line_count() < options.min_lines
                    || functions[j].line_count() < options.min_lines
                {
                    continue;
                }
            }

            // Skip if functions have parent-child relationship
            if functions[i].is_parent_child_relationship(&functions[j]) {
                continue;
            }

            let similarity =
                compare_functions(&functions[i], &functions[j], source_text, source_text, options)?;

            if similarity >= threshold {
                similar_pairs.push(SimilarityResult::new(
                    functions[i].clone(),
                    functions[j].clone(),
                    similarity,
                ));
            }
        }
    }

    // Sort by impact (descending), then by similarity (descending)
    similar_pairs.sort_by(|a, b| {
        b.impact
            .cmp(&a.impact)
            .then(b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal))
    });

    Ok(similar_pairs)
}

/// Find similar functions across multiple files
pub fn find_similar_functions_across_files(
    files: &[(String, String)], // (filename, source_text)
    threshold: f64,
    options: &TSEDOptions,
) -> Result<CrossFileSimilarityResult, String> {
    let mut all_functions = Vec::new();

    // Extract functions from all files
    for (filename, source) in files {
        let functions = extract_functions(filename, source)?;
        for func in functions {
            all_functions.push((filename.clone(), source.clone(), func));
        }
    }

    let mut similar_pairs = Vec::new();

    // Compare all pairs across files
    for i in 0..all_functions.len() {
        for j in (i + 1)..all_functions.len() {
            let (first_file, source1, func1) = &all_functions[i];
            let (second_file, source2, func2) = &all_functions[j];

            // Skip if same file (already handled by find_similar_functions_in_file)
            if first_file == second_file {
                continue;
            }

            // Skip if either function is too short
            if let Some(min_tokens) = options.min_tokens {
                // If min_tokens is specified, use token count instead of line count
                let tokens1 = func1.node_count.unwrap_or(0);
                let tokens2 = func2.node_count.unwrap_or(0);
                if tokens1 < min_tokens || tokens2 < min_tokens {
                    continue;
                }
            } else {
                // Otherwise use line count
                if func1.line_count() < options.min_lines || func2.line_count() < options.min_lines
                {
                    continue;
                }
            }

            // Skip if functions have parent-child relationship (across files)
            if func1.is_parent_child_relationship(func2) {
                continue;
            }

            let similarity = compare_functions(func1, func2, source1, source2, options)?;

            if similarity >= threshold {
                similar_pairs.push((
                    first_file.clone(),
                    SimilarityResult::new(func1.clone(), func2.clone(), similarity),
                    second_file.clone(),
                ));
            }
        }
    }

    // Sort by impact (descending), then by similarity (descending)
    similar_pairs.sort_by(|a, b| {
        b.1.impact
            .cmp(&a.1.impact)
            .then(b.1.similarity.partial_cmp(&a.1.similarity).unwrap_or(std::cmp::Ordering::Equal))
    });

    Ok(similar_pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_functions() {
        let code = r"
            function add(a: number, b: number): number {
                return a + b;
            }
            
            const multiply = (x: number, y: number) => x * y;
            
            class Calculator {
                constructor(private initial: number) {}
                
                add(value: number): number {
                    return this.initial + value;
                }
                
                subtract(value: number): number {
                    return this.initial - value;
                }
            }
            
            export function divide(a: number, b: number): number {
                return a / b;
            }
        ";

        let functions = extract_functions("test.ts", code).unwrap();

        assert_eq!(functions.len(), 6);

        // Check function names
        let names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"add"));
        assert!(names.contains(&"multiply"));
        assert!(names.contains(&"constructor"));
        assert!(names.contains(&"subtract"));
        assert!(names.contains(&"divide"));

        // Check function types
        let add_func =
            functions.iter().find(|f| f.name == "add" && f.class_name.is_none()).unwrap();
        assert_eq!(add_func.function_type, FunctionType::Function);
        assert_eq!(add_func.parameters, vec!["a", "b"]);

        let multiply_func = functions.iter().find(|f| f.name == "multiply").unwrap();
        assert_eq!(multiply_func.function_type, FunctionType::Arrow);

        let constructor = functions.iter().find(|f| f.name == "constructor").unwrap();
        assert_eq!(constructor.function_type, FunctionType::Constructor);
        assert_eq!(constructor.class_name, Some("Calculator".to_string()));

        // Check that node_count is populated for all functions
        for func in &functions {
            assert!(
                func.node_count.is_some(),
                "Function {} should have node_count populated",
                func.name
            );
            // Node count should be reasonable (greater than 0)
            assert!(
                func.node_count.unwrap() > 0,
                "Function {} should have positive node_count",
                func.name
            );
        }
    }

    #[test]
    fn test_node_count_calculation() {
        let code = r#"
            function simple() {
                return 42;
            }
            
            function complex(a: number, b: number): number {
                if (a > b) {
                    return a - b;
                } else {
                    return a + b;
                }
            }
        "#;

        let functions = extract_functions("test.ts", code).unwrap();

        let simple = functions.iter().find(|f| f.name == "simple").unwrap();
        let complex = functions.iter().find(|f| f.name == "complex").unwrap();

        println!("Simple function node count: {:?}", simple.node_count);
        println!("Complex function node count: {:?}", complex.node_count);

        // Simple function should have fewer nodes than complex
        assert!(simple.node_count.is_some());
        assert!(complex.node_count.is_some());
        assert!(simple.node_count.unwrap() < complex.node_count.unwrap());
    }

    #[test]
    fn test_find_similar_functions_in_file() {
        let code = r"
            function calculateSum(a: number, b: number): number {
                return a + b;
            }
            
            function addNumbers(x: number, y: number): number {
                return x + y;
            }
            
            function multiply(a: number, b: number): number {
                return a * b;
            }
            
            function computeSum(first: number, second: number): number {
                return first + second;
            }
        ";

        let mut options = TSEDOptions::default();
        options.apted_options.rename_cost = 0.3; // Lower rename cost for better similarity detection
        options.size_penalty = false; // Disable for test with small functions
        options.min_lines = 1; // Allow small functions in test

        let similar_pairs = find_similar_functions_in_file("test.ts", code, 0.7, &options).unwrap();

        // Should find that calculateSum, addNumbers, and computeSum are similar
        assert!(
            similar_pairs.len() >= 2,
            "Expected at least 2 similar pairs, found {}",
            similar_pairs.len()
        );

        // Note: multiply IS similar to others because they all have the same structure
        // (two parameters, single return statement). This is expected behavior.
        // Let's check that we found the expected similar pairs
        let sum_pairs = similar_pairs
            .iter()
            .filter(|result| {
                (result.func1.name.contains("Sum") || result.func2.name.contains("Sum"))
                    || (result.func1.name == "addNumbers" || result.func2.name == "addNumbers")
            })
            .count();
        assert!(sum_pairs >= 3, "Expected at least 3 pairs involving sum functions");
    }

    #[test]
    fn test_find_similar_functions_across_files() {
        let file1 = (
            "file1.ts".to_string(),
            r#"
            export function processUser(user: User): void {
                validateUser(user);
                saveUser(user);
                notifyUser(user);
            }
            
            function validateUser(user: User): boolean {
                return user.name.length > 0 && user.email.includes('@');
            }
        "#
            .to_string(),
        );

        let file2 = (
            "file2.ts".to_string(),
            r#"
            export function handleUser(u: User): void {
                checkUser(u);
                storeUser(u);
                alertUser(u);
            }
            
            function checkUser(u: User): boolean {
                return u.name.length > 0 && u.email.includes('@');
            }
        "#
            .to_string(),
        );

        let mut options = TSEDOptions::default();
        options.apted_options.rename_cost = 0.3;
        options.size_penalty = false; // Disable for test with small functions
        options.min_lines = 1; // Allow small functions in test

        let similar_pairs =
            find_similar_functions_across_files(&[file1, file2], 0.7, &options).unwrap();

        // Should find that processUser/handleUser and validateUser/checkUser are similar
        assert!(similar_pairs.len() >= 2);

        // Check specific pairs
        let process_handle = similar_pairs.iter().find(|(_, result, _)| {
            (result.func1.name == "processUser" && result.func2.name == "handleUser")
                || (result.func1.name == "handleUser" && result.func2.name == "processUser")
        });
        assert!(process_handle.is_some());

        let validate_check = similar_pairs.iter().find(|(_, result, _)| {
            (result.func1.name == "validateUser" && result.func2.name == "checkUser")
                || (result.func1.name == "checkUser" && result.func2.name == "validateUser")
        });
        assert!(validate_check.is_some());
    }
}
