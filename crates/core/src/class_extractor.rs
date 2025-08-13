use oxc_allocator::Allocator;
use oxc_ast::ast::{ClassElement, MethodDefinitionKind, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

#[derive(Debug, Clone)]
pub struct ClassDefinition {
    pub name: String,
    pub properties: Vec<ClassProperty>,
    pub methods: Vec<ClassMethod>,
    pub constructor_params: Vec<String>,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub file_path: String,
    pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct ClassProperty {
    pub name: String,
    pub type_annotation: String,
    pub is_static: bool,
    pub is_private: bool,
    pub is_readonly: bool,
    pub is_optional: bool,
}

#[derive(Debug, Clone)]
pub struct ClassMethod {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: String,
    pub is_static: bool,
    pub is_private: bool,
    pub is_async: bool,
    pub is_generator: bool,
    pub kind: MethodKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MethodKind {
    Method,
    Getter,
    Setter,
    Constructor,
}

struct ClassExtractor {
    source_text: String,
    file_path: String,
    line_offsets: Vec<usize>,
}

impl ClassExtractor {
    fn new(source_text: String, file_path: String) -> Self {
        let line_offsets = Self::calculate_line_offsets(&source_text);
        Self {
            source_text,
            file_path,
            line_offsets,
        }
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

    fn extract_type_string(&self, type_annotation: &oxc_ast::ast::TSTypeAnnotation) -> String {
        use oxc_ast::ast::TSType;
        
        match &type_annotation.type_annotation {
            TSType::TSStringKeyword(_) => "string".to_string(),
            TSType::TSNumberKeyword(_) => "number".to_string(),
            TSType::TSBooleanKeyword(_) => "boolean".to_string(),
            TSType::TSAnyKeyword(_) => "any".to_string(),
            TSType::TSUnknownKeyword(_) => "unknown".to_string(),
            TSType::TSNeverKeyword(_) => "never".to_string(),
            TSType::TSVoidKeyword(_) => "void".to_string(),
            TSType::TSUndefinedKeyword(_) => "undefined".to_string(),
            TSType::TSNullKeyword(_) => "null".to_string(),
            TSType::TSArrayType(array) => {
                format!("{}[]", self.extract_type_string_from_ts_type(&array.element_type))
            }
            TSType::TSTypeReference(type_ref) => match &type_ref.type_name {
                oxc_ast::ast::TSTypeName::IdentifierReference(ident) => {
                    let base = ident.name.as_str();
                    if let Some(params) = &type_ref.type_arguments {
                        let param_strings: Vec<String> = params
                            .params
                            .iter()
                            .map(|p| self.extract_type_string_from_ts_type(p))
                            .collect();
                        format!("{}<{}>", base, param_strings.join(", "))
                    } else {
                        base.to_string()
                    }
                }
                _ => "unknown".to_string(),
            }
            TSType::TSUnionType(union) => {
                let types: Vec<String> = union
                    .types
                    .iter()
                    .map(|t| self.extract_type_string_from_ts_type(t))
                    .collect();
                types.join(" | ")
            }
            TSType::TSIntersectionType(intersection) => {
                let types: Vec<String> = intersection
                    .types
                    .iter()
                    .map(|t| self.extract_type_string_from_ts_type(t))
                    .collect();
                types.join(" & ")
            }
            TSType::TSFunctionType(func) => {
                let params = self.extract_function_params(&func.params);
                let return_type = self.extract_type_string_from_ts_type(&func.return_type.type_annotation);
                format!("({}) => {}", params, return_type)
            }
            TSType::TSTypeLiteral(literal) => {
                let props: Vec<String> = literal
                    .members
                    .iter()
                    .filter_map(|member| {
                        if let oxc_ast::ast::TSSignature::TSPropertySignature(prop) = member {
                            let name = match &prop.key {
                                oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => {
                                    ident.name.as_str().to_string()
                                }
                                oxc_ast::ast::PropertyKey::StringLiteral(str_lit) => {
                                    str_lit.value.as_str().to_string()
                                }
                                _ => return None,
                            };
                            let type_str = prop
                                .type_annotation
                                .as_ref()
                                .map(|ta| self.extract_type_string(ta))
                                .unwrap_or_else(|| "any".to_string());
                            let optional = if prop.optional { "?" } else { "" };
                            Some(format!("{}{}: {}", name, optional, type_str))
                        } else {
                            None
                        }
                    })
                    .collect();
                format!("{{ {} }}", props.join(", "))
            }
            _ => "any".to_string(),
        }
    }

    fn extract_type_string_from_ts_type(&self, ts_type: &oxc_ast::ast::TSType) -> String {
        use oxc_ast::ast::TSType;
        
        match ts_type {
            TSType::TSStringKeyword(_) => "string".to_string(),
            TSType::TSNumberKeyword(_) => "number".to_string(),
            TSType::TSBooleanKeyword(_) => "boolean".to_string(),
            TSType::TSAnyKeyword(_) => "any".to_string(),
            TSType::TSUnknownKeyword(_) => "unknown".to_string(),
            TSType::TSNeverKeyword(_) => "never".to_string(),
            TSType::TSVoidKeyword(_) => "void".to_string(),
            TSType::TSUndefinedKeyword(_) => "undefined".to_string(),
            TSType::TSNullKeyword(_) => "null".to_string(),
            TSType::TSArrayType(array) => {
                format!("{}[]", self.extract_type_string_from_ts_type(&array.element_type))
            }
            TSType::TSTypeReference(type_ref) => match &type_ref.type_name {
                oxc_ast::ast::TSTypeName::IdentifierReference(ident) => {
                    ident.name.as_str().to_string()
                }
                _ => "unknown".to_string(),
            }
            TSType::TSUnionType(union) => {
                let types: Vec<String> = union
                    .types
                    .iter()
                    .map(|t| self.extract_type_string_from_ts_type(t))
                    .collect();
                types.join(" | ")
            }
            TSType::TSIntersectionType(intersection) => {
                let types: Vec<String> = intersection
                    .types
                    .iter()
                    .map(|t| self.extract_type_string_from_ts_type(t))
                    .collect();
                types.join(" & ")
            }
            TSType::TSFunctionType(func) => {
                let params = self.extract_function_params(&func.params);
                let return_type = self.extract_type_string_from_ts_type(&func.return_type.type_annotation);
                format!("({}) => {}", params, return_type)
            }
            TSType::TSTypeLiteral(literal) => {
                let props: Vec<String> = literal
                    .members
                    .iter()
                    .filter_map(|member| {
                        if let oxc_ast::ast::TSSignature::TSPropertySignature(prop) = member {
                            let name = match &prop.key {
                                oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => {
                                    ident.name.as_str().to_string()
                                }
                                oxc_ast::ast::PropertyKey::StringLiteral(str_lit) => {
                                    str_lit.value.as_str().to_string()
                                }
                                _ => return None,
                            };
                            let type_str = prop
                                .type_annotation
                                .as_ref()
                                .map(|ta| self.extract_type_string(ta))
                                .unwrap_or_else(|| "any".to_string());
                            let optional = if prop.optional { "?" } else { "" };
                            Some(format!("{}{}: {}", name, optional, type_str))
                        } else {
                            None
                        }
                    })
                    .collect();
                format!("{{ {} }}", props.join(", "))
            }
            _ => "any".to_string(),
        }
    }

    fn extract_function_params(&self, params: &oxc_ast::ast::FormalParameters) -> String {
        let param_strings: Vec<String> = params
            .items
            .iter()
            .map(|param| {
                let name = match &param.pattern.kind {
                    oxc_ast::ast::BindingPatternKind::BindingIdentifier(ident) => {
                        ident.name.as_str()
                    }
                    _ => "param",
                };
                let type_str = param
                    .pattern
                    .type_annotation
                    .as_ref()
                    .map(|ta| self.extract_type_string(ta))
                    .unwrap_or_else(|| "any".to_string());
                format!("{}: {}", name, type_str)
            })
            .collect();
        param_strings.join(", ")
    }

    fn extract_class(&self, class: &oxc_ast::ast::Class) -> ClassDefinition {
        let name = class
            .id
            .as_ref()
            .map(|id| id.name.as_str().to_string())
            .unwrap_or_else(|| "AnonymousClass".to_string());

        let start_line = self.get_line_number(class.span.start as usize);
        let end_line = self.get_line_number(class.span.end as usize);

        let extends = class.super_class.as_ref().and_then(|super_class| {
            if let oxc_ast::ast::Expression::Identifier(ident) = super_class {
                Some(ident.name.as_str().to_string())
            } else {
                None
            }
        });

        let implements = class
            .implements
            .iter()
            .filter_map(|impl_clause| match &impl_clause.expression {
                oxc_ast::ast::TSTypeName::IdentifierReference(ident) => {
                    Some(ident.name.as_str().to_string())
                }
                _ => None,
            })
            .collect();

        let mut properties = Vec::new();
        let mut methods = Vec::new();
        let mut constructor_params = Vec::new();

        for element in &class.body.body {
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    let name = match &prop.key {
                        oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => {
                            ident.name.as_str().to_string()
                        }
                        oxc_ast::ast::PropertyKey::StringLiteral(str_lit) => {
                            str_lit.value.as_str().to_string()
                        }
                        _ => continue,
                    };

                    let type_annotation = prop
                        .type_annotation
                        .as_ref()
                        .map(|ta| self.extract_type_string(ta))
                        .unwrap_or_else(|| "any".to_string());

                    properties.push(ClassProperty {
                        name,
                        type_annotation,
                        is_static: prop.r#static,
                        is_private: false, // PropertyDefinitionType doesn't have TSPrivateProperty
                        is_readonly: prop.readonly,
                        is_optional: prop.optional,
                    });
                }
                ClassElement::MethodDefinition(method) => {
                    let name = match &method.key {
                        oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => {
                            ident.name.as_str().to_string()
                        }
                        oxc_ast::ast::PropertyKey::StringLiteral(str_lit) => {
                            str_lit.value.as_str().to_string()
                        }
                        _ => continue,
                    };

                    let kind = match method.kind {
                        MethodDefinitionKind::Constructor => {
                            // Extract constructor parameters
                            constructor_params = method
                                .value
                                .params
                                .items
                                .iter()
                                .map(|param| {
                                    let param_name = match &param.pattern.kind {
                                        oxc_ast::ast::BindingPatternKind::BindingIdentifier(
                                            ident,
                                        ) => ident.name.as_str(),
                                        _ => "param",
                                    };
                                    let type_str = param
                                        .pattern
                                        .type_annotation
                                        .as_ref()
                                        .map(|ta| self.extract_type_string(ta))
                                        .unwrap_or_else(|| "any".to_string());
                                    format!("{}: {}", param_name, type_str)
                                })
                                .collect();
                            MethodKind::Constructor
                        }
                        MethodDefinitionKind::Method => MethodKind::Method,
                        MethodDefinitionKind::Get => MethodKind::Getter,
                        MethodDefinitionKind::Set => MethodKind::Setter,
                    };

                    if kind != MethodKind::Constructor {
                        let parameters = self.extract_function_params(&method.value.params);
                        let return_type = method
                            .value
                            .return_type
                            .as_ref()
                            .map(|rt| self.extract_type_string_from_ts_type(&rt.type_annotation))
                            .unwrap_or_else(|| "void".to_string());

                        methods.push(ClassMethod {
                            name,
                            parameters: vec![parameters],
                            return_type,
                            is_static: method.r#static,
                            is_private: false, // Would need to check for private keyword
                            is_async: method.value.r#async,
                            is_generator: method.value.generator,
                            kind,
                        });
                    }
                }
                _ => {}
            }
        }

        ClassDefinition {
            name,
            properties,
            methods,
            constructor_params,
            extends,
            implements,
            start_line,
            end_line,
            file_path: self.file_path.clone(),
            is_abstract: class.r#abstract,
        }
    }

    pub fn extract_classes(&self) -> Result<Vec<ClassDefinition>, String> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(&self.file_path).unwrap_or(SourceType::tsx());
        let ret = Parser::new(&allocator, &self.source_text, source_type).parse();

        if !ret.errors.is_empty() {
            let error_messages: Vec<String> = ret
                .errors
                .iter()
                .map(|e| format!("{:?}", e))
                .collect();
            return Err(format!("Parse errors: {}", error_messages.join(", ")));
        }

        let mut classes = Vec::new();

        // Walk through all statements and find classes
        for statement in &ret.program.body {
            match statement {
                Statement::ExportDefaultDeclaration(export) => {
                    if let oxc_ast::ast::ExportDefaultDeclarationKind::ClassDeclaration(class) = &export.declaration {
                        classes.push(self.extract_class(class));
                    }
                }
                Statement::ExportNamedDeclaration(export) => {
                    if let Some(oxc_ast::ast::Declaration::ClassDeclaration(class)) = &export.declaration {
                        classes.push(self.extract_class(class));
                    }
                }
                Statement::ClassDeclaration(class) => {
                    classes.push(self.extract_class(class));
                }
                _ => {}
            }
        }

        Ok(classes)
    }
}

pub fn extract_classes_from_code(code: &str, file_path: &str) -> Result<Vec<ClassDefinition>, String> {
    let extractor = ClassExtractor::new(code.to_string(), file_path.to_string());
    extractor.extract_classes()
}

pub fn extract_classes_from_files(files: &[(String, String)]) -> Vec<ClassDefinition> {
    let mut all_classes = Vec::new();

    for (file_path, content) in files {
        match extract_classes_from_code(content, file_path) {
            Ok(classes) => all_classes.extend(classes),
            Err(e) => eprintln!("Error extracting classes from {}: {}", file_path, e),
        }
    }

    all_classes
}