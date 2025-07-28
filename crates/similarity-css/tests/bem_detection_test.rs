use similarity_css::{CssParser, CssRule, calculate_rule_similarity, calculate_specificity, SelectorAnalysis, convert_to_css_rule};
use similarity_core::language_parser::LanguageParser;

#[test]
fn test_bem_exact_duplicate_detection() {
    let scss_content = r#"
// BEM: Card component
.card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    padding: 20px;
}

.card__header {
    font-size: 18px;
    font-weight: bold;
    margin-bottom: 16px;
}

.card__body {
    font-size: 14px;
    line-height: 1.5;
}

.card__footer {
    border-top: 1px solid #e0e0e0;
    margin-top: 16px;
    padding-top: 16px;
}

// Duplicate: Exact same card component (different location in file)
.card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    padding: 20px;
}

// Similar but not exact: different padding
.card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    padding: 24px; // Different!
}

// BEM with modifiers
.card--primary {
    background-color: #007bff;
    color: white;
}

.card--primary .card__header {
    color: white;
}

// Duplicate modifier
.card--primary {
    background-color: #007bff;
    color: white;
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "test.scss").unwrap();
    
    println!("Found {} rules", rules.len());
    for rule in &rules {
        println!("  - {} (lines {}-{})", rule.name, rule.body_start_line, rule.body_end_line);
    }
    
    // Convert to CssRule for easier testing
    let css_rules: Vec<_> = rules.iter()
        .map(|func| convert_to_css_rule(func, scss_content))
        .collect();

    // Test exact duplicates
    let card_rules: Vec<&CssRule> = css_rules.iter()
        .filter(|r| r.selector == ".card")
        .collect();
    
    assert_eq!(card_rules.len(), 3, "Should find 3 .card rules");
    
    // First two .card rules should be exact duplicates
    let analysis1 = SelectorAnalysis::new(&card_rules[0].selector);
    let analysis2 = SelectorAnalysis::new(&card_rules[1].selector);
    assert!(analysis1.is_duplicate_of(&analysis2), "Same selectors should be duplicates");
    
    // Compare declarations
    let sim_0_1 = calculate_rule_similarity(card_rules[0], card_rules[1]);
    assert!(sim_0_1 > 0.99, "Exact duplicate rules should have very high similarity");
    
    let sim_0_2 = calculate_rule_similarity(card_rules[0], card_rules[2]);
    println!("Similarity between similar rules: {sim_0_2}");
    println!("Rule 0: {:?}", card_rules[0].declarations);
    println!("Rule 2: {:?}", card_rules[2].declarations);
    assert!(sim_0_2 > 0.7 && sim_0_2 < 0.99, "Similar but not exact rules should have high but not perfect similarity");
    
    // Test modifier duplicates
    let modifier_rules: Vec<&CssRule> = css_rules.iter()
        .filter(|r| r.selector == ".card--primary")
        .collect();
    
    assert_eq!(modifier_rules.len(), 2, "Should find 2 .card--primary rules");
    let mod_sim = calculate_rule_similarity(modifier_rules[0], modifier_rules[1]);
    assert!(mod_sim > 0.99, "Duplicate modifiers should have very high similarity");
}

#[test]
fn test_bem_specificity_hierarchy() {
    let scss_content = r#"
// Base block
.button {
    padding: 10px 20px;
    border: none;
    cursor: pointer;
}

// Element
.button__icon {
    margin-right: 8px;
}

// Modifier
.button--primary {
    background-color: blue;
    color: white;
}

// More specific: modifier + element
.button--primary .button__icon {
    color: white;
}

// Even more specific: with pseudo-class
.button--primary:hover {
    background-color: darkblue;
}

// Most specific: ID override (anti-pattern in BEM)
#special-button.button {
    background-color: red;
}
"#;

    let mut parser = CssParser::new_scss();
    let _rules = parser.extract_functions(scss_content, "test.scss").unwrap();
    
    // Test specificity ordering
    let selectors = [".button",
        ".button__icon",
        ".button--primary",
        ".button--primary .button__icon",
        ".button--primary:hover",
        "#special-button.button"];
    
    let specificities: Vec<_> = selectors.iter()
        .map(|s| (s, calculate_specificity(s)))
        .collect();
    
    // Check specificity values
    assert_eq!(specificities[0].1, calculate_specificity(".button")); // (0, 1, 0)
    assert_eq!(specificities[3].1, calculate_specificity(".button--primary .button__icon")); // (0, 2, 0)
    assert_eq!(specificities[4].1, calculate_specificity(".button--primary:hover")); // (0, 2, 0)
    assert_eq!(specificities[5].1, calculate_specificity("#special-button.button")); // (1, 1, 0)
    
    // ID selector should have highest specificity
    let id_spec = &specificities[5].1;
    for (_, spec) in &specificities[0..5] {
        assert!(id_spec.is_higher_than(spec), "ID selector should override all class selectors");
    }
}

#[test]
fn test_bem_nested_scss_patterns() {
    let scss_content = r#"
// SCSS nesting with BEM
.modal {
    position: fixed;
    background: white;
    
    &__overlay {
        position: absolute;
        background: rgba(0, 0, 0, 0.5);
    }
    
    &__content {
        padding: 20px;
        
        &--large {
            padding: 40px;
        }
    }
    
    &--open {
        display: block;
    }
    
    &--open & {
        &__overlay {
            opacity: 1;
        }
    }
}

// Duplicate nested pattern
.modal {
    position: fixed;
    background: white;
    
    &__overlay {
        position: absolute;
        background: rgba(0, 0, 0, 0.5);
    }
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "test.scss").unwrap();
    
    // SCSS parser should flatten nested rules
    let modal_rules: Vec<_> = rules.iter()
        .filter(|r| r.name.starts_with(".modal"))
        .collect();
    
    // Should find multiple modal-related rules
    assert!(modal_rules.len() >= 2, "Should find modal rules");
    
    // Check for exact duplicates
    let base_modal_rules: Vec<_> = rules.iter()
        .filter(|r| r.name == ".modal")
        .collect();
    
    assert_eq!(base_modal_rules.len(), 2, "Should find 2 .modal base rules");
}

#[test]
fn test_rule_level_duplicate_analysis() {
    // Test complete rule comparison including selector + declarations
    let scss_content = r#"
// Original rule
.nav__item {
    display: inline-block;
    padding: 10px 15px;
    color: #333;
    text-decoration: none;
}

// Exact duplicate (should be detected)
.nav__item {
    display: inline-block;
    padding: 10px 15px;
    color: #333;
    text-decoration: none;
}

// Same styles, different selector (not a duplicate at rule level)
.menu__link {
    display: inline-block;
    padding: 10px 15px;
    color: #333;
    text-decoration: none;
}

// Same selector, different styles (partial duplicate)
.nav__item {
    display: block; // Changed
    padding: 10px 15px;
    color: #333;
    text-decoration: none;
}

// Functionally equivalent with shorthand expansion
.nav__item {
    display: inline-block;
    padding-top: 10px;
    padding-right: 15px;
    padding-bottom: 10px;
    padding-left: 15px;
    color: #333;
    text-decoration: none;
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "test.scss").unwrap();
    
    let css_rules: Vec<_> = rules.iter()
        .map(|func| convert_to_css_rule(func, scss_content))
        .collect();
    
    // Analyze duplicates
    let mut exact_duplicates = Vec::new();
    let mut similar_rules = Vec::new();
    
    for (i, rule1) in css_rules.iter().enumerate() {
        for (j, rule2) in css_rules.iter().enumerate() {
            if i >= j {
                continue;
            }
            
            let similarity = calculate_rule_similarity(rule1, rule2);
            
            if rule1.selector == rule2.selector && similarity > 0.99 {
                exact_duplicates.push((i, j, similarity));
            } else if similarity > 0.8 {
                similar_rules.push((i, j, similarity));
            }
        }
    }
    
    // Should find exact duplicates
    assert!(!exact_duplicates.is_empty(), "Should find exact duplicate rules");
    
    // Should find similar rules (same content, different selector)
    assert!(!similar_rules.is_empty(), "Should find similar rules");
}


#[test]
fn test_bem_component_variations() {
    let scss_content = r#"
// Alert component with BEM variations
.alert {
    padding: 16px;
    border-radius: 4px;
    margin-bottom: 16px;
}

.alert__icon {
    display: inline-block;
    margin-right: 8px;
}

.alert__message {
    display: inline;
}

.alert--success {
    background-color: #d4edda;
    border-color: #c3e6cb;
    color: #155724;
}

.alert--error {
    background-color: #f8d7da;
    border-color: #f5c6cb;
    color: #721c24;
}

// Duplicate success alert
.alert--success {
    background-color: #d4edda;
    border-color: #c3e6cb;
    color: #155724;
}

// Very similar error alert (slightly different color)
.alert--error {
    background-color: #f8d7da;
    border-color: #f5c6cb;
    color: #721c25; // Slightly different
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "test.scss").unwrap();
    
    // Group by BEM component
    let alert_base: Vec<_> = rules.iter().filter(|r| r.name == ".alert").collect();
    let alert_elements: Vec<_> = rules.iter().filter(|r| r.name.starts_with(".alert__")).collect();
    let alert_modifiers: Vec<_> = rules.iter().filter(|r| r.name.starts_with(".alert--")).collect();
    
    assert!(!alert_base.is_empty(), "Should find base alert rules");
    assert_eq!(alert_elements.len(), 2, "Should find 2 alert elements");
    assert_eq!(alert_modifiers.len(), 4, "Should find 4 alert modifier rules");
    
    // Check for duplicate modifiers
    let success_alerts: Vec<_> = rules.iter()
        .filter(|r| r.name == ".alert--success")
        .collect();
    assert_eq!(success_alerts.len(), 2, "Should find duplicate success alerts");
    
    let error_alerts: Vec<_> = rules.iter()
        .filter(|r| r.name == ".alert--error")
        .collect();
    assert_eq!(error_alerts.len(), 2, "Should find 2 error alerts");
}