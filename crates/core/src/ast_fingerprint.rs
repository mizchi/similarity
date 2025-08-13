use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BinaryExpression, BlockStatement, CallExpression, ClassElement, Expression, FormalParameter,
    FunctionBody, IfStatement, Program, Statement, VariableDeclaration, VariableDeclarator,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::HashMap;

/// AST-based fingerprint for function similarity pre-filtering
#[derive(Debug, Clone, Default)]
pub struct AstFingerprint {
    /// Count of different AST node types
    node_counts: HashMap<&'static str, u32>,
    /// Bloom filter bits for quick comparison
    bloom_bits: u128,
}

impl AstFingerprint {
    pub fn new() -> Self {
        Self { node_counts: HashMap::new(), bloom_bits: 0 }
    }

    /// Create fingerprint from source code
    pub fn from_source(source: &str) -> Result<Self, String> {
        let allocator = Allocator::default();
        let source_type = SourceType::tsx(); // Default to TSX for maximum compatibility
        let ret = Parser::new(&allocator, source, source_type).parse();

        if !ret.errors.is_empty() {
            // Create a more readable error message
            let error_messages: Vec<String> =
                ret.errors.iter().map(|e| e.message.to_string()).collect();
            return Err(format!("Parse errors: {}", error_messages.join(", ")));
        }

        let mut fingerprint = Self::new();
        fingerprint.visit_program(&ret.program);
        Ok(fingerprint)
    }

    /// Visit program and count node types
    fn visit_program(&mut self, program: &Program) {
        self.count_node("Program");
        for stmt in &program.body {
            self.visit_statement(stmt);
        }
    }

    /// Visit statement and count node types
    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::FunctionDeclaration(func) => {
                self.count_node("FunctionDeclaration");
                // Process function parameters
                for param in &func.params.items {
                    self.visit_formal_parameter(param);
                }
                // Process function body
                if let Some(body) = &func.body {
                    self.visit_function_body(body);
                }
            }
            Statement::VariableDeclaration(var_decl) => {
                self.count_node("VariableDeclaration");
                self.visit_variable_declaration(var_decl);
            }
            Statement::ExpressionStatement(expr_stmt) => {
                self.count_node("ExpressionStatement");
                self.visit_expression(&expr_stmt.expression);
            }
            Statement::BlockStatement(block) => {
                self.count_node("BlockStatement");
                self.visit_block_statement(block);
            }
            Statement::IfStatement(if_stmt) => {
                self.count_node("IfStatement");
                self.visit_if_statement(if_stmt);
            }
            Statement::ReturnStatement(ret_stmt) => {
                self.count_node("ReturnStatement");
                if let Some(arg) = &ret_stmt.argument {
                    self.visit_expression(arg);
                }
            }
            Statement::ForStatement(for_stmt) => {
                self.count_node("ForStatement");
                // Skip init processing for now - API is unclear
                if let Some(test) = &for_stmt.test {
                    self.visit_expression(test);
                }
                if let Some(update) = &for_stmt.update {
                    self.visit_expression(update);
                }
                self.visit_statement(&for_stmt.body);
            }
            Statement::WhileStatement(while_stmt) => {
                self.count_node("WhileStatement");
                self.visit_expression(&while_stmt.test);
                self.visit_statement(&while_stmt.body);
            }
            Statement::DoWhileStatement(do_while_stmt) => {
                self.count_node("DoWhileStatement");
                self.visit_statement(&do_while_stmt.body);
                self.visit_expression(&do_while_stmt.test);
            }
            Statement::SwitchStatement(switch_stmt) => {
                self.count_node("SwitchStatement");
                self.visit_expression(&switch_stmt.discriminant);
            }
            Statement::TryStatement(_) => {
                self.count_node("TryStatement");
            }
            Statement::ThrowStatement(throw_stmt) => {
                self.count_node("ThrowStatement");
                self.visit_expression(&throw_stmt.argument);
            }
            Statement::ClassDeclaration(class) => {
                self.count_node("ClassDeclaration");
                for element in &class.body.body {
                    self.visit_class_element(element);
                }
            }
            _ => {
                self.count_node("Statement");
            }
        }
    }

    /// Visit expression and count node types
    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(_) => {
                self.count_node("Identifier");
            }
            Expression::StringLiteral(_) => {
                self.count_node("StringLiteral");
            }
            Expression::NumericLiteral(_) => {
                self.count_node("NumericLiteral");
            }
            Expression::BooleanLiteral(_) => {
                self.count_node("BooleanLiteral");
            }
            Expression::NullLiteral(_) => {
                self.count_node("NullLiteral");
            }
            Expression::BinaryExpression(bin_expr) => {
                self.count_node("BinaryExpression");
                self.visit_binary_expression(bin_expr);
            }
            Expression::UnaryExpression(unary_expr) => {
                self.count_node("UnaryExpression");
                self.visit_expression(&unary_expr.argument);
            }
            Expression::CallExpression(call_expr) => {
                self.count_node("CallExpression");
                self.visit_call_expression(call_expr);
            }
            Expression::ComputedMemberExpression(member_expr) => {
                self.count_node("MemberExpression");
                self.visit_expression(&member_expr.object);
                self.visit_expression(&member_expr.expression);
            }
            Expression::StaticMemberExpression(member_expr) => {
                self.count_node("MemberExpression");
                self.visit_expression(&member_expr.object);
            }
            Expression::PrivateFieldExpression(member_expr) => {
                self.count_node("MemberExpression");
                self.visit_expression(&member_expr.object);
            }
            Expression::ArrayExpression(array_expr) => {
                self.count_node("ArrayExpression");
                for element in &array_expr.elements {
                    if let Some(expr) = element.as_expression() {
                        self.visit_expression(expr);
                    }
                }
            }
            Expression::ObjectExpression(obj_expr) => {
                self.count_node("ObjectExpression");
                for prop in &obj_expr.properties {
                    match &prop {
                        oxc_ast::ast::ObjectPropertyKind::ObjectProperty(p) => {
                            self.visit_expression(&p.value);
                        }
                        oxc_ast::ast::ObjectPropertyKind::SpreadProperty(p) => {
                            self.visit_expression(&p.argument);
                        }
                    }
                }
            }
            Expression::ArrowFunctionExpression(arrow_func) => {
                self.count_node("ArrowFunctionExpression");
                for param in &arrow_func.params.items {
                    self.visit_formal_parameter(param);
                }
                // Arrow function body is not optional
                self.visit_function_body(&arrow_func.body);
            }
            Expression::ConditionalExpression(cond_expr) => {
                self.count_node("ConditionalExpression");
                self.visit_expression(&cond_expr.test);
                self.visit_expression(&cond_expr.consequent);
                self.visit_expression(&cond_expr.alternate);
            }
            Expression::AssignmentExpression(assign_expr) => {
                self.count_node("AssignmentExpression");
                self.visit_expression(&assign_expr.right);
            }
            Expression::LogicalExpression(logical_expr) => {
                self.count_node("LogicalExpression");
                self.visit_expression(&logical_expr.left);
                self.visit_expression(&logical_expr.right);
            }
            Expression::NewExpression(new_expr) => {
                self.count_node("NewExpression");
                self.visit_expression(&new_expr.callee);
                for arg in &new_expr.arguments {
                    if let Some(expr) = arg.as_expression() {
                        self.visit_expression(expr);
                    }
                }
            }
            Expression::ThisExpression(_) => {
                self.count_node("ThisExpression");
            }
            Expression::TemplateLiteral(_) => {
                self.count_node("TemplateLiteral");
            }
            Expression::AwaitExpression(await_expr) => {
                self.count_node("AwaitExpression");
                self.visit_expression(&await_expr.argument);
            }
            _ => {
                self.count_node("Expression");
            }
        }
    }

    /// Visit variable declaration
    fn visit_variable_declaration(&mut self, var_decl: &VariableDeclaration) {
        for decl in &var_decl.declarations {
            self.visit_variable_declarator(decl);
        }
    }

    /// Visit variable declarator
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator) {
        self.count_node("VariableDeclarator");
        if let Some(init) = &decl.init {
            self.visit_expression(init);
        }
    }

    /// Visit block statement
    fn visit_block_statement(&mut self, block: &BlockStatement) {
        for stmt in &block.body {
            self.visit_statement(stmt);
        }
    }

    /// Visit if statement
    fn visit_if_statement(&mut self, if_stmt: &IfStatement) {
        self.visit_expression(&if_stmt.test);
        self.visit_statement(&if_stmt.consequent);
        if let Some(alternate) = &if_stmt.alternate {
            self.visit_statement(alternate);
        }
    }

    /// Visit binary expression
    fn visit_binary_expression(&mut self, bin_expr: &BinaryExpression) {
        // Count specific operators
        match bin_expr.operator {
            oxc_ast::ast::BinaryOperator::Addition => self.count_node("BinaryOp_Add"),
            oxc_ast::ast::BinaryOperator::Subtraction => self.count_node("BinaryOp_Sub"),
            oxc_ast::ast::BinaryOperator::Multiplication => self.count_node("BinaryOp_Mul"),
            oxc_ast::ast::BinaryOperator::Division => self.count_node("BinaryOp_Div"),
            oxc_ast::ast::BinaryOperator::Equality => self.count_node("BinaryOp_Eq"),
            oxc_ast::ast::BinaryOperator::Inequality => self.count_node("BinaryOp_Neq"),
            oxc_ast::ast::BinaryOperator::LessThan => self.count_node("BinaryOp_Lt"),
            oxc_ast::ast::BinaryOperator::GreaterThan => self.count_node("BinaryOp_Gt"),
            _ => self.count_node("BinaryOp_Other"),
        }
        self.visit_expression(&bin_expr.left);
        self.visit_expression(&bin_expr.right);
    }

    /// Visit call expression
    fn visit_call_expression(&mut self, call_expr: &CallExpression) {
        self.visit_expression(&call_expr.callee);
        for arg in &call_expr.arguments {
            if let Some(expr) = arg.as_expression() {
                self.visit_expression(expr);
            }
        }
    }

    /// Visit formal parameter
    fn visit_formal_parameter(&mut self, _param: &FormalParameter) {
        self.count_node("Parameter");
    }

    /// Visit function body
    fn visit_function_body(&mut self, body: &FunctionBody) {
        for stmt in &body.statements {
            self.visit_statement(stmt);
        }
    }

    /// Visit class element
    fn visit_class_element(&mut self, element: &ClassElement) {
        match element {
            ClassElement::MethodDefinition(method) => {
                self.count_node("MethodDefinition");
                if let Some(body) = &method.value.body {
                    self.visit_function_body(body);
                }
            }
            ClassElement::PropertyDefinition(_) => {
                self.count_node("PropertyDefinition");
            }
            _ => {
                self.count_node("ClassElement");
            }
        }
    }

    /// Count a node type and update bloom filter
    fn count_node(&mut self, node_type: &'static str) {
        *self.node_counts.entry(node_type).or_insert(0) += 1;

        // Update bloom filter with multiple hash functions
        let hash1 = simple_hash(node_type);
        let hash2 = simple_hash_2(node_type);
        let hash3 = simple_hash_3(node_type);

        self.bloom_bits |= 1u128 << (hash1 % 128);
        self.bloom_bits |= 1u128 << (hash2 % 128);
        self.bloom_bits |= 1u128 << (hash3 % 128);
    }

    /// Quick check if two fingerprints might be similar (very lenient)
    pub fn might_be_similar(&self, other: &Self, _threshold: f64) -> bool {
        // Check bloom filter overlap
        let overlap = (self.bloom_bits & other.bloom_bits).count_ones();
        let self_bits = self.bloom_bits.count_ones();
        let other_bits = other.bloom_bits.count_ones();

        // If either has no bits set, allow comparison
        if self_bits == 0 || other_bits == 0 {
            return true;
        }

        // If any overlap exists, allow comparison (very lenient)
        // This ensures identical functions always pass
        if overlap > 0 {
            return true;
        }

        // Only reject if there's absolutely no overlap
        false
    }

    /// Get bloom filter bits for SIMD comparison
    pub fn bloom_bits(&self) -> u128 {
        self.bloom_bits
    }

    /// Calculate detailed similarity between fingerprints
    pub fn similarity(&self, other: &Self) -> f64 {
        let mut total_diff = 0.0;
        let mut total_weight = 0.0;

        // Get all node types
        let mut all_nodes = std::collections::HashSet::new();
        all_nodes.extend(self.node_counts.keys());
        all_nodes.extend(other.node_counts.keys());

        for node_type in all_nodes {
            let count1 = *self.node_counts.get(node_type).unwrap_or(&0) as f64;
            let count2 = *other.node_counts.get(node_type).unwrap_or(&0) as f64;

            // Weight by importance of node type
            let weight = get_node_weight(node_type);

            if count1 > 0.0 || count2 > 0.0 {
                let max_count = count1.max(count2);
                let diff = (count1 - count2).abs() / max_count;
                total_diff += diff * weight;
                total_weight += weight;
            }
        }

        if total_weight == 0.0 {
            return 1.0;
        }

        1.0 - (total_diff / total_weight)
    }

    /// Get the count of a specific node type
    pub fn get_node_count(&self, node_type: &str) -> u32 {
        *self.node_counts.get(node_type).unwrap_or(&0)
    }

    /// Get all node counts
    pub fn node_counts(&self) -> &HashMap<&'static str, u32> {
        &self.node_counts
    }
}

/// Simple hash function 1
fn simple_hash(s: &str) -> u64 {
    let mut hash = 0u64;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

/// Simple hash function 2
fn simple_hash_2(s: &str) -> u64 {
    let mut hash = 0u64;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(37).wrapping_add(byte as u64);
    }
    hash
}

/// Simple hash function 3
fn simple_hash_3(s: &str) -> u64 {
    let mut hash = 0u64;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(41).wrapping_add(byte as u64);
    }
    hash
}

/// Get weight for node type (more important nodes have higher weight)
fn get_node_weight(node_type: &str) -> f64 {
    match node_type {
        // Control flow is very important
        "IfStatement" | "ForStatement" | "WhileStatement" | "DoWhileStatement" => 2.0,
        "SwitchStatement" | "ConditionalExpression" => 1.8,

        // Function-related nodes
        "FunctionDeclaration" | "ArrowFunctionExpression" | "MethodDefinition" => 1.5,
        "CallExpression" | "NewExpression" => 1.3,

        // Error handling
        "TryStatement" | "ThrowStatement" => 1.5,

        // Binary operations (differentiated)
        "BinaryOp_Add" | "BinaryOp_Sub" | "BinaryOp_Mul" | "BinaryOp_Div" => 1.2,
        "BinaryOp_Eq" | "BinaryOp_Neq" | "BinaryOp_Lt" | "BinaryOp_Gt" => 1.1,

        // Other important expressions
        "AssignmentExpression" | "LogicalExpression" => 1.0,
        "MemberExpression" | "ArrayExpression" | "ObjectExpression" => 0.9,

        // Variable declarations
        "VariableDeclaration" | "VariableDeclarator" => 0.8,

        // Literals and identifiers
        "Identifier" | "StringLiteral" | "NumericLiteral" | "BooleanLiteral" => 0.5,

        // Other nodes
        _ => 0.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_fingerprint_similarity() {
        let code1 = "function add(a, b) { return a + b; }";
        let code2 = "function sum(x, y) { return x + y; }";
        let code3 = "function multiply(a, b) { return a * b; }";

        let fp1 = AstFingerprint::from_source(code1).unwrap();
        let fp2 = AstFingerprint::from_source(code2).unwrap();
        let fp3 = AstFingerprint::from_source(code3).unwrap();

        // Same structure should have high similarity
        assert!(fp1.similarity(&fp2) > 0.9);

        // Different operation should have lower similarity
        assert!(fp1.similarity(&fp3) < 0.9);
        assert!(fp1.similarity(&fp3) > 0.5);
    }

    #[test]
    fn test_node_counting() {
        let code = r#"
            function test(a, b) {
                if (a > b) {
                    return a;
                } else {
                    return b;
                }
            }
        "#;

        let fp = AstFingerprint::from_source(code).unwrap();

        assert_eq!(fp.get_node_count("FunctionDeclaration"), 1);
        assert_eq!(fp.get_node_count("IfStatement"), 1);
        assert_eq!(fp.get_node_count("ReturnStatement"), 2);
        assert_eq!(fp.get_node_count("Parameter"), 2);
        assert!(fp.get_node_count("BinaryOp_Gt") > 0);
    }

    #[test]
    fn test_bloom_filter_overlap() {
        let code1 = "function test1() { if (x) { return x; } }";
        let code2 = "function test2() { if (y) { return y; } }";
        let code3 = "function test3() { while (true) { break; } }";

        let fp1 = AstFingerprint::from_source(code1).unwrap();
        let fp2 = AstFingerprint::from_source(code2).unwrap();
        let fp3 = AstFingerprint::from_source(code3).unwrap();

        // Similar code should pass the lenient check
        assert!(fp1.might_be_similar(&fp2, 0.5));

        // Different code might still pass (lenient)
        assert!(fp1.might_be_similar(&fp3, 0.5));
    }

    #[test]
    fn test_complex_function_fingerprint() {
        let code1 = r#"
            function processData(items) {
                const results = [];
                for (let i = 0; i < items.length; i++) {
                    if (items[i].value > 10) {
                        results.push(items[i].value * 2);
                    } else {
                        results.push(items[i].value);
                    }
                }
                return results;
            }
        "#;

        let code2 = r#"
            function handleData(elements) {
                const output = [];
                for (let j = 0; j < elements.length; j++) {
                    if (elements[j].val > 10) {
                        output.push(elements[j].val * 2);
                    } else {
                        output.push(elements[j].val);
                    }
                }
                return output;
            }
        "#;

        let fp1 = AstFingerprint::from_source(code1).unwrap();
        let fp2 = AstFingerprint::from_source(code2).unwrap();

        // Very similar structure should have high similarity
        assert!(fp1.similarity(&fp2) > 0.85);

        // Check specific node counts
        assert_eq!(fp1.get_node_count("ForStatement"), 1);
        assert_eq!(fp1.get_node_count("IfStatement"), 1);
        assert_eq!(fp1.get_node_count("BinaryOp_Gt"), 1);
        assert_eq!(fp1.get_node_count("BinaryOp_Lt"), 1);
        assert_eq!(fp1.get_node_count("BinaryOp_Mul"), 1);
        assert!(fp1.get_node_count("CallExpression") >= 2); // push calls
    }

    #[test]
    fn test_different_algorithms() {
        let bubble_sort = r#"
            function bubbleSort(arr) {
                for (let i = 0; i < arr.length; i++) {
                    for (let j = 0; j < arr.length - 1; j++) {
                        if (arr[j] > arr[j + 1]) {
                            const temp = arr[j];
                            arr[j] = arr[j + 1];
                            arr[j + 1] = temp;
                        }
                    }
                }
                return arr;
            }
        "#;

        let quick_sort = r#"
            function quickSort(arr) {
                if (arr.length <= 1) {
                    return arr;
                }
                const pivot = arr[0];
                const left = [];
                const right = [];
                for (let i = 1; i < arr.length; i++) {
                    if (arr[i] < pivot) {
                        left.push(arr[i]);
                    } else {
                        right.push(arr[i]);
                    }
                }
                return quickSort(left).concat(pivot, quickSort(right));
            }
        "#;

        let fp1 = AstFingerprint::from_source(bubble_sort).unwrap();
        let fp2 = AstFingerprint::from_source(quick_sort).unwrap();

        // Different algorithms should have lower similarity
        let similarity = fp1.similarity(&fp2);
        assert!(similarity < 0.7);
        assert!(similarity > 0.3); // But still some similarity (both sorting algorithms)

        // Bubble sort has nested loops
        assert_eq!(fp1.get_node_count("ForStatement"), 2);
        // Quick sort has recursion
        assert!(fp2.get_node_count("CallExpression") >= 3); // recursive calls and push
    }
}
