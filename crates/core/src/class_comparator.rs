use crate::class_extractor::{ClassDefinition, ClassMethod, ClassProperty, MethodKind};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NormalizedClass {
    pub name: String,
    pub properties: HashMap<String, ClassProperty>,
    pub methods: HashMap<String, ClassMethod>,
    pub constructor_signature: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClassComparisonResult {
    pub similarity: f64,
    pub structural_similarity: f64,
    pub naming_similarity: f64,
    pub differences: ClassDifferences,
}

#[derive(Debug, Clone)]
pub struct ClassDifferences {
    pub missing_properties: Vec<String>,
    pub extra_properties: Vec<String>,
    pub missing_methods: Vec<String>,
    pub extra_methods: Vec<String>,
    pub property_type_mismatches: Vec<PropertyMismatch>,
    pub method_signature_mismatches: Vec<MethodMismatch>,
}

#[derive(Debug, Clone)]
pub struct PropertyMismatch {
    pub name: String,
    pub type1: String,
    pub type2: String,
}

#[derive(Debug, Clone)]
pub struct MethodMismatch {
    pub name: String,
    pub signature1: String,
    pub signature2: String,
}

#[derive(Debug, Clone)]
pub struct SimilarClassPair {
    pub class1: ClassDefinition,
    pub class2: ClassDefinition,
    pub result: ClassComparisonResult,
}

pub fn normalize_class(class: &ClassDefinition) -> NormalizedClass {
    let mut properties = HashMap::new();
    for prop in &class.properties {
        properties.insert(prop.name.clone(), prop.clone());
    }

    let mut methods = HashMap::new();
    for method in &class.methods {
        // Normalize method signature
        let normalized_method = ClassMethod {
            name: method.name.clone(),
            parameters: normalize_parameters(&method.parameters),
            return_type: normalize_type(&method.return_type),
            is_static: method.is_static,
            is_private: method.is_private,
            is_async: method.is_async,
            is_generator: method.is_generator,
            kind: method.kind.clone(),
        };
        methods.insert(method.name.clone(), normalized_method);
    }

    let constructor_signature = if class.constructor_params.is_empty() {
        "()".to_string()
    } else {
        format!("({})", class.constructor_params.join(", "))
    };

    NormalizedClass {
        name: class.name.clone(),
        properties,
        methods,
        constructor_signature,
        extends: class.extends.clone(),
        implements: class.implements.clone(),
    }
}

fn normalize_parameters(params: &[String]) -> Vec<String> {
    params.iter().map(|p| normalize_type(p)).collect()
}

fn normalize_type(type_str: &str) -> String {
    // Basic normalization - can be expanded
    type_str
        .replace("Array<", "[")
        .replace(">", "]")
        .replace(" ", "")
        .trim()
        .to_string()
}

pub fn compare_classes(
    class1: &ClassDefinition,
    class2: &ClassDefinition,
) -> ClassComparisonResult {
    let norm1 = normalize_class(class1);
    let norm2 = normalize_class(class2);

    // Calculate naming similarity
    let naming_similarity = calculate_name_similarity(&class1.name, &class2.name);

    // Calculate structural similarity
    let (structural_similarity, differences) = calculate_structural_similarity(&norm1, &norm2);

    // Combined similarity (weighted average)
    let similarity = 0.3 * naming_similarity + 0.7 * structural_similarity;

    ClassComparisonResult {
        similarity,
        structural_similarity,
        naming_similarity,
        differences,
    }
}

fn calculate_name_similarity(name1: &str, name2: &str) -> f64 {
    if name1 == name2 {
        return 1.0;
    }

    // Calculate Levenshtein distance
    let distance = levenshtein_distance(name1, name2);
    let max_len = name1.len().max(name2.len()) as f64;
    
    if max_len > 0.0 {
        1.0 - (distance as f64 / max_len)
    } else {
        1.0
    }
}

fn calculate_structural_similarity(
    class1: &NormalizedClass,
    class2: &NormalizedClass,
) -> (f64, ClassDifferences) {
    let mut missing_properties = Vec::new();
    let mut extra_properties = Vec::new();
    let mut property_type_mismatches = Vec::new();
    
    // Check properties
    let mut property_matches = 0;
    let mut property_total = 0;
    
    for (name, prop1) in &class1.properties {
        property_total += 1;
        if let Some(prop2) = class2.properties.get(name) {
            if prop1.type_annotation == prop2.type_annotation {
                property_matches += 1;
            } else {
                property_type_mismatches.push(PropertyMismatch {
                    name: name.clone(),
                    type1: prop1.type_annotation.clone(),
                    type2: prop2.type_annotation.clone(),
                });
            }
        } else {
            missing_properties.push(name.clone());
        }
    }
    
    for name in class2.properties.keys() {
        if !class1.properties.contains_key(name) {
            extra_properties.push(name.clone());
            property_total += 1;
        }
    }
    
    // Check methods
    let mut missing_methods = Vec::new();
    let mut extra_methods = Vec::new();
    let mut method_signature_mismatches = Vec::new();
    
    let mut method_matches = 0;
    let mut method_total = 0;
    
    for (name, method1) in &class1.methods {
        method_total += 1;
        if let Some(method2) = class2.methods.get(name) {
            let sig1 = format!("({}) => {}", method1.parameters.join(", "), method1.return_type);
            let sig2 = format!("({}) => {}", method2.parameters.join(", "), method2.return_type);
            
            if sig1 == sig2 {
                method_matches += 1;
            } else {
                method_signature_mismatches.push(MethodMismatch {
                    name: name.clone(),
                    signature1: sig1,
                    signature2: sig2,
                });
            }
        } else {
            missing_methods.push(name.clone());
        }
    }
    
    for name in class2.methods.keys() {
        if !class1.methods.contains_key(name) {
            extra_methods.push(name.clone());
            method_total += 1;
        }
    }
    
    // Calculate overall structural similarity
    let total_elements = property_total + method_total;
    let matched_elements = property_matches + method_matches;
    
    let structural_similarity = if total_elements > 0 {
        matched_elements as f64 / total_elements as f64
    } else {
        1.0
    };
    
    let differences = ClassDifferences {
        missing_properties,
        extra_properties,
        missing_methods,
        extra_methods,
        property_type_mismatches,
        method_signature_mismatches,
    };
    
    (structural_similarity, differences)
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    
    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1,
                    matrix[i + 1][j] + 1
                ),
                matrix[i][j] + cost
            );
        }
    }
    
    matrix[len1][len2]
}

pub fn find_similar_classes(
    classes: &[ClassDefinition],
    threshold: f64,
) -> Vec<SimilarClassPair> {
    let mut similar_pairs = Vec::new();
    
    for i in 0..classes.len() {
        for j in i + 1..classes.len() {
            let result = compare_classes(&classes[i], &classes[j]);
            
            if result.similarity >= threshold {
                similar_pairs.push(SimilarClassPair {
                    class1: classes[i].clone(),
                    class2: classes[j].clone(),
                    result,
                });
            }
        }
    }
    
    // Sort by similarity (highest first)
    similar_pairs.sort_by(|a, b| {
        b.result
            .similarity
            .partial_cmp(&a.result.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    
    similar_pairs
}

pub fn find_similar_classes_across_files(
    files: &[(String, String)],
    threshold: f64,
) -> Vec<SimilarClassPair> {
    let mut all_classes = Vec::new();
    
    for (file_path, content) in files {
        if let Ok(classes) = crate::class_extractor::extract_classes_from_code(content, file_path) {
            all_classes.extend(classes);
        }
    }
    
    find_similar_classes(&all_classes, threshold)
}