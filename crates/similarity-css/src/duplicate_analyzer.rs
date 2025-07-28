use crate::{CssRule, SerializableCssRule, calculate_rule_similarity, SelectorAnalysis};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Represents a potential duplicate CSS rule
#[derive(Debug, Clone)]
pub struct DuplicateRule {
    pub rule1: CssRule,
    pub rule2: CssRule,
    pub similarity: f64,
    pub duplicate_type: DuplicateType,
}

/// Serializable version of DuplicateRule for JSON output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableDuplicateRule {
    pub rule1: SerializableCssRule,
    pub rule2: SerializableCssRule,
    pub similarity: f64,
    pub duplicate_type: DuplicateType,
}

impl From<&DuplicateRule> for SerializableDuplicateRule {
    fn from(dup: &DuplicateRule) -> Self {
        SerializableDuplicateRule {
            rule1: (&dup.rule1).into(),
            rule2: (&dup.rule2).into(),
            similarity: dup.similarity,
            duplicate_type: dup.duplicate_type.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DuplicateType {
    /// Exact same selector and declarations
    ExactDuplicate,
    /// Same selector but different declarations
    SelectorConflict { declaration_similarity: f64 },
    /// Different selector but same declarations
    StyleDuplicate { selector1: String, selector2: String },
    /// Same BEM component with variations
    BemVariation { component: String },
    /// One selector overrides another due to specificity
    SpecificityOverride { winner: String, loser: String },
}

/// Analyzes CSS rules for various types of duplicates and conflicts
pub struct DuplicateAnalyzer {
    rules: Vec<CssRule>,
    threshold: f64,
}

impl DuplicateAnalyzer {
    pub fn new(rules: Vec<CssRule>, threshold: f64) -> Self {
        Self { rules, threshold }
    }
    
    /// Find all types of duplicates in the ruleset
    pub fn analyze(&self) -> DuplicateAnalysisResult {
        let mut exact_duplicates = Vec::new();
        let mut selector_conflicts = Vec::new();
        let mut style_duplicates = Vec::new();
        let mut bem_variations = Vec::new();
        let mut specificity_overrides = Vec::new();
        
        // Compare all pairs of rules
        for (i, rule1) in self.rules.iter().enumerate() {
            for (j, rule2) in self.rules.iter().enumerate() {
                if i >= j {
                    continue;
                }
                
                let similarity = calculate_rule_similarity(rule1, rule2);
                let sel_analysis1 = SelectorAnalysis::new(&rule1.selector);
                let sel_analysis2 = SelectorAnalysis::new(&rule2.selector);
                
                // Check for exact duplicates
                if rule1.selector == rule2.selector && similarity > 0.99 {
                    exact_duplicates.push(DuplicateRule {
                        rule1: rule1.clone(),
                        rule2: rule2.clone(),
                        similarity,
                        duplicate_type: DuplicateType::ExactDuplicate,
                    });
                }
                // Check for selector conflicts (same selector, different styles)
                else if rule1.selector == rule2.selector && similarity < 0.99 {
                    selector_conflicts.push(DuplicateRule {
                        rule1: rule1.clone(),
                        rule2: rule2.clone(),
                        similarity,
                        duplicate_type: DuplicateType::SelectorConflict {
                            declaration_similarity: similarity,
                        },
                    });
                }
                // Check for style duplicates (different selector, same styles)
                else if rule1.selector != rule2.selector && similarity > self.threshold {
                    style_duplicates.push(DuplicateRule {
                        rule1: rule1.clone(),
                        rule2: rule2.clone(),
                        similarity,
                        duplicate_type: DuplicateType::StyleDuplicate {
                            selector1: rule1.selector.clone(),
                            selector2: rule2.selector.clone(),
                        },
                    });
                    
                    // Check if they're BEM variations
                    if let (Some(bem1), Some(bem2)) = (&sel_analysis1.bem_parts, &sel_analysis2.bem_parts) {
                        if bem1.block == bem2.block {
                            bem_variations.push(DuplicateRule {
                                rule1: rule1.clone(),
                                rule2: rule2.clone(),
                                similarity,
                                duplicate_type: DuplicateType::BemVariation {
                                    component: bem1.block.clone(),
                                },
                            });
                        }
                    }
                }
                
                // Check for specificity overrides
                if sel_analysis1.overrides(&sel_analysis2) || sel_analysis2.overrides(&sel_analysis1) {
                    let (winner, loser) = if sel_analysis1.overrides(&sel_analysis2) {
                        (&rule1.selector, &rule2.selector)
                    } else {
                        (&rule2.selector, &rule1.selector)
                    };
                    
                    specificity_overrides.push(DuplicateRule {
                        rule1: rule1.clone(),
                        rule2: rule2.clone(),
                        similarity,
                        duplicate_type: DuplicateType::SpecificityOverride {
                            winner: winner.clone(),
                            loser: loser.clone(),
                        },
                    });
                }
            }
        }
        
        let summary = self.generate_summary(&exact_duplicates, &selector_conflicts, &style_duplicates);
        
        DuplicateAnalysisResult {
            exact_duplicates,
            selector_conflicts,
            style_duplicates,
            bem_variations,
            specificity_overrides,
            summary,
        }
    }
    
    /// Generate a summary of the analysis
    fn generate_summary(
        &self,
        exact_duplicates: &[DuplicateRule],
        selector_conflicts: &[DuplicateRule],
        style_duplicates: &[DuplicateRule],
    ) -> DuplicateSummary {
        let mut selector_usage = HashMap::new();
        for rule in &self.rules {
            *selector_usage.entry(rule.selector.clone()).or_insert(0) += 1;
        }
        
        let repeated_selectors: Vec<(String, usize)> = selector_usage
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .collect();
        
        DuplicateSummary {
            total_rules: self.rules.len(),
            exact_duplicate_count: exact_duplicates.len(),
            selector_conflict_count: selector_conflicts.len(),
            style_duplicate_count: style_duplicates.len(),
            repeated_selectors,
        }
    }
    
    /// Get recommendations for fixing duplicates
    pub fn get_recommendations(&self, result: &DuplicateAnalysisResult) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Exact duplicates
        if !result.exact_duplicates.is_empty() {
            recommendations.push(format!(
                "Found {} exact duplicate rules that can be safely removed",
                result.exact_duplicates.len()
            ));
            
            for dup in &result.exact_duplicates {
                recommendations.push(format!(
                    "  - Remove duplicate '{}' at line {}",
                    dup.rule2.selector,
                    dup.rule2.start_line
                ));
            }
        }
        
        // Selector conflicts
        if !result.selector_conflicts.is_empty() {
            recommendations.push(format!(
                "\nFound {} selector conflicts that need manual review",
                result.selector_conflicts.len()
            ));
            
            for conflict in &result.selector_conflicts {
                if let DuplicateType::SelectorConflict { declaration_similarity } = &conflict.duplicate_type {
                    recommendations.push(format!(
                        "  - Selector '{}' appears at lines {} and {} with {:.0}% similar styles",
                        conflict.rule1.selector,
                        conflict.rule1.start_line,
                        conflict.rule2.start_line,
                        declaration_similarity * 100.0
                    ));
                }
            }
        }
        
        // Style duplicates
        if !result.style_duplicates.is_empty() {
            recommendations.push(format!(
                "\nFound {} style duplicates that could be consolidated",
                result.style_duplicates.len()
            ));
            
            for dup in result.style_duplicates.iter().take(5) {
                if let DuplicateType::StyleDuplicate { selector1, selector2 } = &dup.duplicate_type {
                    recommendations.push(format!(
                        "  - '{}' and '{}' have {:.0}% similar styles",
                        selector1,
                        selector2,
                        dup.similarity * 100.0
                    ));
                }
            }
            
            if result.style_duplicates.len() > 5 {
                recommendations.push(format!(
                    "  ... and {} more style duplicates",
                    result.style_duplicates.len() - 5
                ));
            }
        }
        
        // BEM recommendations
        if !result.bem_variations.is_empty() {
            let mut bem_components: HashMap<String, usize> = HashMap::new();
            for var in &result.bem_variations {
                if let DuplicateType::BemVariation { component } = &var.duplicate_type {
                    *bem_components.entry(component.clone()).or_insert(0) += 1;
                }
            }
            
            recommendations.push("\nBEM component analysis:".to_string());
            for (component, count) in bem_components {
                recommendations.push(format!(
                    "  - Component '{component}' has {count} variations with similar styles"
                ));
            }
        }
        
        recommendations
    }
}

/// Result of duplicate analysis
#[derive(Debug)]
pub struct DuplicateAnalysisResult {
    pub exact_duplicates: Vec<DuplicateRule>,
    pub selector_conflicts: Vec<DuplicateRule>,
    pub style_duplicates: Vec<DuplicateRule>,
    pub bem_variations: Vec<DuplicateRule>,
    pub specificity_overrides: Vec<DuplicateRule>,
    pub summary: DuplicateSummary,
}

#[derive(Debug)]
pub struct DuplicateSummary {
    pub total_rules: usize,
    pub exact_duplicate_count: usize,
    pub selector_conflict_count: usize,
    pub style_duplicate_count: usize,
    pub repeated_selectors: Vec<(String, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_rule(selector: &str, declarations: Vec<(&str, &str)>, line: usize) -> CssRule {
        use similarity_core::tree::TreeNode;
        use std::rc::Rc;
        
        CssRule {
            selector: selector.to_string(),
            declarations: declarations.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            tree: Rc::new(TreeNode::new(selector.to_string(), String::new(), 0)),
            start_line: line,
            end_line: line + declarations.len(),
        }
    }
    
    #[test]
    fn test_exact_duplicate_detection() {
        let rules = vec![
            create_test_rule(".btn", vec![("color", "blue"), ("padding", "10px")], 1),
            create_test_rule(".btn", vec![("color", "blue"), ("padding", "10px")], 5),
            create_test_rule(".btn-primary", vec![("color", "white")], 10),
        ];
        
        let analyzer = DuplicateAnalyzer::new(rules, 0.8);
        let result = analyzer.analyze();
        
        assert_eq!(result.exact_duplicates.len(), 1);
        assert_eq!(result.exact_duplicates[0].duplicate_type, DuplicateType::ExactDuplicate);
    }
    
    #[test]
    fn test_style_duplicate_detection() {
        let rules = vec![
            create_test_rule(".card", vec![("padding", "20px"), ("background", "white")], 1),
            create_test_rule(".panel", vec![("padding", "20px"), ("background", "white")], 5),
            create_test_rule(".box", vec![("margin", "10px")], 10),
        ];
        
        let analyzer = DuplicateAnalyzer::new(rules, 0.5);
        let result = analyzer.analyze();
        
        assert_eq!(result.style_duplicates.len(), 1);
        match &result.style_duplicates[0].duplicate_type {
            DuplicateType::StyleDuplicate { selector1, selector2 } => {
                assert!(
                    (selector1 == ".card" && selector2 == ".panel") ||
                    (selector1 == ".panel" && selector2 == ".card")
                );
            }
            _ => panic!("Expected StyleDuplicate"),
        }
    }
    
    #[test]
    fn test_recommendations() {
        let rules = vec![
            create_test_rule(".btn", vec![("color", "blue")], 1),
            create_test_rule(".btn", vec![("color", "blue")], 5),
            create_test_rule(".btn", vec![("color", "red")], 10),
        ];
        
        let analyzer = DuplicateAnalyzer::new(rules, 0.8);
        let result = analyzer.analyze();
        let recommendations = analyzer.get_recommendations(&result);
        
        assert!(!recommendations.is_empty());
        assert!(recommendations[0].contains("exact duplicate"));
    }
}