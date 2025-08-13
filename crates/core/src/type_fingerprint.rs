use crate::type_extractor::TypeDefinition;
use std::collections::HashMap;

/// Generate a fingerprint for a type definition based on its properties
pub fn generate_type_fingerprint(type_def: &TypeDefinition) -> String {
    let mut fingerprint_parts = Vec::new();
    
    // Count property types
    let mut type_counts: HashMap<&str, usize> = HashMap::new();
    
    for prop in &type_def.properties {
        let type_str = prop.type_annotation.as_str();
        
        // Normalize common types for fingerprinting
        let normalized_type = match type_str {
            s if s.contains("string") => "string",
            s if s.contains("number") => "number",
            s if s.contains("boolean") => "boolean",
            s if s.contains("Date") => "Date",
            s if s.contains("[]") => "array",
            s if s.contains("{") && s.contains("}") => "object",
            _ => type_str,
        };
        
        *type_counts.entry(normalized_type).or_insert(0) += 1;
    }
    
    // Create fingerprint from sorted type counts
    let mut sorted_types: Vec<_> = type_counts.iter().collect();
    sorted_types.sort_by_key(|(k, _)| *k);
    
    for (type_name, count) in sorted_types {
        fingerprint_parts.push(format!("{}:{}", type_name, count));
    }
    
    // Add property count
    fingerprint_parts.push(format!("props:{}", type_def.properties.len()));
    
    // Add generic count if any
    if !type_def.generics.is_empty() {
        fingerprint_parts.push(format!("generics:{}", type_def.generics.len()));
    }
    
    fingerprint_parts.join(",")
}

/// Group types by their fingerprints for efficient comparison
pub fn group_types_by_fingerprint(types: &[TypeDefinition]) -> HashMap<String, Vec<usize>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    
    for (index, type_def) in types.iter().enumerate() {
        let fingerprint = generate_type_fingerprint(type_def);
        groups.entry(fingerprint).or_insert_with(Vec::new).push(index);
    }
    
    groups
}

/// Find similar types using fingerprint-based optimization
pub fn find_similar_types_with_fingerprint(
    types: &[TypeDefinition],
    threshold: f64,
    compare_fn: impl Fn(&TypeDefinition, &TypeDefinition) -> f64,
) -> Vec<(usize, usize, f64)> {
    let mut similar_pairs = Vec::new();
    let fingerprint_groups = group_types_by_fingerprint(types);
    
    // First, compare types within the same fingerprint group
    for (_fingerprint, indices) in &fingerprint_groups {
        if indices.len() < 2 {
            continue;
        }
        
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                let idx1 = indices[i];
                let idx2 = indices[j];
                let type1 = &types[idx1];
                let type2 = &types[idx2];
                
                // Skip if same type (same name and file)
                if type1.name == type2.name && type1.file_path == type2.file_path {
                    continue;
                }
                
                let similarity = compare_fn(type1, type2);
                if similarity >= threshold {
                    similar_pairs.push((idx1, idx2, similarity));
                }
            }
        }
    }
    
    // Then, compare types from different groups but with similar fingerprints
    let fingerprints: Vec<_> = fingerprint_groups.keys().collect();
    for i in 0..fingerprints.len() {
        for j in (i + 1)..fingerprints.len() {
            let fp1 = fingerprints[i];
            let fp2 = fingerprints[j];
            
            // Check if fingerprints are similar enough to warrant comparison
            if are_fingerprints_similar(fp1, fp2) {
                for &idx1 in &fingerprint_groups[fp1] {
                    for &idx2 in &fingerprint_groups[fp2] {
                        let type1 = &types[idx1];
                        let type2 = &types[idx2];
                        
                        // Skip if same type
                        if type1.name == type2.name && type1.file_path == type2.file_path {
                            continue;
                        }
                        
                        let similarity = compare_fn(type1, type2);
                        if similarity >= threshold {
                            similar_pairs.push((idx1, idx2, similarity));
                        }
                    }
                }
            }
        }
    }
    
    // Sort by similarity (descending)
    similar_pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    similar_pairs
}

/// Check if two fingerprints are similar enough to warrant detailed comparison
fn are_fingerprints_similar(fp1: &str, fp2: &str) -> bool {
    let parts1: HashMap<&str, &str> = fp1.split(',')
        .filter_map(|p| {
            let mut iter = p.split(':');
            Some((iter.next()?, iter.next()?))
        })
        .collect();
    
    let parts2: HashMap<&str, &str> = fp2.split(',')
        .filter_map(|p| {
            let mut iter = p.split(':');
            Some((iter.next()?, iter.next()?))
        })
        .collect();
    
    // Check property count difference
    if let (Some(props1), Some(props2)) = (parts1.get("props"), parts2.get("props")) {
        if let (Ok(count1), Ok(count2)) = (props1.parse::<usize>(), props2.parse::<usize>()) {
            let diff = (count1 as isize - count2 as isize).abs();
            // Allow up to 2 property difference
            if diff > 2 {
                return false;
            }
        }
    }
    
    // Check if they share common type patterns
    let mut common_types = 0;
    let mut total_types = 0;
    
    for (key, _) in &parts1 {
        if key != &"props" && key != &"generics" {
            total_types += 1;
            if parts2.contains_key(key) {
                common_types += 1;
            }
        }
    }
    
    // Consider similar if they share at least 50% of type patterns
    total_types > 0 && common_types as f64 / total_types as f64 >= 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_extractor::{PropertyDefinition, TypeKind};

    #[test]
    fn test_generate_fingerprint() {
        let type_def = TypeDefinition {
            name: "User".to_string(),
            kind: TypeKind::Interface,
            properties: vec![
                PropertyDefinition {
                    name: "id".to_string(),
                    type_annotation: "number".to_string(),
                    optional: false,
                    readonly: false,
                },
                PropertyDefinition {
                    name: "name".to_string(),
                    type_annotation: "string".to_string(),
                    optional: false,
                    readonly: false,
                },
                PropertyDefinition {
                    name: "email".to_string(),
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
        
        let fingerprint = generate_type_fingerprint(&type_def);
        assert!(fingerprint.contains("string:2"));
        assert!(fingerprint.contains("number:1"));
        assert!(fingerprint.contains("props:3"));
    }
    
    #[test]
    fn test_fingerprint_similarity() {
        // Similar fingerprints (one property difference)
        assert!(are_fingerprints_similar(
            "number:1,string:2,props:3",
            "number:1,string:2,props:4"
        ));
        
        // Different fingerprints (too many property differences)
        assert!(!are_fingerprints_similar(
            "number:1,string:2,props:3",
            "number:1,string:2,props:6"
        ));
        
        // Similar type patterns
        assert!(are_fingerprints_similar(
            "boolean:1,number:1,string:2,props:4",
            "boolean:1,number:2,string:1,props:4"
        ));
    }
}