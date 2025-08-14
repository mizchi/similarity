use crate::structure_comparator::{
    Structure, StructureIdentifier, StructureKind, StructureMember, StructureMetadata,
    SourceLocation, StructureComparator, ComparisonOptions, StructureComparisonResult,
};
use crate::type_extractor::{TypeDefinition, TypeKind, TypeLiteralDefinition, PropertyDefinition};
use crate::class_extractor::{ClassDefinition, ClassProperty, ClassMethod};

/// TypeScriptの型定義を一般構造に変換
impl From<TypeDefinition> for Structure {
    fn from(type_def: TypeDefinition) -> Self {
        let kind = match type_def.kind {
            TypeKind::Interface => StructureKind::TypeScriptInterface,
            TypeKind::TypeAlias => StructureKind::TypeScriptTypeAlias,
            TypeKind::TypeLiteral => StructureKind::TypeScriptTypeLiteral,
        };
        
        Structure {
            identifier: StructureIdentifier {
                name: type_def.name.clone(),
                kind,
                namespace: Some(type_def.file_path.clone()),
            },
            members: type_def.properties
                .into_iter()
                .map(|prop| property_to_member(prop))
                .collect(),
            metadata: StructureMetadata {
                location: SourceLocation {
                    file_path: type_def.file_path,
                    start_line: type_def.start_line,
                    end_line: type_def.end_line,
                },
                generics: type_def.generics,
                extends: type_def.extends,
                visibility: None,
            },
        }
    }
}

/// TypeScriptのtype literalを一般構造に変換
impl From<TypeLiteralDefinition> for Structure {
    fn from(literal: TypeLiteralDefinition) -> Self {
        Structure {
            identifier: StructureIdentifier {
                name: literal.name.clone(),
                kind: StructureKind::TypeScriptTypeLiteral,
                namespace: Some(literal.file_path.clone()),
            },
            members: literal.properties
                .into_iter()
                .map(|prop| property_to_member(prop))
                .collect(),
            metadata: StructureMetadata {
                location: SourceLocation {
                    file_path: literal.file_path,
                    start_line: literal.start_line,
                    end_line: literal.end_line,
                },
                generics: Vec::new(),
                extends: Vec::new(),
                visibility: None,
            },
        }
    }
}

/// TypeScriptのクラスを一般構造に変換
impl From<ClassDefinition> for Structure {
    fn from(class: ClassDefinition) -> Self {
        let mut members = Vec::new();
        
        // プロパティを追加
        for prop in class.properties {
            members.push(class_property_to_member(prop));
        }
        
        // メソッドを追加
        for method in class.methods {
            members.push(class_method_to_member(method));
        }
        
        // コンストラクタパラメータを追加
        for (i, param) in class.constructor_params.iter().enumerate() {
            members.push(StructureMember {
                name: format!("constructor_param_{}", i),
                value_type: param.clone(),
                modifiers: vec!["constructor".to_string()],
                nested: None,
            });
        }
        
        Structure {
            identifier: StructureIdentifier {
                name: class.name.clone(),
                kind: StructureKind::TypeScriptClass,
                namespace: Some(class.file_path.clone()),
            },
            members,
            metadata: StructureMetadata {
                location: SourceLocation {
                    file_path: class.file_path,
                    start_line: class.start_line,
                    end_line: class.end_line,
                },
                generics: Vec::new(),
                extends: class.extends.into_iter().collect::<Vec<_>>(),
                visibility: if class.is_abstract {
                    Some("abstract".to_string())
                } else {
                    None
                },
            },
        }
    }
}

fn property_to_member(prop: PropertyDefinition) -> StructureMember {
    let mut modifiers = Vec::new();
    if prop.optional {
        modifiers.push("optional".to_string());
    }
    if prop.readonly {
        modifiers.push("readonly".to_string());
    }
    
    StructureMember {
        name: prop.name,
        value_type: prop.type_annotation,
        modifiers,
        nested: None,
    }
}

fn class_property_to_member(prop: ClassProperty) -> StructureMember {
    let mut modifiers = Vec::new();
    if prop.is_private {
        modifiers.push("private".to_string());
    }
    if prop.is_static {
        modifiers.push("static".to_string());
    }
    if prop.is_readonly {
        modifiers.push("readonly".to_string());
    }
    if prop.is_optional {
        modifiers.push("optional".to_string());
    }
    
    StructureMember {
        name: prop.name,
        value_type: prop.type_annotation,
        modifiers,
        nested: None,
    }
}

fn class_method_to_member(method: ClassMethod) -> StructureMember {
    let mut modifiers = Vec::new();
    if method.is_private {
        modifiers.push("private".to_string());
    }
    if method.is_static {
        modifiers.push("static".to_string());
    }
    if method.is_async {
        modifiers.push("async".to_string());
    }
    modifiers.push("method".to_string());
    
    // メソッドシグネチャを型として表現
    let signature = format!(
        "({}) => {}",
        method.parameters.join(", "),
        method.return_type
    );
    
    StructureMember {
        name: method.name,
        value_type: signature,
        modifiers,
        nested: None,
    }
}

/// TypeScript用の比較エンジン
pub struct TypeScriptStructureComparator {
    pub comparator: StructureComparator,
}

impl TypeScriptStructureComparator {
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
    
    /// 型定義を比較
    pub fn compare_types(&mut self, type1: &TypeDefinition, type2: &TypeDefinition) -> StructureComparisonResult {
        let struct1 = Structure::from(type1.clone());
        let struct2 = Structure::from(type2.clone());
        self.comparator.compare(&struct1, &struct2)
    }
    
    /// type literalを比較
    pub fn compare_type_literals(&mut self, lit1: &TypeLiteralDefinition, lit2: &TypeLiteralDefinition) -> StructureComparisonResult {
        let struct1 = Structure::from(lit1.clone());
        let struct2 = Structure::from(lit2.clone());
        self.comparator.compare(&struct1, &struct2)
    }
    
    /// 型定義とtype literalを比較
    pub fn compare_type_with_literal(&mut self, type_def: &TypeDefinition, literal: &TypeLiteralDefinition) -> StructureComparisonResult {
        let struct1 = Structure::from(type_def.clone());
        let struct2 = Structure::from(literal.clone());
        self.comparator.compare(&struct1, &struct2)
    }
    
    /// クラスを比較
    pub fn compare_classes(&mut self, class1: &ClassDefinition, class2: &ClassDefinition) -> StructureComparisonResult {
        let struct1 = Structure::from(class1.clone());
        let struct2 = Structure::from(class2.clone());
        self.comparator.compare(&struct1, &struct2)
    }
    
    /// 任意の構造を比較（型、クラス、type literalなど）
    pub fn compare_any(&mut self, struct1: Structure, struct2: Structure) -> StructureComparisonResult {
        self.comparator.compare(&struct1, &struct2)
    }
}

/// 複数の構造を効率的に比較
pub struct BatchComparator {
    comparator: TypeScriptStructureComparator,
    fingerprint_cache: std::collections::HashMap<String, Vec<Structure>>,
}

impl BatchComparator {
    pub fn new() -> Self {
        Self {
            comparator: TypeScriptStructureComparator::new(),
            fingerprint_cache: std::collections::HashMap::new(),
        }
    }
    
    /// 構造をフィンガープリントでグループ化
    pub fn group_by_fingerprint(&mut self, structures: Vec<Structure>) {
        for structure in structures {
            let fingerprint = self.comparator.comparator.generate_fingerprint(&structure);
            self.fingerprint_cache
                .entry(fingerprint)
                .or_insert_with(Vec::new)
                .push(structure);
        }
    }
    
    /// 類似構造を検出
    pub fn find_similar_structures(&mut self, threshold: f64) -> Vec<(Structure, Structure, f64)> {
        use crate::structure_comparator::should_compare_fingerprints;
        
        let mut results = Vec::new();
        
        // フィンガープリントのリストを取得
        let fingerprints: Vec<String> = self.fingerprint_cache.keys().cloned().collect();
        
        // フィンガープリントが類似している組み合わせのみ比較
        for i in 0..fingerprints.len() {
            for j in i..fingerprints.len() {
                let fp1 = &fingerprints[i];
                let fp2 = &fingerprints[j];
                
                // フィンガープリントが比較対象として妥当かチェック
                if !should_compare_fingerprints(fp1, fp2) {
                    continue;
                }
                
                let structures1 = &self.fingerprint_cache[fp1];
                let structures2 = &self.fingerprint_cache[fp2];
                
                // 同じグループ内または異なるグループ間で比較
                for s1 in structures1 {
                    let start_idx = if i == j { 
                        // 同じグループ内の場合、自己比較を避ける
                        structures2.iter().position(|s| std::ptr::eq(s, s1))
                            .map(|pos| pos + 1).unwrap_or(0)
                    } else { 
                        0 
                    };
                    
                    for s2 in &structures2[start_idx..] {
                        let result = self.comparator.compare_any(
                            s1.clone(),
                            s2.clone()
                        );
                        
                        if result.overall_similarity >= threshold {
                            results.push((
                                s1.clone(),
                                s2.clone(),
                                result.overall_similarity,
                            ));
                        }
                    }
                }
            }
        }
        
        // 類似度でソート
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_to_structure_conversion() {
        let type_def = TypeDefinition {
            name: "User".to_string(),
            kind: TypeKind::Interface,
            properties: vec![
                PropertyDefinition {
                    name: "id".to_string(),
                    type_annotation: "string".to_string(),
                    optional: false,
                    readonly: true,
                },
                PropertyDefinition {
                    name: "name".to_string(),
                    type_annotation: "string".to_string(),
                    optional: false,
                    readonly: false,
                },
            ],
            generics: vec![],
            extends: vec![],
            start_line: 1,
            end_line: 5,
            file_path: "user.ts".to_string(),
        };
        
        let structure = Structure::from(type_def);
        
        assert_eq!(structure.identifier.name, "User");
        assert_eq!(structure.identifier.kind, StructureKind::TypeScriptInterface);
        assert_eq!(structure.members.len(), 2);
        assert!(structure.members[0].modifiers.contains(&"readonly".to_string()));
    }
    
    #[test]
    fn test_structure_comparison() {
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
                PropertyDefinition {
                    name: "name".to_string(),
                    type_annotation: "string".to_string(),
                    optional: false,
                    readonly: false,
                },
            ],
            generics: vec![],
            extends: vec![],
            start_line: 1,
            end_line: 5,
            file_path: "user.ts".to_string(),
        };
        
        let type2 = TypeDefinition {
            name: "Person".to_string(),
            kind: TypeKind::Interface,
            properties: vec![
                PropertyDefinition {
                    name: "id".to_string(),
                    type_annotation: "string".to_string(),
                    optional: false,
                    readonly: false,
                },
                PropertyDefinition {
                    name: "name".to_string(),
                    type_annotation: "string".to_string(),
                    optional: false,
                    readonly: false,
                },
            ],
            generics: vec![],
            extends: vec![],
            start_line: 10,
            end_line: 15,
            file_path: "person.ts".to_string(),
        };
        
        let result = comparator.compare_types(&type1, &type2);
        
        // User vs Person have different names but same structure
        // With default weights (0.3 name, 0.7 structure), similarity should be ~0.7
        assert!(result.overall_similarity > 0.6);
        assert_eq!(result.member_matches.len(), 2);
        assert!(result.differences.missing_members.is_empty());
        assert!(result.differences.extra_members.is_empty());
    }
}