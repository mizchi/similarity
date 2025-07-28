pub mod parser;
pub mod css_parser;
pub mod css_comparator;
pub mod shorthand_expander;
pub mod specificity;
pub mod duplicate_analyzer;
pub mod css_rule_converter;
pub mod scss_flattener;
pub mod scss_simple_flattener;

pub use parser::CssParser;
pub use css_comparator::{CssRule, SerializableCssRule, CssSimilarityResult, compare_css_rules, calculate_rule_similarity};
pub use shorthand_expander::expand_shorthand_properties;
pub use specificity::{Specificity, calculate_specificity, SelectorAnalysis};
pub use duplicate_analyzer::{DuplicateAnalyzer, DuplicateAnalysisResult, DuplicateType, DuplicateRule, SerializableDuplicateRule};
pub use css_rule_converter::{convert_to_css_rule, parse_css_to_rules};
pub use scss_flattener::{flatten_scss_rules, FlatRule};