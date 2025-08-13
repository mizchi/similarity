use crate::type_extractor::{TypeDefinition, TypeKind};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct NormalizedType {
    pub properties: HashMap<String, String>, // プロパティ名 -> 型
    pub optional_properties: HashSet<String>,
    pub readonly_properties: HashSet<String>,
    pub signature: String, // 正規化された型シグネチャ
    pub original_name: String,
    pub kind: TypeKind,
}

#[derive(Debug, Clone)]
pub struct NormalizationOptions {
    pub ignore_property_order: bool,
    pub ignore_optional_modifiers: bool,
    pub ignore_readonly_modifiers: bool,
    pub normalize_type_names: bool,
}

impl Default for NormalizationOptions {
    fn default() -> Self {
        Self {
            ignore_property_order: true,
            ignore_optional_modifiers: false,
            ignore_readonly_modifiers: true,
            normalize_type_names: true,
        }
    }
}

/// Normalize a type definition for comparison
pub fn normalize_type(type_def: &TypeDefinition, options: &NormalizationOptions) -> NormalizedType {
    let mut properties = HashMap::new();
    let mut optional_properties = HashSet::new();
    let mut readonly_properties = HashSet::new();

    // Process each property
    for prop in &type_def.properties {
        let normalized_prop_name = prop.name.to_lowercase().trim().to_string();
        let normalized_type = if options.normalize_type_names {
            normalize_type_name(&prop.type_annotation)
        } else {
            prop.type_annotation.clone()
        };

        properties.insert(normalized_prop_name.clone(), normalized_type);

        if prop.optional && !options.ignore_optional_modifiers {
            optional_properties.insert(normalized_prop_name.clone());
        }

        if prop.readonly && !options.ignore_readonly_modifiers {
            readonly_properties.insert(normalized_prop_name);
        }
    }

    // Generate normalized signature
    let signature = generate_type_signature(
        &properties,
        &optional_properties,
        &readonly_properties,
        options.ignore_property_order,
    );

    NormalizedType {
        properties,
        optional_properties,
        readonly_properties,
        signature,
        original_name: type_def.name.clone(),
        kind: type_def.kind.clone(),
    }
}

/// Normalize type names for consistent comparison
pub fn normalize_type_name(type_name: &str) -> String {
    // Remove extra whitespace
    let mut normalized = type_name.trim().to_string();

    // Normalize primitive types
    let type_map = [
        ("String", "string"),
        ("Number", "number"),
        ("Boolean", "boolean"),
        ("Object", "object"),
        ("Array", "array"),
        ("Function", "function"),
    ];

    // Normalize array syntax: T[] vs Array<T> - do this before type replacements
    if normalized.starts_with("Array<") && normalized.ends_with(">") {
        let inner = &normalized[6..normalized.len() - 1];
        // Check if the inner type contains balanced angle brackets
        let mut bracket_count = 0;
        let mut valid = true;
        for ch in inner.chars() {
            match ch {
                '<' => bracket_count += 1,
                '>' => {
                    bracket_count -= 1;
                    if bracket_count < 0 {
                        valid = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if valid && bracket_count == 0 {
            normalized = format!("{}[]", inner);
        }
    }

    // Replace known type aliases
    for (original, replacement) in &type_map {
        normalized = normalized.replace(original, replacement);
    }

    // Normalize function types: convert arrow function to method syntax
    // Pattern 1: () => ReturnType -> (): ReturnType
    // Pattern 2: (param: Type) => ReturnType -> (param: Type): ReturnType
    normalized = normalize_function_syntax(&normalized);

    // Sort union types for consistent comparison
    if normalized.contains(" | ") {
        let mut union_types: Vec<&str> = normalized.split(" | ").map(|t| t.trim()).collect();
        union_types.sort();
        normalized = union_types.join(" | ");
    }

    // Sort intersection types for consistent comparison
    if normalized.contains(" & ") {
        let mut intersection_types: Vec<&str> = normalized.split(" & ").map(|t| t.trim()).collect();
        intersection_types.sort();
        normalized = intersection_types.join(" & ");
    }

    normalized
}

/// Normalize function syntax to a consistent format
/// Converts arrow functions to method syntax: `() => T` becomes `(): T`
fn normalize_function_syntax(type_str: &str) -> String {
    let mut result = type_str.to_string();

    // Find and replace arrow function patterns
    // We need to be careful with nested types and preserve them correctly

    // Simple pattern: () => Type
    if let Some(arrow_pos) = result.find(" => ") {
        // Check if this is a function type at the top level
        // Find the matching opening parenthesis
        let before_arrow = &result[..arrow_pos];

        // Count parentheses to find the start of the function signature
        let mut paren_count = 0;
        let mut func_start = None;

        for (i, ch) in before_arrow.chars().rev().enumerate() {
            match ch {
                ')' => paren_count += 1,
                '(' => {
                    paren_count -= 1;
                    if paren_count == 0 {
                        func_start = Some(arrow_pos - i - 1);
                        break;
                    }
                }
                _ => {
                    // If we hit a non-parenthesis character while not inside parentheses,
                    // this might not be a simple function type
                    if paren_count == 0 && !ch.is_whitespace() {
                        break;
                    }
                }
            }
        }

        if let Some(start) = func_start {
            // Check if this looks like a function signature
            let func_params = &result[start..arrow_pos].trim();
            if func_params.starts_with('(') && func_params.ends_with(')') {
                // Extract return type (everything after =>)
                let return_type = result[arrow_pos + 4..].trim();

                // Build the normalized version
                result = format!(
                    "{}{}: {}{}",
                    &result[..start],
                    func_params,
                    return_type,
                    "" // We might have more content after
                );
            }
        }
    }

    result
}

/// Generate a normalized signature for the type
fn generate_type_signature(
    properties: &HashMap<String, String>,
    optional_properties: &HashSet<String>,
    readonly_properties: &HashSet<String>,
    ignore_order: bool,
) -> String {
    let mut prop_entries: Vec<(&String, &String)> = properties.iter().collect();

    if ignore_order {
        prop_entries.sort_by(|a, b| a.0.cmp(b.0));
    }

    let prop_strings: Vec<String> = prop_entries
        .iter()
        .map(|(name, type_annotation)| {
            let mut prop_str = String::new();

            if readonly_properties.contains(*name) {
                prop_str.push_str("readonly ");
            }

            prop_str.push_str(name);

            if optional_properties.contains(*name) {
                prop_str.push('?');
            }

            prop_str.push_str(": ");
            prop_str.push_str(type_annotation);

            prop_str
        })
        .collect();

    format!("{{ {} }}", prop_strings.join("; "))
}

/// Calculate similarity between two property names using Levenshtein distance
pub fn calculate_property_similarity(prop1: &str, prop2: &str) -> f64 {
    if prop1 == prop2 {
        return 1.0;
    }

    let normalized1 = prop1.to_lowercase();
    let normalized2 = prop2.to_lowercase();
    let normalized1 = normalized1.trim();
    let normalized2 = normalized2.trim();

    if normalized1 == normalized2 {
        return 0.95;
    }

    let max_length = normalized1.len().max(normalized2.len());
    if max_length == 0 {
        return 1.0;
    }

    let distance = levenshtein_distance(normalized1, normalized2);
    (1.0 - (distance as f64 / max_length as f64)).max(0.0)
}

/// Calculate similarity between two type strings
pub fn calculate_type_similarity(type1: &str, type2: &str) -> f64 {
    let normalized1 = normalize_type_name(type1);
    let normalized2 = normalize_type_name(type2);

    if normalized1 == normalized2 {
        return 1.0;
    }

    // Handle union types specially
    if normalized1.contains(" | ") || normalized2.contains(" | ") {
        return calculate_union_type_similarity(&normalized1, &normalized2);
    }

    // Handle intersection types specially
    if normalized1.contains(" & ") || normalized2.contains(" & ") {
        return calculate_intersection_type_similarity(&normalized1, &normalized2);
    }

    // For other types, use string similarity
    let max_length = normalized1.len().max(normalized2.len());
    if max_length == 0 {
        return 1.0;
    }

    let distance = levenshtein_distance(&normalized1, &normalized2);
    (1.0 - (distance as f64 / max_length as f64)).max(0.0)
}

/// Calculate similarity between union types
fn calculate_union_type_similarity(type1: &str, type2: &str) -> f64 {
    let union1: Vec<&str> = if type1.contains(" | ") {
        type1.split(" | ").map(|t| t.trim()).collect()
    } else {
        vec![type1]
    };

    let union2: Vec<&str> = if type2.contains(" | ") {
        type2.split(" | ").map(|t| t.trim()).collect()
    } else {
        vec![type2]
    };

    let common_types: Vec<&str> =
        union1.iter().filter(|t1| union2.iter().any(|t2| *t1 == t2)).copied().collect();

    if union1.is_empty() && union2.is_empty() {
        1.0
    } else {
        (common_types.len() * 2) as f64 / (union1.len() + union2.len()) as f64
    }
}

/// Calculate similarity between intersection types
fn calculate_intersection_type_similarity(type1: &str, type2: &str) -> f64 {
    let intersection1: Vec<&str> = if type1.contains(" & ") {
        type1.split(" & ").map(|t| t.trim()).collect()
    } else {
        vec![type1]
    };

    let intersection2: Vec<&str> = if type2.contains(" & ") {
        type2.split(" & ").map(|t| t.trim()).collect()
    } else {
        vec![type2]
    };

    let common_types: Vec<&str> = intersection1
        .iter()
        .filter(|t1| intersection2.iter().any(|t2| *t1 == t2))
        .copied()
        .collect();

    if intersection1.is_empty() && intersection2.is_empty() {
        1.0
    } else {
        (common_types.len() * 2) as f64 / (intersection1.len() + intersection2.len()) as f64
    }
}

#[derive(Debug, Clone)]
pub struct PropertyMatch {
    pub prop1: String,
    pub prop2: String,
    pub name_similarity: f64,
    pub type_similarity: f64,
    pub overall_similarity: f64,
}

/// Find the best property matches between two normalized types
pub fn find_property_matches(
    type1: &NormalizedType,
    type2: &NormalizedType,
    _threshold: f64, // Keep for API compatibility but not used
) -> Vec<PropertyMatch> {
    let mut matches = Vec::new();

    // Only match properties with exactly the same name
    for (prop1, type1_annotation) in &type1.properties {
        if let Some(type2_annotation) = type2.properties.get(prop1) {
            let name_similarity = 1.0; // Exact match only
            let type_similarity = calculate_type_similarity(type1_annotation, type2_annotation);

            // Since names must match exactly, overall similarity is just type similarity
            let overall_similarity = type_similarity;

            matches.push(PropertyMatch {
                prop1: prop1.clone(),
                prop2: prop1.clone(), // Same property name
                name_similarity,
                type_similarity,
                overall_similarity,
            });
        }
    }

    // Sort by overall similarity (descending)
    matches.sort_by(|a, b| b.overall_similarity.partial_cmp(&a.overall_similarity).unwrap());

    matches
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    #[allow(clippy::needless_range_loop)]
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };

            matrix[i][j] = (matrix[i - 1][j] + 1) // deletion
                .min(matrix[i][j - 1] + 1) // insertion
                .min(matrix[i - 1][j - 1] + cost); // substitution
        }
    }

    matrix[len1][len2]
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
        }
    }

    #[test]
    fn test_normalize_type() {
        let type_def = create_test_type(
            "User",
            vec![
                ("id", "string", false, false),
                ("name", "string", false, false),
                ("age", "number", true, false),
                ("email", "string", false, true),
            ],
        );

        let options = NormalizationOptions::default();
        let normalized = normalize_type(&type_def, &options);

        assert_eq!(normalized.original_name, "User");
        assert_eq!(normalized.properties.len(), 4);
        assert_eq!(normalized.optional_properties.len(), 1); // "age" is optional, and ignore_optional_modifiers is false by default
        assert!(normalized.readonly_properties.is_empty()); // ignore_readonly_modifiers is true by default
    }

    #[test]
    fn test_normalize_type_name() {
        assert_eq!(normalize_type_name("String"), "string");
        assert_eq!(normalize_type_name("Array<string>"), "string[]");
        assert_eq!(normalize_type_name("Array<number>"), "number[]");
        assert_eq!(normalize_type_name("number | string"), "number | string");
        assert_eq!(normalize_type_name("string | number"), "number | string"); // sorted
    }

    #[test]
    fn test_calculate_property_similarity() {
        assert_eq!(calculate_property_similarity("name", "name"), 1.0);
        assert_eq!(calculate_property_similarity("name", "Name"), 0.95);
        assert!(calculate_property_similarity("name", "fullName") > 0.0);
        assert!(calculate_property_similarity("name", "fullName") < 1.0);
    }

    #[test]
    fn test_calculate_type_similarity() {
        assert_eq!(calculate_type_similarity("string", "string"), 1.0);
        assert_eq!(calculate_type_similarity("String", "string"), 1.0);
        assert!(calculate_type_similarity("string", "number") < 1.0);
    }

    #[test]
    fn test_union_type_similarity() {
        assert_eq!(calculate_union_type_similarity("string | number", "number | string"), 1.0);
        assert!(calculate_union_type_similarity("string | number", "string | boolean") > 0.0);
        assert!(calculate_union_type_similarity("string | number", "string | boolean") < 1.0);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "ab"), 1);
        assert_eq!(levenshtein_distance("abc", "def"), 3);
    }
}
