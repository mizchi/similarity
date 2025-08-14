use crate::type_comparator::{
    compare_type_literal_with_type, compare_types, TypeComparisonOptions, TypeComparisonResult,
};
use crate::type_extractor::{TypeDefinition, TypeKind, TypeLiteralDefinition};
use crate::typescript_structure_adapter::TypeScriptStructureComparator;
use crate::structure_comparator::{Structure, ComparisonOptions};

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
            }
            .to_string(),
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
        (UnifiedType::TypeDef(def), UnifiedType::TypeLiteral(lit))
        | (UnifiedType::TypeLiteral(lit), UnifiedType::TypeDef(def)) => {
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
        if matches!((type1, type2), (UnifiedType::TypeLiteral(_), UnifiedType::TypeLiteral(_)))
            && type1.name() == type2.name()
        {
            return false;
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
    similar_pairs.sort_by(|a, b| b.result.similarity.partial_cmp(&a.result.similarity).unwrap());

    similar_pairs
}

/// Find similar types using the new generalized structure comparison framework
pub fn find_similar_unified_types_structured(
    type_definitions: &[TypeDefinition],
    type_literals: &[TypeLiteralDefinition],
    threshold: f64,
    options: Option<ComparisonOptions>,
) -> Vec<UnifiedTypeComparisonPair> {
    let mut comparator = if let Some(opts) = options {
        TypeScriptStructureComparator::with_options(opts)
    } else {
        TypeScriptStructureComparator::new()
    };
    
    let mut similar_pairs = Vec::new();
    
    // Convert all to structures
    let mut all_structures: Vec<(UnifiedType, Structure)> = Vec::new();
    
    for type_def in type_definitions {
        let unified = UnifiedType::TypeDef(type_def.clone());
        let structure = Structure::from(type_def.clone());
        all_structures.push((unified, structure));
    }
    
    for type_literal in type_literals {
        let unified = UnifiedType::TypeLiteral(type_literal.clone());
        let structure = Structure::from(type_literal.clone());
        all_structures.push((unified, structure));
    }
    
    // Compare all pairs
    for i in 0..all_structures.len() {
        for j in (i + 1)..all_structures.len() {
            let (unified1, struct1) = &all_structures[i];
            let (unified2, struct2) = &all_structures[j];
            
            if !should_compare(unified1, unified2) {
                continue;
            }
            
            let result = comparator.comparator.compare(struct1, struct2);
            
            if result.overall_similarity >= threshold {
                // Convert structure comparison result to TypeComparisonResult
                let type_result = TypeComparisonResult {
                    similarity: result.overall_similarity,
                    structural_similarity: result.member_similarity,
                    naming_similarity: result.identifier_similarity,
                    matched_properties: result.member_matches.iter().map(|m| {
                        crate::type_comparator::MatchedProperty {
                            prop1: m.member1.clone(),
                            prop2: m.member2.clone(),
                            similarity: m.similarity,
                        }
                    }).collect(),
                    differences: crate::type_comparator::TypeDifferences {
                        missing_properties: result.differences.missing_members.clone(),
                        extra_properties: result.differences.extra_members.clone(),
                        type_mismatches: result.differences.type_mismatches.iter().map(|(name, t1, t2)| {
                            crate::type_comparator::TypeMismatch {
                                property: name.clone(),
                                type1: t1.clone(),
                                type2: t2.clone(),
                            }
                        }).collect(),
                        optionality_differences: Vec::new(),
                    },
                };
                
                similar_pairs.push(UnifiedTypeComparisonPair {
                    type1: unified1.clone(),
                    type2: unified2.clone(),
                    result: type_result,
                });
            }
        }
    }
    
    // Sort by similarity (descending)
    similar_pairs.sort_by(|a, b| b.result.similarity.partial_cmp(&a.result.similarity).unwrap());
    
    similar_pairs
}
