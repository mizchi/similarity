use crate::structure_comparator::{
    Structure, StructureIdentifier, StructureKind, StructureMember, StructureMetadata,
    SourceLocation, StructureComparator, ComparisonOptions, StructureComparisonResult,
};
use crate::language_parser::GenericTypeDef;

/// Rustの型定義を一般構造に変換
impl From<GenericTypeDef> for Structure {
    fn from(type_def: GenericTypeDef) -> Self {
        let kind = match type_def.kind.as_str() {
            "struct" => StructureKind::RustStruct,
            "enum" => StructureKind::RustEnum,
            _ => StructureKind::Generic(type_def.kind.clone()),
        };
        
        let members = type_def.fields
            .into_iter()
            .map(|field| {
                // For enums, fields are variants
                // For structs, fields are actual fields
                let (name, value_type) = if type_def.kind == "enum" {
                    // Enum variant - the field is the variant name
                    (field.clone(), "variant".to_string())
                } else {
                    // Struct field - we need to parse the type
                    // For now, we'll use a placeholder since GenericTypeDef doesn't store types
                    (field.clone(), "unknown".to_string())
                };
                
                StructureMember {
                    name,
                    value_type,
                    modifiers: vec![],
                    nested: None,
                }
            })
            .collect();
        
        Structure {
            identifier: StructureIdentifier {
                name: type_def.name.clone(),
                kind,
                namespace: None, // Could be module path
            },
            members,
            metadata: StructureMetadata {
                location: SourceLocation {
                    file_path: String::new(), // Would need to pass this separately
                    start_line: type_def.start_line as usize,
                    end_line: type_def.end_line as usize,
                },
                generics: Vec::new(), // Could extract from type parameters
                extends: Vec::new(), // Could extract traits
                visibility: None, // Could extract pub/pub(crate)/etc
            },
        }
    }
}

/// Rust構造体の詳細定義（より詳細な情報を含む）
#[derive(Debug, Clone)]
pub struct RustStructDef {
    pub name: String,
    pub fields: Vec<RustFieldDef>,
    pub generics: Vec<String>,
    pub derives: Vec<String>,
    pub attributes: Vec<String>,  // Other attributes like #[serde(...)], #[cfg(...)]
    pub visibility: Option<String>,
    pub is_tuple_struct: bool,
    pub start_line: usize,
    pub end_line: usize,
    pub file_path: String,
}

#[derive(Debug, Clone)]
pub struct RustFieldDef {
    pub name: String,
    pub field_type: String,
    pub visibility: Option<String>,
}

/// Rust enum の詳細定義
#[derive(Debug, Clone)]
pub struct RustEnumDef {
    pub name: String,
    pub variants: Vec<RustVariantDef>,
    pub generics: Vec<String>,
    pub derives: Vec<String>,
    pub attributes: Vec<String>,  // Other attributes like #[serde(...)], #[cfg(...)]
    pub visibility: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub file_path: String,
}

#[derive(Debug, Clone)]
pub struct RustVariantDef {
    pub name: String,
    pub variant_type: RustVariantType,
}

#[derive(Debug, Clone)]
pub enum RustVariantType {
    Unit,
    Tuple(Vec<String>),
    Struct(Vec<RustFieldDef>),
}

/// Rust構造体を一般構造に変換
impl From<RustStructDef> for Structure {
    fn from(struct_def: RustStructDef) -> Self {
        let mut members: Vec<StructureMember> = struct_def.fields
            .into_iter()
            .map(|field| StructureMember {
                name: field.name,
                value_type: field.field_type,
                modifiers: field.visibility.map(|v| vec![v]).unwrap_or_default(),
                nested: None,
            })
            .collect();
        
        // Add derives as special members for comparison
        if !struct_def.derives.is_empty() {
            members.push(StructureMember {
                name: "@derives".to_string(),
                value_type: struct_def.derives.join(", "),
                modifiers: vec!["attribute".to_string()],
                nested: None,
            });
        }
        
        // Add other attributes as special members
        if !struct_def.attributes.is_empty() {
            members.push(StructureMember {
                name: "@attributes".to_string(),
                value_type: struct_def.attributes.join(", "),
                modifiers: vec!["attribute".to_string()],
                nested: None,
            });
        }
        
        Structure {
            identifier: StructureIdentifier {
                name: struct_def.name.clone(),
                kind: StructureKind::RustStruct,
                namespace: Some(struct_def.file_path.clone()),
            },
            members,
            metadata: StructureMetadata {
                location: SourceLocation {
                    file_path: struct_def.file_path,
                    start_line: struct_def.start_line,
                    end_line: struct_def.end_line,
                },
                generics: struct_def.generics,
                extends: vec![], // Don't put derives here, they're in members now
                visibility: struct_def.visibility,
            },
        }
    }
}

/// Rust enumを一般構造に変換
impl From<RustEnumDef> for Structure {
    fn from(enum_def: RustEnumDef) -> Self {
        let mut members: Vec<StructureMember> = enum_def.variants
            .into_iter()
            .map(|variant| {
                let value_type = match variant.variant_type {
                    RustVariantType::Unit => "unit".to_string(),
                    RustVariantType::Tuple(ref types) => format!("({})", types.join(", ")),
                    RustVariantType::Struct(ref fields) => {
                        let field_strs: Vec<String> = fields
                            .iter()
                            .map(|f| format!("{}: {}", f.name, f.field_type))
                            .collect();
                        format!("{{ {} }}", field_strs.join(", "))
                    }
                };
                
                StructureMember {
                    name: variant.name,
                    value_type,
                    modifiers: vec!["variant".to_string()],
                    nested: None,
                }
            })
            .collect();
        
        // Add derives as special members for comparison
        if !enum_def.derives.is_empty() {
            members.push(StructureMember {
                name: "@derives".to_string(),
                value_type: enum_def.derives.join(", "),
                modifiers: vec!["attribute".to_string()],
                nested: None,
            });
        }
        
        // Add other attributes as special members
        if !enum_def.attributes.is_empty() {
            members.push(StructureMember {
                name: "@attributes".to_string(),
                value_type: enum_def.attributes.join(", "),
                modifiers: vec!["attribute".to_string()],
                nested: None,
            });
        }
        
        Structure {
            identifier: StructureIdentifier {
                name: enum_def.name.clone(),
                kind: StructureKind::RustEnum,
                namespace: Some(enum_def.file_path.clone()),
            },
            members,
            metadata: StructureMetadata {
                location: SourceLocation {
                    file_path: enum_def.file_path,
                    start_line: enum_def.start_line,
                    end_line: enum_def.end_line,
                },
                generics: enum_def.generics,
                extends: vec![], // Don't put derives here, they're in members now
                visibility: enum_def.visibility,
            },
        }
    }
}

/// Rust用の比較エンジン
pub struct RustStructureComparator {
    pub comparator: StructureComparator,
}

impl RustStructureComparator {
    pub fn new() -> Self {
        let options = ComparisonOptions {
            name_weight: 0.3,
            structure_weight: 0.7,
            threshold: 0.7,
            ..Default::default()
        };
        
        Self {
            comparator: StructureComparator::new(options),
        }
    }
    
    pub fn with_options(options: ComparisonOptions) -> Self {
        Self {
            comparator: StructureComparator::new(options),
        }
    }
    
    /// 構造体を比較
    pub fn compare_structs(&mut self, struct1: &RustStructDef, struct2: &RustStructDef) -> StructureComparisonResult {
        let s1 = Structure::from(struct1.clone());
        let s2 = Structure::from(struct2.clone());
        self.comparator.compare(&s1, &s2)
    }
    
    /// Enumを比較
    pub fn compare_enums(&mut self, enum1: &RustEnumDef, enum2: &RustEnumDef) -> StructureComparisonResult {
        let s1 = Structure::from(enum1.clone());
        let s2 = Structure::from(enum2.clone());
        self.comparator.compare(&s1, &s2)
    }
    
    /// 汎用型定義を比較
    pub fn compare_generic_types(&mut self, type1: &GenericTypeDef, type2: &GenericTypeDef) -> StructureComparisonResult {
        let s1 = Structure::from(type1.clone());
        let s2 = Structure::from(type2.clone());
        self.comparator.compare(&s1, &s2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_struct_to_structure_conversion() {
        let rust_struct = RustStructDef {
            name: "User".to_string(),
            fields: vec![
                RustFieldDef {
                    name: "id".to_string(),
                    field_type: "u64".to_string(),
                    visibility: Some("pub".to_string()),
                },
                RustFieldDef {
                    name: "name".to_string(),
                    field_type: "String".to_string(),
                    visibility: Some("pub".to_string()),
                },
            ],
            generics: vec![],
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            attributes: vec![],
            visibility: Some("pub".to_string()),
            is_tuple_struct: false,
            start_line: 1,
            end_line: 5,
            file_path: "user.rs".to_string(),
        };
        
        let structure = Structure::from(rust_struct);
        
        assert_eq!(structure.identifier.name, "User");
        assert_eq!(structure.identifier.kind, StructureKind::RustStruct);
        assert_eq!(structure.members.len(), 3); // 2 fields + 1 @derives
        assert_eq!(structure.members[0].name, "id");
        assert_eq!(structure.members[0].value_type, "u64");
        // Check derives member
        assert_eq!(structure.members[2].name, "@derives");
        assert_eq!(structure.members[2].value_type, "Debug, Clone");
    }
    
    #[test]
    fn test_enum_to_structure_conversion() {
        let rust_enum = RustEnumDef {
            name: "Result".to_string(),
            variants: vec![
                RustVariantDef {
                    name: "Ok".to_string(),
                    variant_type: RustVariantType::Tuple(vec!["T".to_string()]),
                },
                RustVariantDef {
                    name: "Err".to_string(),
                    variant_type: RustVariantType::Tuple(vec!["E".to_string()]),
                },
            ],
            generics: vec!["T".to_string(), "E".to_string()],
            derives: vec!["Debug".to_string()],
            attributes: vec![],
            visibility: Some("pub".to_string()),
            start_line: 1,
            end_line: 4,
            file_path: "result.rs".to_string(),
        };
        
        let structure = Structure::from(rust_enum);
        
        assert_eq!(structure.identifier.name, "Result");
        assert_eq!(structure.identifier.kind, StructureKind::RustEnum);
        assert_eq!(structure.members.len(), 3); // 2 variants + 1 @derives
        assert_eq!(structure.members[0].name, "Ok");
        assert_eq!(structure.members[0].value_type, "(T)");
        // Check derives member
        assert_eq!(structure.members[2].name, "@derives");
        assert_eq!(structure.members[2].value_type, "Debug");
    }
    
    #[test]
    fn test_rust_comparator() {
        let mut comparator = RustStructureComparator::new();
        
        let struct1 = RustStructDef {
            name: "User".to_string(),
            fields: vec![
                RustFieldDef {
                    name: "id".to_string(),
                    field_type: "u64".to_string(),
                    visibility: Some("pub".to_string()),
                },
                RustFieldDef {
                    name: "name".to_string(),
                    field_type: "String".to_string(),
                    visibility: Some("pub".to_string()),
                },
            ],
            generics: vec![],
            derives: vec![],
            attributes: vec![],
            visibility: Some("pub".to_string()),
            is_tuple_struct: false,
            start_line: 1,
            end_line: 5,
            file_path: "user.rs".to_string(),
        };
        
        let struct2 = RustStructDef {
            name: "Person".to_string(),
            fields: vec![
                RustFieldDef {
                    name: "id".to_string(),
                    field_type: "u64".to_string(),
                    visibility: Some("pub".to_string()),
                },
                RustFieldDef {
                    name: "name".to_string(),
                    field_type: "String".to_string(),
                    visibility: Some("pub".to_string()),
                },
            ],
            generics: vec![],
            derives: vec![],
            attributes: vec![],
            visibility: Some("pub".to_string()),
            is_tuple_struct: false,
            start_line: 10,
            end_line: 15,
            file_path: "person.rs".to_string(),
        };
        
        let result = comparator.compare_structs(&struct1, &struct2);
        
        // Same structure, different names
        assert!(result.member_similarity > 0.9);
        assert!(result.identifier_similarity < 0.5);
        assert!(result.overall_similarity > 0.6);
    }
    
    
    #[test]
    fn test_struct_comparison_with_derives() {
        let mut comparator = RustStructureComparator::new();
        
        let struct1 = RustStructDef {
            name: "User".to_string(),
            fields: vec![
                RustFieldDef {
                    name: "id".to_string(),
                    field_type: "u64".to_string(),
                    visibility: Some("pub".to_string()),
                },
            ],
            generics: vec![],
            derives: vec!["Debug".to_string(), "Clone".to_string(), "Serialize".to_string()],
            attributes: vec![],
            visibility: Some("pub".to_string()),
            is_tuple_struct: false,
            start_line: 1,
            end_line: 5,
            file_path: "user.rs".to_string(),
        };
        
        // Same fields, different derives
        let struct2 = RustStructDef {
            name: "User".to_string(),
            fields: vec![
                RustFieldDef {
                    name: "id".to_string(),
                    field_type: "u64".to_string(),
                    visibility: Some("pub".to_string()),
                },
            ],
            generics: vec![],
            derives: vec!["Debug".to_string(), "PartialEq".to_string()],
            attributes: vec![],
            visibility: Some("pub".to_string()),
            is_tuple_struct: false,
            start_line: 10,
            end_line: 14,
            file_path: "user.rs".to_string(),
        };
        
        let result1 = comparator.compare_structs(&struct1, &struct2);
        
        // Same fields, same derives
        let struct3 = RustStructDef {
            name: "User".to_string(),
            fields: vec![
                RustFieldDef {
                    name: "id".to_string(),
                    field_type: "u64".to_string(),
                    visibility: Some("pub".to_string()),
                },
            ],
            generics: vec![],
            derives: vec!["Debug".to_string(), "Clone".to_string(), "Serialize".to_string()],
            attributes: vec![],
            visibility: Some("pub".to_string()),
            is_tuple_struct: false,
            start_line: 20,
            end_line: 24,
            file_path: "user.rs".to_string(),
        };
        
        let result2 = comparator.compare_structs(&struct1, &struct3);
        
        // Structs with same derives should have equal or higher similarity
        // Both have the same fields, but different @derives members
        assert!(result2.overall_similarity >= result1.overall_similarity);
        
        // Check @derives member comparison
        // For struct1 and struct3 (same derives), @derives should match perfectly (1.0)
        let derives_match_result2 = result2.member_matches.iter()
            .find(|m| m.member1 == "@derives")
            .map(|m| m.similarity);
        assert_eq!(derives_match_result2, Some(1.0));
        
        // For struct1 and struct2 (different derives), @derives should have lower similarity
        let derives_match_result1 = result1.member_matches.iter()
            .find(|m| m.member1 == "@derives")
            .map(|m| m.similarity);
        // The derives are different but may have partial match (both have "Debug")
        assert!(derives_match_result1.unwrap_or(0.0) < 1.0);
        
        // Both should have same number of member matches (id + @derives)
        assert_eq!(result2.member_matches.len(), 2);
        assert_eq!(result1.member_matches.len(), 2);
    }
}