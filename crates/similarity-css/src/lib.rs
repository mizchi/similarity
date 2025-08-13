pub mod css_comparator;
pub mod css_parser;
pub mod css_rule_converter;
pub mod duplicate_analyzer;
pub mod parser;
pub mod scss_flattener;
pub mod scss_simple_flattener;
pub mod shorthand_expander;
pub mod specificity;

pub use css_comparator::{
    calculate_rule_similarity, compare_css_rules, CssRule, CssSimilarityResult, SerializableCssRule,
};
pub use css_rule_converter::{convert_to_css_rule, parse_css_to_rules};
pub use duplicate_analyzer::{
    DuplicateAnalysisResult, DuplicateAnalyzer, DuplicateRule, DuplicateType,
    SerializableDuplicateRule,
};
pub use parser::CssParser;
pub use scss_flattener::{flatten_scss_rules, FlatRule};
pub use shorthand_expander::expand_shorthand_properties;
pub use specificity::{calculate_specificity, SelectorAnalysis, Specificity};
