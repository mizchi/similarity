use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Expression, PropertyKey, Statement, TSInterfaceDeclaration, TSPropertySignature, TSType,
    TSTypeAliasDeclaration, VariableDeclarator,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub kind: TypeKind,
    pub properties: Vec<PropertyDefinition>,
    pub generics: Vec<String>,
    pub extends: Vec<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub file_path: String,
}
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Interface,
    TypeAlias,
    TypeLiteral,
}

#[derive(Debug, Clone)]
pub struct PropertyDefinition {
    pub name: String,
    pub type_annotation: String,
    pub optional: bool,
    pub readonly: bool,
}

#[derive(Debug, Clone)]
pub struct TypeLiteralDefinition {
    pub name: String, // Function name, variable name, etc.
    pub context: TypeLiteralContext,
    pub properties: Vec<PropertyDefinition>,
    pub start_line: usize,
    pub end_line: usize,
    pub file_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeLiteralContext {
    FunctionReturn(String),            // Function name
    FunctionParameter(String, String), // Function name, parameter name
    VariableDeclaration(String),       // Variable name
    ArrowFunctionReturn(String),       // Variable name for arrow function
}

pub struct TypeExtractor {
    source_text: String,
    file_path: String,
    line_offsets: Vec<usize>,
}

impl TypeExtractor {
    pub fn new(source_text: String, file_path: String) -> Self {
        let line_offsets = Self::calculate_line_offsets(&source_text);
        Self { source_text, file_path, line_offsets }
    }

    fn calculate_line_offsets(source: &str) -> Vec<usize> {
        let mut offsets = vec![0];
        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                offsets.push(i + 1);
            }
        }
        offsets
    }

    fn get_line_number(&self, offset: usize) -> usize {
        match self.line_offsets.binary_search(&offset) {
            Ok(line) => line + 1,
            Err(line) => line,
        }
    }

    pub fn extract_types(&self) -> Result<Vec<TypeDefinition>, String> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(&self.file_path).unwrap_or(SourceType::tsx());
        let ret = Parser::new(&allocator, &self.source_text, source_type).parse();

        if !ret.errors.is_empty() {
            // Create a more readable error message
            let error_messages: Vec<String> =
                ret.errors.iter().map(|e| e.message.to_string()).collect();
            return Err(format!("Parse errors: {}", error_messages.join(", ")));
        }

        let mut types = Vec::new();

        for stmt in &ret.program.body {
            match stmt {
                Statement::TSInterfaceDeclaration(interface) => {
                    if let Some(type_def) = self.extract_interface(interface) {
                        types.push(type_def);
                    }
                }
                Statement::TSTypeAliasDeclaration(type_alias) => {
                    if let Some(type_def) = self.extract_type_alias(type_alias) {
                        types.push(type_def);
                    }
                }
                _ => {}
            }
        }

        Ok(types)
    }

    pub fn extract_type_literals(&self) -> Result<Vec<TypeLiteralDefinition>, String> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(&self.file_path).unwrap_or(SourceType::tsx());
        let ret = Parser::new(&allocator, &self.source_text, source_type).parse();

        if !ret.errors.is_empty() {
            // Create a more readable error message
            let error_messages: Vec<String> =
                ret.errors.iter().map(|e| e.message.to_string()).collect();
            return Err(format!("Parse errors: {}", error_messages.join(", ")));
        }

        let mut type_literals = Vec::new();

        for stmt in &ret.program.body {
            self.extract_type_literals_from_statement(stmt, &mut type_literals);
        }

        Ok(type_literals)
    }

    fn extract_interface(&self, interface: &TSInterfaceDeclaration) -> Option<TypeDefinition> {
        let name = interface.id.name.as_str().to_string();
        let start_line = self.get_line_number(interface.span.start as usize);
        let end_line = self.get_line_number(interface.span.end as usize);

        let properties = self.extract_interface_properties(&interface.body.body);
        let generics = self.extract_generics(interface.type_parameters.as_ref());
        let extends = self.extract_extends(Some(&interface.extends));

        Some(TypeDefinition {
            name,
            kind: TypeKind::Interface,
            properties,
            generics,
            extends,
            start_line,
            end_line,
            file_path: self.file_path.clone(),
        })
    }

    fn extract_type_alias(&self, type_alias: &TSTypeAliasDeclaration) -> Option<TypeDefinition> {
        let name = type_alias.id.name.as_str().to_string();
        let start_line = self.get_line_number(type_alias.span.start as usize);
        let end_line = self.get_line_number(type_alias.span.end as usize);

        let properties = self.extract_type_properties(&type_alias.type_annotation);
        let generics = self.extract_generics(type_alias.type_parameters.as_ref());

        Some(TypeDefinition {
            name,
            kind: TypeKind::TypeAlias,
            properties,
            generics,
            extends: Vec::new(), // Type aliases don't have extends
            start_line,
            end_line,
            file_path: self.file_path.clone(),
        })
    }

    fn extract_interface_properties(
        &self,
        signatures: &[oxc_ast::ast::TSSignature],
    ) -> Vec<PropertyDefinition> {
        let mut properties = Vec::new();

        for signature in signatures {
            match signature {
                oxc_ast::ast::TSSignature::TSPropertySignature(prop_sig) => {
                    if let Some(prop_def) = self.extract_property_from_signature(prop_sig) {
                        properties.push(prop_def);
                    }
                }
                oxc_ast::ast::TSSignature::TSMethodSignature(method_sig) => {
                    if let Some(prop_def) = self.extract_method_from_signature(method_sig) {
                        properties.push(prop_def);
                    }
                }
                _ => {}
            }
        }

        properties
    }

    fn extract_type_properties(&self, ts_type: &TSType) -> Vec<PropertyDefinition> {
        match ts_type {
            TSType::TSTypeLiteral(type_literal) => {
                self.extract_interface_properties(&type_literal.members)
            }
            _ => Vec::new(), // For non-object types, return empty properties
        }
    }

    fn extract_property_from_signature(
        &self,
        prop_sig: &TSPropertySignature,
    ) -> Option<PropertyDefinition> {
        let name = match &prop_sig.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.as_str().to_string(),
            PropertyKey::StringLiteral(str_lit) => str_lit.value.as_str().to_string(),
            _ => return None,
        };

        let type_annotation = prop_sig
            .type_annotation
            .as_ref()
            .map(|ta| self.extract_type_string(&ta.type_annotation))
            .unwrap_or_else(|| "any".to_string());

        Some(PropertyDefinition {
            name,
            type_annotation,
            optional: prop_sig.optional,
            readonly: prop_sig.readonly,
        })
    }

    fn extract_method_from_signature(
        &self,
        method_sig: &oxc_ast::ast::TSMethodSignature,
    ) -> Option<PropertyDefinition> {
        let name = match &method_sig.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.as_str().to_string(),
            PropertyKey::StringLiteral(str_lit) => str_lit.value.as_str().to_string(),
            _ => return None,
        };

        // Extract method signature as a function type string
        let params = self.extract_function_params(&method_sig.params);
        let return_type = method_sig
            .return_type
            .as_ref()
            .map(|rt| self.extract_type_string(&rt.type_annotation))
            .unwrap_or_else(|| "void".to_string());

        let type_annotation = format!("({}) => {}", params, return_type);

        Some(PropertyDefinition {
            name,
            type_annotation,
            optional: method_sig.optional,
            readonly: false,
        })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn extract_type_string(&self, ts_type: &TSType) -> String {
        match ts_type {
            TSType::TSStringKeyword(_) => "string".to_string(),
            TSType::TSNumberKeyword(_) => "number".to_string(),
            TSType::TSBooleanKeyword(_) => "boolean".to_string(),
            TSType::TSAnyKeyword(_) => "any".to_string(),
            TSType::TSUnknownKeyword(_) => "unknown".to_string(),
            TSType::TSVoidKeyword(_) => "void".to_string(),
            TSType::TSNullKeyword(_) => "null".to_string(),
            TSType::TSUndefinedKeyword(_) => "undefined".to_string(),
            TSType::TSTypeReference(type_ref) => match &type_ref.type_name {
                oxc_ast::ast::TSTypeName::IdentifierReference(ident) => {
                    ident.name.as_str().to_string()
                }
                _ => "unknown".to_string(),
            },
            TSType::TSArrayType(array_type) => {
                let element_type = self.extract_type_string(&array_type.element_type);
                format!("{element_type}[]")
            }
            TSType::TSUnionType(union_type) => {
                let types: Vec<String> =
                    union_type.types.iter().map(|t| self.extract_type_string(t)).collect();
                types.join(" | ")
            }
            TSType::TSIntersectionType(intersection_type) => {
                let types: Vec<String> =
                    intersection_type.types.iter().map(|t| self.extract_type_string(t)).collect();
                types.join(" & ")
            }
            TSType::TSLiteralType(literal_type) => match &literal_type.literal {
                oxc_ast::ast::TSLiteral::StringLiteral(str_lit) => {
                    format!("\"{}\"", str_lit.value.as_str())
                }
                oxc_ast::ast::TSLiteral::NumericLiteral(num_lit) => num_lit.value.to_string(),
                oxc_ast::ast::TSLiteral::BooleanLiteral(bool_lit) => bool_lit.value.to_string(),
                _ => "unknown".to_string(),
            },
            TSType::TSFunctionType(func_type) => {
                let params = self.extract_function_params(&func_type.params);
                let return_type = self.extract_type_string(&func_type.return_type.type_annotation);
                format!("({}) => {}", params, return_type)
            }
            TSType::TSTypeLiteral(_) => "object".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn extract_function_params(&self, params: &oxc_ast::ast::FormalParameters) -> String {
        let param_strings: Vec<String> = params
            .items
            .iter()
            .map(|param| {
                let param_name = match &param.pattern.kind {
                    oxc_ast::ast::BindingPatternKind::BindingIdentifier(ident) => {
                        ident.name.as_str()
                    }
                    _ => "_",
                };

                let param_type = param
                    .pattern
                    .type_annotation
                    .as_ref()
                    .map(|ta| self.extract_type_string(&ta.type_annotation))
                    .unwrap_or_else(|| "any".to_string());

                format!("{}: {}", param_name, param_type)
            })
            .collect();

        param_strings.join(", ")
    }

    fn extract_generics(
        &self,
        type_params: Option<&oxc_allocator::Box<oxc_ast::ast::TSTypeParameterDeclaration>>,
    ) -> Vec<String> {
        if let Some(params) = type_params {
            params.params.iter().map(|param| param.name.name.as_str().to_string()).collect()
        } else {
            Vec::new()
        }
    }

    fn extract_extends(
        &self,
        extends: Option<&oxc_allocator::Vec<oxc_ast::ast::TSInterfaceHeritage>>,
    ) -> Vec<String> {
        if let Some(heritage_clauses) = extends {
            heritage_clauses
                .iter()
                .filter_map(|heritage| match &heritage.expression {
                    oxc_ast::ast::Expression::Identifier(ident) => {
                        Some(ident.name.as_str().to_string())
                    }
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn extract_type_literals_from_statement(
        &self,
        stmt: &Statement,
        type_literals: &mut Vec<TypeLiteralDefinition>,
    ) {
        match stmt {
            Statement::FunctionDeclaration(func) => {
                if let Some(name) = &func.id {
                    // Extract return type literal
                    if let Some(return_type) = &func.return_type {
                        if let Some(type_literal) = self.extract_type_literal_from_ts_type(
                            &return_type.type_annotation,
                            TypeLiteralContext::FunctionReturn(name.name.to_string()),
                        ) {
                            type_literals.push(type_literal);
                        }
                    }

                    // Extract parameter type literals
                    for param in &func.params.items {
                        if let Some(param_name) = self.get_parameter_name(param) {
                            if let Some(type_annotation) = &param.pattern.type_annotation {
                                if let Some(type_literal) = self.extract_type_literal_from_ts_type(
                                    &type_annotation.type_annotation,
                                    TypeLiteralContext::FunctionParameter(
                                        name.name.to_string(),
                                        param_name,
                                    ),
                                ) {
                                    type_literals.push(type_literal);
                                }
                            }
                        }
                    }
                }
            }
            Statement::VariableDeclaration(var_decl) => {
                for declarator in &var_decl.declarations {
                    if let Some(var_name) = self.get_variable_name(declarator) {
                        // Check for variable type annotation
                        if let Some(type_annotation) = &declarator.id.type_annotation {
                            if let Some(type_literal) = self.extract_type_literal_from_ts_type(
                                &type_annotation.type_annotation,
                                TypeLiteralContext::VariableDeclaration(var_name.clone()),
                            ) {
                                type_literals.push(type_literal);
                            }
                        }

                        // Check for arrow function in variable initialization
                        if let Some(init) = &declarator.init {
                            if let Some(type_literal) = self.extract_type_literal_from_expression(
                                init,
                                TypeLiteralContext::ArrowFunctionReturn(var_name),
                            ) {
                                type_literals.push(type_literal);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn extract_type_literal_from_ts_type(
        &self,
        ts_type: &TSType,
        context: TypeLiteralContext,
    ) -> Option<TypeLiteralDefinition> {
        match ts_type {
            TSType::TSTypeLiteral(type_literal) => {
                let properties = self.extract_interface_properties(&type_literal.members);
                let start_line = self.get_line_number(type_literal.span.start as usize);
                let end_line = self.get_line_number(type_literal.span.end as usize);

                Some(TypeLiteralDefinition {
                    name: self.get_context_name(&context),
                    context,
                    properties,
                    start_line,
                    end_line,
                    file_path: self.file_path.clone(),
                })
            }
            _ => None,
        }
    }

    fn extract_type_literal_from_expression(
        &self,
        expr: &Expression,
        context: TypeLiteralContext,
    ) -> Option<TypeLiteralDefinition> {
        match expr {
            Expression::ArrowFunctionExpression(arrow_func) => {
                if let Some(return_type) = &arrow_func.return_type {
                    self.extract_type_literal_from_ts_type(&return_type.type_annotation, context)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn get_context_name(&self, context: &TypeLiteralContext) -> String {
        match context {
            TypeLiteralContext::FunctionReturn(name) => format!("{name} (return type)"),
            TypeLiteralContext::FunctionParameter(func_name, param_name) => {
                format!("{func_name} (parameter: {param_name})")
            }
            TypeLiteralContext::VariableDeclaration(name) => format!("{name} (variable)"),
            TypeLiteralContext::ArrowFunctionReturn(name) => format!("{} (arrow function)", name),
        }
    }

    fn get_parameter_name(&self, param: &oxc_ast::ast::FormalParameter) -> Option<String> {
        match &param.pattern.kind {
            oxc_ast::ast::BindingPatternKind::BindingIdentifier(ident) => {
                Some(ident.name.to_string())
            }
            _ => None,
        }
    }

    fn get_variable_name(&self, declarator: &VariableDeclarator) -> Option<String> {
        match &declarator.id.kind {
            oxc_ast::ast::BindingPatternKind::BindingIdentifier(ident) => {
                Some(ident.name.to_string())
            }
            _ => None,
        }
    }
}

/// Extract types from source code
pub fn extract_types_from_code(
    source_text: &str,
    file_path: &str,
) -> Result<Vec<TypeDefinition>, String> {
    let extractor = TypeExtractor::new(source_text.to_string(), file_path.to_string());
    extractor.extract_types()
}

/// Extract types from multiple files
pub fn extract_types_from_files(
    files: &[(String, String)], // (file_path, content)
) -> HashMap<String, Vec<TypeDefinition>> {
    let mut results = HashMap::new();

    for (file_path, content) in files {
        match extract_types_from_code(content, file_path) {
            Ok(types) => {
                results.insert(file_path.clone(), types);
            }
            Err(err) => {
                eprintln!("Failed to extract types from {}: {}", file_path, err);
                results.insert(file_path.clone(), Vec::new());
            }
        }
    }

    results
}

/// Extract type literals from source code
pub fn extract_type_literals_from_code(
    source_text: &str,
    file_path: &str,
) -> Result<Vec<TypeLiteralDefinition>, String> {
    let extractor = TypeExtractor::new(source_text.to_string(), file_path.to_string());
    extractor.extract_type_literals()
}

/// Extract type literals from multiple files
pub fn extract_type_literals_from_files(
    files: &[(String, String)], // (file_path, content)
) -> HashMap<String, Vec<TypeLiteralDefinition>> {
    let mut results = HashMap::new();

    for (file_path, content) in files {
        match extract_type_literals_from_code(content, file_path) {
            Ok(type_literals) => {
                results.insert(file_path.clone(), type_literals);
            }
            Err(err) => {
                eprintln!("Failed to extract type literals from {}: {}", file_path, err);
                results.insert(file_path.clone(), Vec::new());
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_interface() {
        let source = r#"
interface User {
    id: string;
    name: string;
    age?: number;
    readonly email: string;
}
"#;

        let types = extract_types_from_code(source, "test.ts").unwrap();
        assert_eq!(types.len(), 1);

        let user_type = &types[0];
        assert_eq!(user_type.name, "User");
        assert_eq!(user_type.kind, TypeKind::Interface);
        assert_eq!(user_type.properties.len(), 4);

        let id_prop = &user_type.properties[0];
        assert_eq!(id_prop.name, "id");
        assert_eq!(id_prop.type_annotation, "string");
        assert!(!id_prop.optional);
        assert!(!id_prop.readonly);

        let age_prop = &user_type.properties[2];
        assert_eq!(age_prop.name, "age");
        assert_eq!(age_prop.type_annotation, "number");
        assert!(age_prop.optional);

        let email_prop = &user_type.properties[3];
        assert_eq!(email_prop.name, "email");
        assert!(email_prop.readonly);
    }

    #[test]
    fn test_extract_type_alias() {
        let source = r#"
type Status = "active" | "inactive" | "pending";

type User = {
    id: string;
    name: string;
};
"#;

        let types = extract_types_from_code(source, "test.ts").unwrap();
        assert_eq!(types.len(), 2);

        let status_type = &types[0];
        assert_eq!(status_type.name, "Status");
        assert_eq!(status_type.kind, TypeKind::TypeAlias);

        let user_type = &types[1];
        assert_eq!(user_type.name, "User");
        assert_eq!(user_type.kind, TypeKind::TypeAlias);
        assert_eq!(user_type.properties.len(), 2);
    }

    #[test]
    fn test_extract_generic_interface() {
        let source = r#"
interface Container<T> {
    value: T;
}
"#;

        let types = extract_types_from_code(source, "test.ts").unwrap();
        assert_eq!(types.len(), 1);

        let container_type = &types[0];
        assert_eq!(container_type.name, "Container");
        assert_eq!(container_type.generics, vec!["T"]);
    }

    #[test]
    fn test_extract_interface_with_extends() {
        let source = r#"
interface BaseUser {
    id: string;
}

interface User extends BaseUser {
    name: string;
}
"#;

        let types = extract_types_from_code(source, "test.ts").unwrap();
        assert_eq!(types.len(), 2);

        let user_type = &types[1];
        assert_eq!(user_type.name, "User");
        assert_eq!(user_type.extends, vec!["BaseUser"]);
    }
}
