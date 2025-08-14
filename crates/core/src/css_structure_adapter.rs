use crate::structure_comparator::{
    Structure, StructureIdentifier, StructureKind, StructureMember, StructureMetadata,
    SourceLocation, StructureComparator, ComparisonOptions, StructureComparisonResult,
};
use std::collections::HashMap;

/// CSS rule definition for structure comparison
#[derive(Debug, Clone)]
pub struct CssStructDef {
    pub selector: String,
    pub declarations: Vec<(String, String)>,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub media_query: Option<String>,
    pub parent_selectors: Vec<String>,
}

/// CSS構造を一般構造に変換
impl From<CssStructDef> for Structure {
    fn from(css_rule: CssStructDef) -> Self {
        let kind = if css_rule.selector.starts_with('.') {
            StructureKind::CssClass
        } else {
            StructureKind::CssRule
        };
        
        let mut members = Vec::new();
        
        // CSSプロパティをメンバーとして追加
        for (property, value) in css_rule.declarations {
            members.push(StructureMember {
                name: property.clone(),
                value_type: categorize_css_value(&value),
                modifiers: vec![],
                nested: None,
            });
        }
        
        // メディアクエリがあれば特殊メンバーとして追加
        if let Some(media) = &css_rule.media_query {
            members.push(StructureMember {
                name: "@media".to_string(),
                value_type: media.clone(),
                modifiers: vec!["media-query".to_string()],
                nested: None,
            });
        }
        
        // 親セレクタがあれば特殊メンバーとして追加
        if !css_rule.parent_selectors.is_empty() {
            members.push(StructureMember {
                name: "@parent".to_string(),
                value_type: css_rule.parent_selectors.join(" "),
                modifiers: vec!["parent-selector".to_string()],
                nested: None,
            });
        }
        
        Structure {
            identifier: StructureIdentifier {
                name: css_rule.selector.clone(),
                kind,
                namespace: Some(css_rule.file_path.clone()),
            },
            members,
            metadata: StructureMetadata {
                location: SourceLocation {
                    file_path: css_rule.file_path,
                    start_line: css_rule.start_line,
                    end_line: css_rule.end_line,
                },
                generics: vec![],
                extends: vec![],
                visibility: None,
            },
        }
    }
}

/// CSS値をカテゴライズ（型として扱う）
fn categorize_css_value(value: &str) -> String {
    let value = value.trim();
    
    // Color values
    if value.starts_with('#') || 
       value.starts_with("rgb") || 
       value.starts_with("hsl") ||
       value.starts_with("rgba") ||
       value.starts_with("hsla") ||
       is_named_color(value) {
        return "color".to_string();
    }
    
    // Length values
    if value.ends_with("px") || 
       value.ends_with("em") || 
       value.ends_with("rem") ||
       value.ends_with("%") ||
       value.ends_with("vh") ||
       value.ends_with("vw") ||
       value.ends_with("pt") ||
       value.ends_with("cm") ||
       value.ends_with("mm") {
        return "length".to_string();
    }
    
    // Time values
    if value.ends_with("s") || value.ends_with("ms") {
        return "time".to_string();
    }
    
    // Font values
    if is_font_family(value) {
        return "font-family".to_string();
    }
    
    // Number values
    if value.parse::<f64>().is_ok() {
        return "number".to_string();
    }
    
    // URL values
    if value.starts_with("url(") {
        return "url".to_string();
    }
    
    // Keyword values
    if is_css_keyword(value) {
        return "keyword".to_string();
    }
    
    // Default
    "value".to_string()
}

fn is_named_color(value: &str) -> bool {
    matches!(value, 
        "red" | "green" | "blue" | "black" | "white" | "gray" | "grey" |
        "yellow" | "orange" | "purple" | "pink" | "brown" | "cyan" |
        "magenta" | "lime" | "indigo" | "violet" | "transparent" | "currentColor"
    )
}

fn is_font_family(value: &str) -> bool {
    value.contains("serif") || 
    value.contains("sans-serif") ||
    value.contains("monospace") ||
    value.contains("cursive") ||
    value.contains("fantasy") ||
    value.contains("Arial") ||
    value.contains("Helvetica") ||
    value.contains("Times") ||
    value.contains("Courier") ||
    value.contains("Georgia") ||
    value.contains("Verdana") ||
    value.contains('"') || 
    value.contains('\'')
}

fn is_css_keyword(value: &str) -> bool {
    matches!(value,
        "none" | "auto" | "inherit" | "initial" | "unset" | "normal" |
        "bold" | "italic" | "underline" | "center" | "left" | "right" |
        "top" | "bottom" | "middle" | "baseline" | "flex" | "grid" |
        "block" | "inline" | "inline-block" | "table" | "relative" |
        "absolute" | "fixed" | "sticky" | "static" | "hidden" | "visible" |
        "scroll" | "pointer" | "default" | "solid" | "dashed" | "dotted"
    )
}

/// CSS用の比較エンジン
pub struct CssStructureComparator {
    pub comparator: StructureComparator,
}

impl CssStructureComparator {
    pub fn new() -> Self {
        let options = ComparisonOptions {
            name_weight: 0.4,  // セレクタの重要度を高める
            structure_weight: 0.6,  // プロパティの重要度
            threshold: 0.7,
            fuzzy_matching: true,  // CSSでは類似セレクタも検出したい
            ignore_order: true,  // CSSプロパティの順序は無視
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
    
    /// CSSルールを比較
    pub fn compare_rules(&mut self, rule1: &CssStructDef, rule2: &CssStructDef) -> StructureComparisonResult {
        let struct1 = Structure::from(rule1.clone());
        let struct2 = Structure::from(rule2.clone());
        self.comparator.compare(&struct1, &struct2)
    }
    
    /// セレクタの正規化（比較用）
    pub fn normalize_selector(selector: &str) -> String {
        // Remove whitespace variations
        let mut normalized = selector.trim().to_string();
        
        // Normalize multiple spaces to single space
        while normalized.contains("  ") {
            normalized = normalized.replace("  ", " ");
        }
        
        // Normalize combinators
        normalized = normalized.replace(" > ", ">")
                              .replace(" + ", "+")
                              .replace(" ~ ", "~");
        
        // Sort comma-separated selectors
        if normalized.contains(',') {
            let mut parts: Vec<_> = normalized.split(',')
                                             .map(|s| s.trim())
                                             .collect();
            parts.sort();
            normalized = parts.join(", ");
        }
        
        normalized
    }
    
    /// プロパティを正規化して比較しやすくする
    pub fn normalize_properties(declarations: &[(String, String)]) -> Vec<(String, String)> {
        let mut normalized = Vec::new();
        let mut property_map: HashMap<String, String> = HashMap::new();
        
        for (prop, value) in declarations {
            // ショートハンドプロパティを展開
            if is_shorthand_property(prop) {
                let expanded = expand_shorthand(prop, value);
                for (exp_prop, exp_value) in expanded {
                    property_map.insert(exp_prop, exp_value);
                }
            } else {
                property_map.insert(prop.clone(), value.clone());
            }
        }
        
        // ソートして一貫性を保つ
        let mut entries: Vec<_> = property_map.into_iter().collect();
        entries.sort_by_key(|(k, _)| k.clone());
        
        for (prop, value) in entries {
            normalized.push((prop, normalize_css_value(&value)));
        }
        
        normalized
    }
}

fn is_shorthand_property(property: &str) -> bool {
    matches!(property,
        "margin" | "padding" | "border" | "border-radius" | 
        "background" | "font" | "flex" | "grid" |
        "animation" | "transition" | "transform"
    )
}

fn expand_shorthand(property: &str, value: &str) -> Vec<(String, String)> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    
    match property {
        "margin" | "padding" => {
            let prefix = property;
            match parts.len() {
                1 => vec![
                    (format!("{}-top", prefix), parts[0].to_string()),
                    (format!("{}-right", prefix), parts[0].to_string()),
                    (format!("{}-bottom", prefix), parts[0].to_string()),
                    (format!("{}-left", prefix), parts[0].to_string()),
                ],
                2 => vec![
                    (format!("{}-top", prefix), parts[0].to_string()),
                    (format!("{}-right", prefix), parts[1].to_string()),
                    (format!("{}-bottom", prefix), parts[0].to_string()),
                    (format!("{}-left", prefix), parts[1].to_string()),
                ],
                3 => vec![
                    (format!("{}-top", prefix), parts[0].to_string()),
                    (format!("{}-right", prefix), parts[1].to_string()),
                    (format!("{}-bottom", prefix), parts[2].to_string()),
                    (format!("{}-left", prefix), parts[1].to_string()),
                ],
                4 => vec![
                    (format!("{}-top", prefix), parts[0].to_string()),
                    (format!("{}-right", prefix), parts[1].to_string()),
                    (format!("{}-bottom", prefix), parts[2].to_string()),
                    (format!("{}-left", prefix), parts[3].to_string()),
                ],
                _ => vec![(property.to_string(), value.to_string())],
            }
        }
        "border" => {
            // 簡略化: border: 1px solid red -> border-width, border-style, border-color
            vec![
                ("border-width".to_string(), value.to_string()),
                ("border-style".to_string(), value.to_string()),
                ("border-color".to_string(), value.to_string()),
            ]
        }
        _ => vec![(property.to_string(), value.to_string())],
    }
}

fn normalize_css_value(value: &str) -> String {
    let mut normalized = value.trim().to_lowercase();
    
    // Normalize hex colors to lowercase
    if normalized.starts_with('#') {
        // Convert 3-digit hex to 6-digit
        if normalized.len() == 4 {
            let r = &normalized[1..2];
            let g = &normalized[2..3];
            let b = &normalized[3..4];
            normalized = format!("#{}{}{}{}{}{}", r, r, g, g, b, b);
        }
    }
    
    // Normalize 0 values
    if normalized == "0px" || normalized == "0em" || normalized == "0rem" || 
       normalized == "0%" || normalized == "0pt" {
        normalized = "0".to_string();
    }
    
    normalized
}

/// 複数のCSSルールを効率的に比較
pub struct CssBatchComparator {
    comparator: CssStructureComparator,
    fingerprint_cache: HashMap<String, Vec<Structure>>,
}

impl CssBatchComparator {
    pub fn new() -> Self {
        Self {
            comparator: CssStructureComparator::new(),
            fingerprint_cache: HashMap::new(),
        }
    }
    
    /// CSSルールをフィンガープリントでグループ化
    pub fn group_by_fingerprint(&mut self, rules: Vec<CssStructDef>) {
        for rule in rules {
            let structure = Structure::from(rule);
            let fingerprint = self.comparator.comparator.generate_fingerprint(&structure);
            self.fingerprint_cache
                .entry(fingerprint)
                .or_insert_with(Vec::new)
                .push(structure);
        }
    }
    
    /// 類似CSSルールを検出
    pub fn find_similar_rules(&mut self, threshold: f64) -> Vec<(Structure, Structure, f64)> {
        use crate::structure_comparator::should_compare_fingerprints;
        
        let mut results = Vec::new();
        let fingerprints: Vec<String> = self.fingerprint_cache.keys().cloned().collect();
        
        for i in 0..fingerprints.len() {
            for j in i..fingerprints.len() {
                let fp1 = &fingerprints[i];
                let fp2 = &fingerprints[j];
                
                if !should_compare_fingerprints(fp1, fp2) {
                    continue;
                }
                
                let structures1 = &self.fingerprint_cache[fp1];
                let structures2 = &self.fingerprint_cache[fp2];
                
                for s1 in structures1 {
                    let start_idx = if i == j {
                        structures2.iter().position(|s| std::ptr::eq(s, s1))
                            .map(|pos| pos + 1).unwrap_or(0)
                    } else {
                        0
                    };
                    
                    for s2 in &structures2[start_idx..] {
                        let result = self.comparator.comparator.compare(s1, s2);
                        
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
        
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_css_to_structure_conversion() {
        let css_rule = CssStructDef {
            selector: ".button".to_string(),
            declarations: vec![
                ("background-color".to_string(), "#007bff".to_string()),
                ("color".to_string(), "white".to_string()),
                ("padding".to_string(), "10px 20px".to_string()),
                ("border-radius".to_string(), "4px".to_string()),
            ],
            file_path: "styles.css".to_string(),
            start_line: 1,
            end_line: 6,
            media_query: None,
            parent_selectors: vec![],
        };
        
        let structure = Structure::from(css_rule);
        
        assert_eq!(structure.identifier.name, ".button");
        assert_eq!(structure.identifier.kind, StructureKind::CssClass);
        assert_eq!(structure.members.len(), 4);
        
        // Check value categorization
        let bg_color = structure.members.iter()
            .find(|m| m.name == "background-color")
            .unwrap();
        assert_eq!(bg_color.value_type, "color");
        
        let padding = structure.members.iter()
            .find(|m| m.name == "padding")
            .unwrap();
        assert_eq!(padding.value_type, "value"); // Complex value
    }
    
    #[test]
    fn test_css_comparison() {
        let mut comparator = CssStructureComparator::new();
        
        let rule1 = CssStructDef {
            selector: ".btn-primary".to_string(),
            declarations: vec![
                ("background".to_string(), "#007bff".to_string()),
                ("color".to_string(), "#fff".to_string()),
                ("padding".to_string(), "8px 16px".to_string()),
            ],
            file_path: "buttons.css".to_string(),
            start_line: 1,
            end_line: 5,
            media_query: None,
            parent_selectors: vec![],
        };
        
        let rule2 = CssStructDef {
            selector: ".button-primary".to_string(),
            declarations: vec![
                ("background-color".to_string(), "#007bff".to_string()),
                ("color".to_string(), "white".to_string()),
                ("padding".to_string(), "8px 16px".to_string()),
            ],
            file_path: "components.css".to_string(),
            start_line: 10,
            end_line: 14,
            media_query: None,
            parent_selectors: vec![],
        };
        
        let result = comparator.compare_rules(&rule1, &rule2);
        
        // Should have high similarity (similar selectors and properties)
        assert!(result.overall_similarity > 0.7);
        assert_eq!(result.member_matches.len(), 2); // color and padding match
    }
    
    #[test]
    fn test_selector_normalization() {
        assert_eq!(
            CssStructureComparator::normalize_selector(".class1  >  .class2"),
            ".class1>.class2"
        );
        
        assert_eq!(
            CssStructureComparator::normalize_selector("h1, h3, h2"),
            "h1, h2, h3"
        );
    }
    
    #[test]
    fn test_value_categorization() {
        assert_eq!(categorize_css_value("#ff0000"), "color");
        assert_eq!(categorize_css_value("rgb(255, 0, 0)"), "color");
        assert_eq!(categorize_css_value("10px"), "length");
        assert_eq!(categorize_css_value("2em"), "length");
        assert_eq!(categorize_css_value("100%"), "length");
        assert_eq!(categorize_css_value("0.5s"), "time");
        assert_eq!(categorize_css_value("300ms"), "time");
        assert_eq!(categorize_css_value("url(image.png)"), "url");
        assert_eq!(categorize_css_value("bold"), "keyword");
        assert_eq!(categorize_css_value("42"), "number");
    }
}