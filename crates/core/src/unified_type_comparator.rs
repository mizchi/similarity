use crate::type_comparator::{TypeComparisonOptions, TypeComparisonResult, compare_types, compare_type_literal_with_type};
use crate::type_extractor::{TypeDefinition, TypeLiteralDefinition, TypeKind};

#[derive(Debug, Clone)]
pub enum UnifiedType {
    TypeDef(TypeDefinition),
    TypeLiteral(TypeLiteralDefinition),
}

impl UnifiedType {
    pub fn name(&self) -> &str {
        match self {
            UnifiedType::TypeDef(def) => &def.name,
            UnifiedType::TypeLiteral(lit) => &lit.name,
        }
    }

    pub fn file_path(&self) -> &str {
        match self {
            UnifiedType::TypeDef(def) => &def.file_path,
            UnifiedType::TypeLiteral(lit) => &lit.file_path,
        }
    }

    pub fn start_line(&self) -> usize {
        match self {
            UnifiedType::TypeDef(def) => def.start_line,
            UnifiedType::TypeLiteral(lit) => lit.start_line,
        }
    }

    pub fn end_line(&self) -> usize {
        match self {
            UnifiedType::TypeDef(def) => def.end_line,
            UnifiedType::TypeLiteral(lit) => lit.end_line,
        }
    }

    pub fn type_string(&self) -> String {
        match self {
            UnifiedType::TypeDef(def) => match def.kind {
                TypeKind::Interface => "interface",
                TypeKind::TypeAlias => "type",
                TypeKind::TypeLiteral => "type-literal",
            }.to_string(),
            UnifiedType::TypeLiteral(_) => "type-literal".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnifiedTypeComparisonPair {
    pub type1: UnifiedType,
    pub type2: UnifiedType,
    pub result: TypeComparisonResult,
}

/// Compare two unified types
fn compare_unified_types(
    type1: &UnifiedType,
    type2: &UnifiedType,
    options: &TypeComparisonOptions,
) -> TypeComparisonResult {
    match (type1, type2) {
        (UnifiedType::TypeDef(def1), UnifiedType::TypeDef(def2)) => {
            compare_types(def1, def2, options)
        }
        (UnifiedType::TypeDef(def), UnifiedType::TypeLiteral(lit)) |
        (UnifiedType::TypeLiteral(lit), UnifiedType::TypeDef(def)) => {
            compare_type_literal_with_type(lit, def, options)
        }
        (UnifiedType::TypeLiteral(lit1), UnifiedType::TypeLiteral(lit2)) => {
            // Convert type literals to temporary type definitions for comparison
            let def1 = type_literal_to_type_def(lit1);
            let def2 = type_literal_to_type_def(lit2);
            compare_types(&def1, &def2, options)
        }
    }
}

/// Convert type literal to type definition for comparison
fn type_literal_to_type_def(literal: &TypeLiteralDefinition) -> TypeDefinition {
    TypeDefinition {
        name: literal.name.clone(),
        kind: TypeKind::TypeLiteral,
        properties: literal.properties.clone(),
        generics: Vec::new(),
        extends: Vec::new(),
        start_line: literal.start_line,
        end_line: literal.end_line,
        file_path: literal.file_path.clone(),
    }
}

/// Check if two types should be compared (avoid self-comparison)
fn should_compare(type1: &UnifiedType, type2: &UnifiedType) -> bool {
    // Never compare a type with itself
    if std::ptr::eq(type1, type2) {
        return false;
    }

    // If same file and overlapping lines, it's likely the same definition
    if type1.file_path() == type2.file_path() {
        let range1 = type1.start_line()..=type1.end_line();
        let range2 = type2.start_line()..=type2.end_line();
        
        // Check if ranges overlap
        if range1.start() <= range2.end() && range2.start() <= range1.end() {
            return false;
        }
        
        // For type literals with same name in same file, skip
        if matches!((type1, type2), (UnifiedType::TypeLiteral(_), UnifiedType::TypeLiteral(_))) {
            if type1.name() == type2.name() {
                return false;
            }
        }
    }

    true
}

/// Find all similar types (unified comparison)
pub fn find_similar_unified_types(
    type_definitions: &[TypeDefinition],
    type_literals: &[TypeLiteralDefinition],
    threshold: f64,
    options: &TypeComparisonOptions,
) -> Vec<UnifiedTypeComparisonPair> {
    // Combine all types into unified list
    let mut all_types = Vec::new();
    
    for def in type_definitions {
        all_types.push(UnifiedType::TypeDef(def.clone()));
    }
    
    for lit in type_literals {
        all_types.push(UnifiedType::TypeLiteral(lit.clone()));
    }
    
    let mut similar_pairs = Vec::new();
    
    // Compare all pairs
    for i in 0..all_types.len() {
        for j in (i + 1)..all_types.len() {
            let type1 = &all_types[i];
            let type2 = &all_types[j];
            
            if !should_compare(type1, type2) {
                continue;
            }
            
            let result = compare_unified_types(type1, type2, options);
            
            if result.similarity >= threshold {
                similar_pairs.push(UnifiedTypeComparisonPair {
                    type1: type1.clone(),
                    type2: type2.clone(),
                    result,
                });
            }
        }
    }
    
    // Sort by similarity (descending)
    similar_pairs.sort_by(|a, b| {
        b.result.similarity.partial_cmp(&a.result.similarity).unwrap()
    });
    
    similar_pairs
}