use indexmap::IndexMap;
use similarity_core::tree::TreeNode;
use similarity_core::tsed;
use std::rc::Rc;
use crate::shorthand_expander::expand_shorthand_properties;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct CssRule {
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub tree: Rc<TreeNode>,
    pub start_line: usize,
    pub end_line: usize,
}

// Serializable version of CssRule for JSON output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCssRule {
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub start_line: usize,
    pub end_line: usize,
}

impl From<&CssRule> for SerializableCssRule {
    fn from(rule: &CssRule) -> Self {
        SerializableCssRule {
            selector: rule.selector.clone(),
            declarations: rule.declarations.clone(),
            start_line: rule.start_line,
            end_line: rule.end_line,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssSimilarityResult {
    pub name1: String,
    pub name2: String,
    pub similarity: f64,
    pub file1: String,
    pub file2: String,
    pub start_line1: usize,
    pub end_line1: usize,
    pub start_line2: usize,
    pub end_line2: usize,
}

pub fn compare_css_rules(
    rules1: &[CssRule],
    rules2: &[CssRule],
    threshold: f64,
) -> Vec<CssSimilarityResult> {
    let mut results = Vec::new();
    
    for rule1 in rules1 {
        for rule2 in rules2 {
            let similarity = calculate_rule_similarity(rule1, rule2);
            
            if similarity >= threshold {
                results.push(CssSimilarityResult {
                    name1: rule1.selector.clone(),
                    name2: rule2.selector.clone(),
                    similarity,
                    file1: String::new(),
                    file2: String::new(),
                    start_line1: 0,
                    end_line1: 0,
                    start_line2: 0,
                    end_line2: 0,
                });
            }
        }
    }
    
    results
}

pub fn calculate_rule_similarity(
    rule1: &CssRule,
    rule2: &CssRule,
) -> f64 {
    let selector_similarity = calculate_selector_similarity(&rule1.selector, &rule2.selector);
    
    let ast_similarity = tsed::calculate_tsed(&rule1.tree, &rule2.tree, &tsed::TSEDOptions {
        size_penalty: true,
        ..Default::default()
    });
    
    // Expand shorthand properties before comparison
    let expanded_decls1 = expand_shorthand_properties(&rule1.declarations);
    let expanded_decls2 = expand_shorthand_properties(&rule2.declarations);
    let declaration_similarity = calculate_declaration_similarity(&expanded_decls1, &expanded_decls2);
    
    let weights = CssSimilarityWeights {
        selector: 0.4,
        ast: 0.0,
        declarations: 0.6,
    };
    
    weights.selector * selector_similarity
        + weights.ast * ast_similarity
        + weights.declarations * declaration_similarity
}

struct CssSimilarityWeights {
    selector: f64,
    ast: f64,
    declarations: f64,
}

pub fn calculate_selector_similarity(selector1: &str, selector2: &str) -> f64 {
    if selector1 == selector2 {
        return 1.0;
    }
    
    let tokens1 = tokenize_selector(selector1);
    let tokens2 = tokenize_selector(selector2);
    
    let common = tokens1.intersection(&tokens2).count() as f64;
    let total = tokens1.union(&tokens2).count() as f64;
    
    if total > 0.0 {
        common / total
    } else {
        0.0
    }
}

fn tokenize_selector(selector: &str) -> std::collections::HashSet<String> {
    let mut tokens = std::collections::HashSet::new();
    
    let parts: Vec<&str> = selector.split_whitespace()
        .flat_map(|s| s.split(','))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
        
    for part in parts {
        tokens.insert(part.to_string());
        
        let classes: Vec<String> = part.split('.')
            .filter(|s| !s.is_empty())
            .map(|s| format!(".{}", s))
            .collect();
        tokens.extend(classes);
        
        let ids: Vec<String> = part.split('#')
            .filter(|s| !s.is_empty())
            .map(|s| format!("#{}", s))
            .collect();
        tokens.extend(ids);
        
        if part.contains('[') && part.contains(']') {
            tokens.insert("[attr]".to_string());
        }
    }
    
    tokens
}

pub fn calculate_declaration_similarity(
    decls1: &[(String, String)],
    decls2: &[(String, String)],
) -> f64 {
    if decls1.is_empty() && decls2.is_empty() {
        return 1.0;
    }
    
    let map1: IndexMap<&str, &str> = decls1.iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let map2: IndexMap<&str, &str> = decls2.iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    
    let mut matching_properties = 0.0;
    let total_properties = map1.len().max(map2.len()) as f64;
    
    for (prop, value1) in &map1 {
        if let Some(value2) = map2.get(prop) {
            if value1 == value2 {
                matching_properties += 1.0;
            } else {
                matching_properties += calculate_value_similarity(value1, value2);
            }
        }
    }
    
    if total_properties > 0.0 {
        matching_properties / total_properties
    } else {
        0.0
    }
}

fn calculate_value_similarity(value1: &str, value2: &str) -> f64 {
    if value1 == value2 {
        return 1.0;
    }
    
    let norm1 = normalize_css_value(value1);
    let norm2 = normalize_css_value(value2);
    
    if norm1 == norm2 {
        return 0.9;
    }
    
    if is_color_value(&norm1) && is_color_value(&norm2) {
        return 0.7;
    }
    
    if is_numeric_value(&norm1) && is_numeric_value(&norm2) {
        return 0.6;
    }
    
    0.0
}

fn normalize_css_value(value: &str) -> String {
    value.trim()
        .to_lowercase()
        .replace("  ", " ")
        .replace(" !important", "")
}

fn is_color_value(value: &str) -> bool {
    value.starts_with('#')
        || value.starts_with("rgb")
        || value.starts_with("rgba")
        || value.starts_with("hsl")
        || value.starts_with("hsla")
        || matches!(
            value,
            "red" | "green" | "blue" | "white" | "black" | "gray" | "grey"
                | "yellow" | "orange" | "purple" | "pink"
        )
}

fn is_numeric_value(value: &str) -> bool {
    value.ends_with("px")
        || value.ends_with("em")
        || value.ends_with("rem")
        || value.ends_with('%')
        || value.ends_with("vh")
        || value.ends_with("vw")
        || value.parse::<f64>().is_ok()
}