#[cfg(test)]
mod tests {
    use crate::structure_comparator::*;
    use crate::typescript_structure_adapter::*;
    use crate::type_extractor::{TypeDefinition, TypeKind, PropertyDefinition};
    
    #[test]
    fn test_structure_comparison_basic() {
        let mut comparator = StructureComparator::new(ComparisonOptions::default());
        
        let struct1 = Structure {
            identifier: StructureIdentifier {
                name: "User".to_string(),
                kind: StructureKind::TypeScriptInterface,
                namespace: Some("test.ts".to_string()),
            },
            members: vec![
                StructureMember {
                    name: "id".to_string(),
                    value_type: "string".to_string(),
                    modifiers: vec![],
                    nested: None,
                },
                StructureMember {
                    name: "name".to_string(),
                    value_type: "string".to_string(),
                    modifiers: vec![],
                    nested: None,
                },
            ],
            metadata: StructureMetadata::default(),
        };
        
        let struct2 = Structure {
            identifier: StructureIdentifier {
                name: "Person".to_string(),
                kind: StructureKind::TypeScriptInterface,
                namespace: Some("test.ts".to_string()),
            },
            members: vec![
                StructureMember {
                    name: "id".to_string(),
                    value_type: "string".to_string(),
                    modifiers: vec![],
                    nested: None,
                },
                StructureMember {
                    name: "name".to_string(),
                    value_type: "string".to_string(),
                    modifiers: vec![],
                    nested: None,
                },
            ],
            metadata: StructureMetadata::default(),
        };
        
        let result = comparator.compare(&struct1, &struct2);
        
        // Should have high structural similarity (same members)
        assert!(result.member_similarity > 0.9);
        // Should have lower naming similarity (different names)
        assert!(result.identifier_similarity < 0.5);
        // Overall should be reasonably high
        assert!(result.overall_similarity > 0.6);
    }
    
    #[test]
    fn test_fingerprint_generation() {
        let structure = Structure {
            identifier: StructureIdentifier {
                name: "User".to_string(),
                kind: StructureKind::TypeScriptInterface,
                namespace: Some("test.ts".to_string()),
            },
            members: vec![
                StructureMember {
                    name: "id".to_string(),
                    value_type: "string".to_string(),
                    modifiers: vec![],
                    nested: None,
                },
                StructureMember {
                    name: "age".to_string(),
                    value_type: "number".to_string(),
                    modifiers: vec![],
                    nested: None,
                },
                StructureMember {
                    name: "tags".to_string(),
                    value_type: "string[]".to_string(),
                    modifiers: vec![],
                    nested: None,
                },
            ],
            metadata: StructureMetadata::default(),
        };
        
        let fingerprint = compute_structure_fingerprint(&structure);
        
        println!("Fingerprint: {}", fingerprint);
        
        // Should contain kind, size category, member count, and type distribution
        assert!(fingerprint.contains("kind:TypeScriptInterface"));
        assert!(fingerprint.contains("size:small")); // 3 members = small
        assert!(fingerprint.contains("members:3"));
        // Type distribution gets normalized - id is string, tags[] becomes array
        assert!(fingerprint.contains("string:1"));
        assert!(fingerprint.contains("number:1"));
        // string[] is normalized to "array"
        assert!(fingerprint.contains("array:1"));
    }
    
    #[test]
    fn test_typescript_adapter() {
        let mut comparator = TypeScriptStructureComparator::new();
        
        let type1 = TypeDefinition {
            name: "User".to_string(),
            kind: TypeKind::Interface,
            properties: vec![
                PropertyDefinition {
                    name: "id".to_string(),
                    type_annotation: "string".to_string(),
                    optional: false,
                    readonly: false,
                },
            ],
            generics: vec![],
            extends: vec![],
            start_line: 1,
            end_line: 5,
            file_path: "test.ts".to_string(),
        };
        
        let type2 = TypeDefinition {
            name: "User".to_string(),
            kind: TypeKind::Interface,
            properties: vec![
                PropertyDefinition {
                    name: "id".to_string(),
                    type_annotation: "string".to_string(),
                    optional: false,
                    readonly: false,
                },
            ],
            generics: vec![],
            extends: vec![],
            start_line: 10,
            end_line: 15,
            file_path: "test.ts".to_string(),
        };
        
        let result = comparator.compare_types(&type1, &type2);
        
        // Identical types should have very high similarity
        assert!(result.overall_similarity > 0.95);
        assert_eq!(result.member_matches.len(), 1);
        assert!(result.differences.missing_members.is_empty());
        assert!(result.differences.extra_members.is_empty());
    }
}