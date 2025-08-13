/// CSS Specificity calculation and analysis
///
/// Specificity is calculated as (a, b, c) where:
/// - a = number of ID selectors
/// - b = number of class selectors, attributes, and pseudo-classes
/// - c = number of type selectors and pseudo-elements

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity {
    pub ids: u32,
    pub classes: u32,
    pub types: u32,
}

impl Specificity {
    pub fn new(ids: u32, classes: u32, types: u32) -> Self {
        Self { ids, classes, types }
    }

    /// Calculate specificity value for comparison
    /// Using a large base to ensure proper ordering
    pub fn value(&self) -> u32 {
        self.ids * 10000 + self.classes * 100 + self.types
    }

    /// Check if this specificity is higher than another
    pub fn is_higher_than(&self, other: &Specificity) -> bool {
        self > other
    }

    /// Check if specificities are equal
    pub fn is_equal_to(&self, other: &Specificity) -> bool {
        self == other
    }
}

impl std::fmt::Display for Specificity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.ids, self.classes, self.types)
    }
}

/// Calculate specificity for a CSS selector
pub fn calculate_specificity(selector: &str) -> Specificity {
    let mut ids = 0;
    let mut classes = 0;
    let mut types = 0;

    // Remove pseudo-element content and normalize
    let normalized = normalize_selector(selector);

    // Split by combinators while preserving them
    let parts = split_selector_parts(&normalized);

    for part in parts {
        if part.is_empty() || is_combinator(&part) {
            continue;
        }

        // Count IDs
        ids += part.matches('#').count() as u32;

        // Count classes
        classes += part.matches('.').count() as u32;

        // Count attribute selectors
        classes += count_attributes(&part);

        // Count pseudo-classes (excluding pseudo-elements)
        classes += count_pseudo_classes(&part);

        // Count pseudo-elements
        types += count_pseudo_elements(&part);

        // Count type selectors
        if !part.starts_with('.')
            && !part.starts_with('#')
            && !part.starts_with('[')
            && !part.starts_with(':')
        {
            // Check if it's a type selector (not universal selector)
            if !part.trim().is_empty() && part.trim() != "*" {
                types += 1;
            }
        }
    }

    Specificity::new(ids, classes, types)
}

/// Normalize selector for parsing
fn normalize_selector(selector: &str) -> String {
    selector
        .trim()
        // Remove content inside :not(), :is(), :where() etc
        .replace(":not(", ":not")
        .replace(":is(", ":is")
        .replace(":where(", ":where")
        .replace(":has(", ":has")
        // Remove closing parentheses
        .replace(")", "")
        // Normalize whitespace around combinators
        .replace(">", " > ")
        .replace("+", " + ")
        .replace("~", " ~ ")
        // Collapse multiple spaces
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Split selector into parts
fn split_selector_parts(selector: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_brackets = false;

    for ch in selector.chars() {
        match ch {
            '[' => {
                in_brackets = true;
                current.push(ch);
            }
            ']' => {
                in_brackets = false;
                current.push(ch);
            }
            ' ' if !in_brackets => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

/// Check if a part is a combinator
fn is_combinator(part: &str) -> bool {
    matches!(part.trim(), ">" | "+" | "~" | "||")
}

/// Count attribute selectors
fn count_attributes(part: &str) -> u32 {
    part.matches('[').count() as u32
}

/// Count pseudo-classes (single colon, not pseudo-elements)
fn count_pseudo_classes(part: &str) -> u32 {
    let mut count = 0;
    let mut chars = part.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == ':' {
            // Check if it's not a pseudo-element (::)
            if chars.peek() != Some(&':') {
                count += 1;
            } else {
                // Skip the second colon
                chars.next();
            }
        }
    }

    count
}

/// Count pseudo-elements (double colon)
fn count_pseudo_elements(part: &str) -> u32 {
    part.matches("::").count() as u32
}

/// Analyze and compare selectors for duplication
#[derive(Debug)]
pub struct SelectorAnalysis {
    pub selector: String,
    pub specificity: Specificity,
    pub is_bem: bool,
    pub bem_parts: Option<BemParts>,
}

#[derive(Debug, Clone)]
pub struct BemParts {
    pub block: String,
    pub element: Option<String>,
    pub modifier: Option<String>,
}

impl SelectorAnalysis {
    pub fn new(selector: &str) -> Self {
        let specificity = calculate_specificity(selector);
        let bem_parts = parse_bem_selector(selector);
        let is_bem = bem_parts.is_some();

        Self { selector: selector.to_string(), specificity, is_bem, bem_parts }
    }

    /// Check if this selector is effectively identical to another
    pub fn is_duplicate_of(&self, other: &SelectorAnalysis) -> bool {
        // Exact match
        if self.selector == other.selector {
            return true;
        }

        // Same specificity and BEM parts
        if self.specificity == other.specificity {
            if let (Some(bem1), Some(bem2)) = (&self.bem_parts, &other.bem_parts) {
                return bem1.block == bem2.block
                    && bem1.element == bem2.element
                    && bem1.modifier == bem2.modifier;
            }
        }

        false
    }

    /// Check if this selector might override another
    pub fn overrides(&self, other: &SelectorAnalysis) -> bool {
        self.specificity > other.specificity
    }
}

/// Parse BEM notation from a selector
fn parse_bem_selector(selector: &str) -> Option<BemParts> {
    // Simple BEM parser for class selectors
    if !selector.starts_with('.') {
        return None;
    }

    let class_name = selector.trim_start_matches('.').split_whitespace().next()?;

    // Check for element separator
    if let Some(elem_pos) = class_name.find("__") {
        let block = class_name[..elem_pos].to_string();
        let rest = &class_name[elem_pos + 2..];

        // Check for modifier
        if let Some(mod_pos) = rest.find("--") {
            let element = rest[..mod_pos].to_string();
            let modifier = rest[mod_pos + 2..].to_string();
            return Some(BemParts { block, element: Some(element), modifier: Some(modifier) });
        } else {
            return Some(BemParts { block, element: Some(rest.to_string()), modifier: None });
        }
    }

    // Check for modifier on block
    if let Some(mod_pos) = class_name.find("--") {
        let block = class_name[..mod_pos].to_string();
        let modifier = class_name[mod_pos + 2..].to_string();
        return Some(BemParts { block, element: None, modifier: Some(modifier) });
    }

    // Just a block
    Some(BemParts { block: class_name.to_string(), element: None, modifier: None })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_specificity() {
        assert_eq!(calculate_specificity("div"), Specificity::new(0, 0, 1));
        assert_eq!(calculate_specificity(".class"), Specificity::new(0, 1, 0));
        assert_eq!(calculate_specificity("#id"), Specificity::new(1, 0, 0));
    }

    #[test]
    fn test_complex_specificity() {
        assert_eq!(calculate_specificity("div.class#id"), Specificity::new(1, 1, 1));

        assert_eq!(calculate_specificity(".class1.class2"), Specificity::new(0, 2, 0));

        assert_eq!(calculate_specificity("div > p + span"), Specificity::new(0, 0, 3));
    }

    #[test]
    fn test_pseudo_classes_and_elements() {
        assert_eq!(calculate_specificity("a:hover"), Specificity::new(0, 1, 1));

        assert_eq!(calculate_specificity("p::first-line"), Specificity::new(0, 0, 2));

        assert_eq!(calculate_specificity("input[type='text']"), Specificity::new(0, 1, 1));
    }

    #[test]
    fn test_bem_parsing() {
        let bem = parse_bem_selector(".block").unwrap();
        assert_eq!(bem.block, "block");
        assert!(bem.element.is_none());
        assert!(bem.modifier.is_none());

        let bem = parse_bem_selector(".block__element").unwrap();
        assert_eq!(bem.block, "block");
        assert_eq!(bem.element.unwrap(), "element");
        assert!(bem.modifier.is_none());

        let bem = parse_bem_selector(".block--modifier").unwrap();
        assert_eq!(bem.block, "block");
        assert!(bem.element.is_none());
        assert_eq!(bem.modifier.unwrap(), "modifier");

        let bem = parse_bem_selector(".block__element--modifier").unwrap();
        assert_eq!(bem.block, "block");
        assert_eq!(bem.element.unwrap(), "element");
        assert_eq!(bem.modifier.unwrap(), "modifier");
    }

    #[test]
    fn test_specificity_comparison() {
        let spec1 = Specificity::new(1, 0, 0);
        let spec2 = Specificity::new(0, 10, 10);
        assert!(spec1.is_higher_than(&spec2));

        let spec3 = Specificity::new(0, 1, 0);
        let spec4 = Specificity::new(0, 0, 10);
        assert!(spec3.is_higher_than(&spec4));
    }
}
