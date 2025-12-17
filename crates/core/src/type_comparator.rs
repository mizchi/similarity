use crate::type_extractor::{TypeDefinition, TypeLiteralDefinition};
use crate::type_normalizer::{
    calculate_property_similarity, find_property_matches, normalize_type, NormalizationOptions,
    NormalizedType, PropertyMatch,
};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct TypeComparisonResult {
    pub similarity: f64,
    pub structural_similarity: f64,
    pub naming_similarity: f64,
    pub differences: TypeDifferences,
    pub matched_properties: Vec<MatchedProperty>,
}

#[derive(Debug, Clone)]
pub struct TypeDifferences {
    pub missing_properties: Vec<String>,
    pub extra_properties: Vec<String>,
    pub type_mismatches: Vec<TypeMismatch>,
    pub optionality_differences: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TypeMismatch {
    pub property: String,
    pub type1: String,
    pub type2: String,
}

#[derive(Debug, Clone)]
pub struct MatchedProperty {
    pub prop1: String,
    pub prop2: String,
    pub similarity: f64,
}

#[derive(Debug, Clone)]
pub struct TypeComparisonOptions {
    pub structural_weight: f64, // Weight for structural similarity (default: 0.6)
    pub naming_weight: f64,     // Weight for naming similarity (default: 0.4)
    pub property_match_threshold: f64, // Threshold for property matching (default: 0.7)
    pub allow_cross_kind_comparison: bool, // Allow interface vs type comparison (default: true)
    pub normalization_options: NormalizationOptions,
}

impl Default for TypeComparisonOptions {
    fn default() -> Self {
        Self {
            structural_weight: 0.6,
            naming_weight: 0.4,
            property_match_threshold: 0.7,
            allow_cross_kind_comparison: true,
            normalization_options: NormalizationOptions::default(),
        }
    }
}

/// Compare two type definitions and calculate their similarity
pub fn compare_types(
    type1: &TypeDefinition,
    type2: &TypeDefinition,
    options: &TypeComparisonOptions,
) -> TypeComparisonResult {
    // Early exit if cross-kind comparison is not allowed
    if !options.allow_cross_kind_comparison && type1.kind != type2.kind {
        return create_empty_comparison_result();
    }

    // Normalize both types
    let normalized1 = normalize_type(type1, &options.normalization_options);
    let normalized2 = normalize_type(type2, &options.normalization_options);

    // Find property matches
    let property_matches =
        find_property_matches(&normalized1, &normalized2, options.property_match_threshold);

    // Calculate structural similarity
    let structural_similarity =
        calculate_structural_similarity(&normalized1, &normalized2, &property_matches);

    // Calculate naming similarity
    let naming_similarity =
        calculate_naming_similarity(&normalized1, &normalized2, &property_matches);

    // Calculate overall similarity
    let similarity = (structural_similarity * options.structural_weight)
        + (naming_similarity * options.naming_weight);

    // Identify differences
    let differences = identify_differences(&normalized1, &normalized2, &property_matches);

    // Create matched properties result
    let matched_properties = property_matches
        .iter()
        .map(|m| MatchedProperty {
            prop1: m.prop1.clone(),
            prop2: m.prop2.clone(),
            similarity: m.overall_similarity,
        })
        .collect();

    TypeComparisonResult {
        similarity,
        structural_similarity,
        naming_similarity,
        differences,
        matched_properties,
    }
}

/// Calculate structural similarity between two normalized types
fn calculate_structural_similarity(
    type1: &NormalizedType,
    type2: &NormalizedType,
    matches: &[PropertyMatch],
) -> f64 {
    let total_props1 = type1.properties.len();
    let total_props2 = type2.properties.len();

    if total_props1 == 0 && total_props2 == 0 {
        return 1.0; // Both empty
    }

    if total_props1 == 0 || total_props2 == 0 {
        return 0.0; // One empty, one not
    }

    // Use the best matches (avoid double counting)
    let mut used_props1 = HashSet::new();
    let mut used_props2 = HashSet::new();
    let mut matched_count = 0;
    let mut total_match_score = 0.0;

    for property_match in matches {
        if !used_props1.contains(&property_match.prop1)
            && !used_props2.contains(&property_match.prop2)
        {
            used_props1.insert(property_match.prop1.clone());
            used_props2.insert(property_match.prop2.clone());
            matched_count += 1;
            total_match_score += property_match.overall_similarity;
        }
    }

    if matched_count == 0 {
        return 0.0;
    }

    // Calculate similarity based on matched properties
    let average_match_quality = total_match_score / matched_count as f64;
    let coverage_ratio = (matched_count * 2) as f64 / (total_props1 + total_props2) as f64;

    average_match_quality * coverage_ratio
}

/// Calculate naming similarity between two normalized types
fn calculate_naming_similarity(
    type1: &NormalizedType,
    type2: &NormalizedType,
    matches: &[PropertyMatch],
) -> f64 {
    if matches.is_empty() {
        return 0.0;
    }

    // Calculate average naming similarity from matches
    let naming_similarities: Vec<f64> = matches.iter().map(|m| m.name_similarity).collect();
    let average_naming_similarity =
        naming_similarities.iter().sum::<f64>() / naming_similarities.len() as f64;

    // Also consider type name similarity
    let type_name_similarity =
        calculate_property_similarity(&type1.original_name, &type2.original_name);

    // Weight property naming more heavily than type naming
    (average_naming_similarity * 0.8) + (type_name_similarity * 0.2)
}

/// Identify differences between two normalized types
fn identify_differences(
    type1: &NormalizedType,
    type2: &NormalizedType,
    matches: &[PropertyMatch],
) -> TypeDifferences {
    let matched_props1: HashSet<String> = matches.iter().map(|m| m.prop1.clone()).collect();
    let matched_props2: HashSet<String> = matches.iter().map(|m| m.prop2.clone()).collect();

    let missing_properties: Vec<String> =
        type1.properties.keys().filter(|prop| !matched_props1.contains(*prop)).cloned().collect();

    let extra_properties: Vec<String> =
        type2.properties.keys().filter(|prop| !matched_props2.contains(*prop)).cloned().collect();

    let mut type_mismatches = Vec::new();
    let mut optionality_differences = Vec::new();

    for property_match in matches {
        let empty_string = String::new();
        let type1_type = type1.properties.get(&property_match.prop1).unwrap_or(&empty_string);
        let type2_type = type2.properties.get(&property_match.prop2).unwrap_or(&empty_string);

        if type1_type != type2_type {
            type_mismatches.push(TypeMismatch {
                property: format!("{} -> {}", property_match.prop1, property_match.prop2),
                type1: type1_type.clone(),
                type2: type2_type.clone(),
            });
        }

        // Check optionality differences
        let is_optional1 = type1.optional_properties.contains(&property_match.prop1);
        let is_optional2 = type2.optional_properties.contains(&property_match.prop2);

        if is_optional1 != is_optional2 {
            optionality_differences
                .push(format!("{} -> {}", property_match.prop1, property_match.prop2));
        }
    }

    TypeDifferences {
        missing_properties,
        extra_properties,
        type_mismatches,
        optionality_differences,
    }
}

/// Create an empty comparison result for cases where comparison is not possible
fn create_empty_comparison_result() -> TypeComparisonResult {
    TypeComparisonResult {
        similarity: 0.0,
        structural_similarity: 0.0,
        naming_similarity: 0.0,
        differences: TypeDifferences {
            missing_properties: Vec::new(),
            extra_properties: Vec::new(),
            type_mismatches: Vec::new(),
            optionality_differences: Vec::new(),
        },
        matched_properties: Vec::new(),
    }
}

#[derive(Debug, Clone)]
pub struct SimilarTypePair {
    pub type1: TypeDefinition,
    pub type2: TypeDefinition,
    pub result: TypeComparisonResult,
}

#[derive(Debug, Clone)]
pub struct TypeLiteralComparisonPair {
    pub type_literal: TypeLiteralDefinition,
    pub type_definition: TypeDefinition,
    pub result: TypeComparisonResult,
}

/// Compare multiple types and find similar pairs
pub fn find_similar_types(
    types: &[TypeDefinition],
    threshold: f64,
    options: &TypeComparisonOptions,
) -> Vec<SimilarTypePair> {
    let mut similar_pairs = Vec::new();

    for i in 0..types.len() {
        for j in (i + 1)..types.len() {
            let type1 = &types[i];
            let type2 = &types[j];

            // Skip if same type (same name and file)
            if type1.name == type2.name && type1.file_path == type2.file_path {
                continue;
            }

            let result = compare_types(type1, type2, options);

            if result.similarity >= threshold {
                similar_pairs.push(SimilarTypePair {
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

/// Find duplicate types (very high similarity)
pub fn find_duplicate_types(
    types: &[TypeDefinition],
    threshold: f64,
    options: &TypeComparisonOptions,
) -> Vec<SimilarTypePair> {
    find_similar_types(types, threshold.max(0.9), options)
}

/// Group similar types into clusters
pub fn group_similar_types(
    types: &[TypeDefinition],
    threshold: f64,
    options: &TypeComparisonOptions,
) -> Vec<Vec<TypeDefinition>> {
    let similar_pairs = find_similar_types(types, threshold, options);
    let mut groups: Vec<Vec<TypeDefinition>> = Vec::new();
    let mut processed = HashSet::new();

    for pair in similar_pairs {
        let type1_id = format!("{}:{}", pair.type1.file_path, pair.type1.name);
        let type2_id = format!("{}:{}", pair.type2.file_path, pair.type2.name);

        if processed.contains(&type1_id) || processed.contains(&type2_id) {
            continue;
        }

        // Find existing group that contains either type
        let mut found_group = None;
        for (group_idx, group) in groups.iter().enumerate() {
            if group.iter().any(|t| {
                let id = format!("{}:{}", t.file_path, t.name);
                id == type1_id || id == type2_id
            }) {
                found_group = Some(group_idx);
                break;
            }
        }

        match found_group {
            Some(group_idx) => {
                // Add to existing group
                if !groups[group_idx].iter().any(|t| {
                    let id = format!("{}:{}", t.file_path, t.name);
                    id == type1_id
                }) {
                    groups[group_idx].push(pair.type1.clone());
                }
                if !groups[group_idx].iter().any(|t| {
                    let id = format!("{}:{}", t.file_path, t.name);
                    id == type2_id
                }) {
                    groups[group_idx].push(pair.type2.clone());
                }
            }
            None => {
                // Create new group
                groups.push(vec![pair.type1.clone(), pair.type2.clone()]);
            }
        }

        processed.insert(type1_id);
        processed.insert(type2_id);
    }

    // Filter out groups with only one type
    groups.into_iter().filter(|group| group.len() > 1).collect()
}

/// Compare type literal with type definition
pub fn compare_type_literal_with_type(
    type_literal: &TypeLiteralDefinition,
    type_definition: &TypeDefinition,
    options: &TypeComparisonOptions,
) -> TypeComparisonResult {
    // Convert type literal to TypeDefinition for comparison
    let temp_type_def = TypeDefinition {
        name: type_literal.name.clone(),
        kind: crate::type_extractor::TypeKind::TypeLiteral,
        properties: type_literal.properties.clone(),
        generics: Vec::new(),
        extends: Vec::new(),
        start_line: type_literal.start_line,
        end_line: type_literal.end_line,
        file_path: type_literal.file_path.clone(),
        has_ignore_directive: false,
    };

    compare_types(&temp_type_def, type_definition, options)
}

/// Find type literals that are similar to existing type definitions
pub fn find_similar_type_literals(
    type_literals: &[TypeLiteralDefinition],
    type_definitions: &[TypeDefinition],
    threshold: f64,
    options: &TypeComparisonOptions,
) -> Vec<TypeLiteralComparisonPair> {
    let mut similar_pairs = Vec::new();

    for type_literal in type_literals {
        for type_definition in type_definitions {
            // Skip if same file and overlapping lines (avoid self-comparison)
            if type_literal.file_path == type_definition.file_path {
                let literal_range = type_literal.start_line..=type_literal.end_line;
                let def_range = type_definition.start_line..=type_definition.end_line;

                // Check if ranges overlap
                if literal_range.start() <= def_range.end()
                    && def_range.start() <= literal_range.end()
                {
                    continue;
                }
            }

            let result = compare_type_literal_with_type(type_literal, type_definition, options);

            if result.similarity >= threshold {
                similar_pairs.push(TypeLiteralComparisonPair {
                    type_literal: type_literal.clone(),
                    type_definition: type_definition.clone(),
                    result,
                });
            }
        }
    }

    // Sort by similarity (descending)
    similar_pairs.sort_by(|a, b| b.result.similarity.partial_cmp(&a.result.similarity).unwrap());

    similar_pairs
}

/// Find type literals that are similar to each other
pub fn find_similar_type_literals_pairs(
    type_literals: &[TypeLiteralDefinition],
    threshold: f64,
    options: &TypeComparisonOptions,
) -> Vec<(TypeLiteralDefinition, TypeLiteralDefinition, TypeComparisonResult)> {
    let mut similar_pairs = Vec::new();

    for i in 0..type_literals.len() {
        for j in (i + 1)..type_literals.len() {
            let type_literal1 = &type_literals[i];
            let type_literal2 = &type_literals[j];

            // Skip if same context (avoid comparing function return type with its parameter)
            if type_literal1.file_path == type_literal2.file_path
                && type_literal1.name == type_literal2.name
            {
                continue;
            }

            let result = compare_type_literal_with_type(
                type_literal1,
                &TypeDefinition {
                    name: type_literal2.name.clone(),
                    kind: crate::type_extractor::TypeKind::TypeLiteral,
                    properties: type_literal2.properties.clone(),
                    generics: Vec::new(),
                    extends: Vec::new(),
                    start_line: type_literal2.start_line,
                    end_line: type_literal2.end_line,
                    file_path: type_literal2.file_path.clone(),
                    has_ignore_directive: false,
                },
                options,
            );

            if result.similarity >= threshold {
                similar_pairs.push((type_literal1.clone(), type_literal2.clone(), result));
            }
        }
    }

    // Sort by similarity (descending)
    similar_pairs.sort_by(|a, b| b.2.similarity.partial_cmp(&a.2.similarity).unwrap());

    similar_pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_extractor::{PropertyDefinition, TypeDefinition, TypeKind};

    fn create_test_type(name: &str, properties: Vec<(&str, &str, bool, bool)>) -> TypeDefinition {
        TypeDefinition {
            name: name.to_string(),
            kind: TypeKind::Interface,
            properties: properties
                .into_iter()
                .map(|(name, type_annotation, optional, readonly)| PropertyDefinition {
                    name: name.to_string(),
                    type_annotation: type_annotation.to_string(),
                    optional,
                    readonly,
                })
                .collect(),
            generics: Vec::new(),
            extends: Vec::new(),
            start_line: 1,
            end_line: 10,
            file_path: "test.ts".to_string(),
            has_ignore_directive: false,
        }
    }

    #[test]
    fn test_compare_identical_types() {
        let type1 = create_test_type(
            "User",
            vec![("id", "string", false, false), ("name", "string", false, false)],
        );
        let type2 = create_test_type(
            "Person",
            vec![("id", "string", false, false), ("name", "string", false, false)],
        );

        let options = TypeComparisonOptions::default();
        let result = compare_types(&type1, &type2, &options);

        assert!(result.similarity > 0.9);
        assert_eq!(result.matched_properties.len(), 2);
    }

    #[test]
    fn test_compare_similar_types() {
        let type1 = create_test_type(
            "User",
            vec![("id", "string", false, false), ("name", "string", false, false)],
        );
        let type2 = create_test_type(
            "Person",
            vec![("id", "string", false, false), ("fullName", "string", false, false)],
        );

        let options = TypeComparisonOptions::default();
        let result = compare_types(&type1, &type2, &options);

        assert!(result.similarity > 0.5);
        assert!(result.similarity < 0.9);
        assert_eq!(result.matched_properties.len(), 1); // Only "id" matches
    }

    #[test]
    fn test_compare_different_types() {
        let type1 = create_test_type(
            "User",
            vec![("id", "string", false, false), ("name", "string", false, false)],
        );
        let type2 = create_test_type(
            "Product",
            vec![("sku", "string", false, false), ("price", "number", false, false)],
        );

        let options = TypeComparisonOptions::default();
        let result = compare_types(&type1, &type2, &options);

        assert!(result.similarity < 0.5);
    }

    #[test]
    fn test_find_similar_types() {
        let types = vec![
            create_test_type(
                "User",
                vec![("id", "string", false, false), ("name", "string", false, false)],
            ),
            create_test_type(
                "Person",
                vec![("id", "string", false, false), ("name", "string", false, false)],
            ),
            create_test_type(
                "Product",
                vec![("sku", "string", false, false), ("price", "number", false, false)],
            ),
        ];

        let options = TypeComparisonOptions::default();
        let similar_pairs = find_similar_types(&types, 0.7, &options);

        assert_eq!(similar_pairs.len(), 1);
        assert_eq!(similar_pairs[0].type1.name, "User");
        assert_eq!(similar_pairs[0].type2.name, "Person");
    }
}
