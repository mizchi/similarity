use similarity_css::{
    CssParser, DuplicateAnalyzer, DuplicateType, calculate_specificity, convert_to_css_rule,
};
use similarity_core::language_parser::LanguageParser;

#[test]
fn test_full_duplicate_analysis_workflow() {
    let scss_content = r#"
// Navigation component
.nav {
    display: flex;
    justify-content: space-between;
    padding: 1rem;
    background-color: #f8f9fa;
}

.nav__list {
    display: flex;
    list-style: none;
    margin: 0;
    padding: 0;
}

.nav__item {
    margin-right: 1rem;
}

.nav__link {
    color: #333;
    text-decoration: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    transition: background-color 0.2s;
}

.nav__link:hover {
    background-color: #e9ecef;
}

.nav__link--active {
    background-color: #007bff;
    color: white;
}

// Duplicate navigation (maybe from merge conflict)
.nav {
    display: flex;
    justify-content: space-between;
    padding: 1rem;
    background-color: #f8f9fa;
}

// Similar menu component
.menu {
    display: flex;
    justify-content: space-between;
    padding: 1rem;
    background-color: #f8f9fa;
}

.menu__link {
    color: #333;
    text-decoration: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    transition: background-color 0.2s;
}

// Card component
.card {
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    padding: 1.5rem;
}

.card--primary {
    background: #007bff;
    color: white;
}

.card--primary .card__title {
    color: white;
}

// Another card variant (duplicate styles)
.panel {
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    padding: 1.5rem;
}
"#;

    // Parse CSS
    let mut parser = CssParser::new_scss();
    let functions = parser.extract_functions(scss_content, "test.scss").unwrap();
    
    // Convert to CssRule format with proper tree nodes
    let css_rules: Vec<_> = functions.iter()
        .map(|func| convert_to_css_rule(func, scss_content))
        .collect();
    
    // Analyze duplicates
    let analyzer = DuplicateAnalyzer::new(css_rules, 0.8);
    let result = analyzer.analyze();
    
    // Test exact duplicates
    assert!(!result.exact_duplicates.is_empty(), "Should find exact duplicate .nav rules");
    let nav_duplicate = result.exact_duplicates.iter()
        .find(|d| d.rule1.selector == ".nav")
        .expect("Should find .nav duplicate");
    assert_eq!(nav_duplicate.duplicate_type, DuplicateType::ExactDuplicate);
    
    // Test style duplicates
    assert!(!result.style_duplicates.is_empty(), "Should find style duplicates");
    
    // Check for nav/menu similarity
    let nav_menu_similar = result.style_duplicates.iter()
        .find(|d| {
            (d.rule1.selector == ".nav" && d.rule2.selector == ".menu") ||
            (d.rule1.selector == ".menu" && d.rule2.selector == ".nav")
        });
    assert!(nav_menu_similar.is_some(), "Should detect nav/menu similarity");
    
    // Check for card/panel similarity
    let card_panel_similar = result.style_duplicates.iter()
        .find(|d| {
            (d.rule1.selector == ".card" && d.rule2.selector == ".panel") ||
            (d.rule1.selector == ".panel" && d.rule2.selector == ".card")
        });
    assert!(card_panel_similar.is_some(), "Should detect card/panel similarity");
    
    // Test BEM variations
    let nav_bem_variations = result.bem_variations.iter()
        .filter(|v| {
            if let DuplicateType::BemVariation { component } = &v.duplicate_type {
                component == "nav"
            } else {
                false
            }
        })
        .count();
    assert!(nav_bem_variations > 0, "Should detect nav BEM variations");
    
    // Get recommendations
    let recommendations = analyzer.get_recommendations(&result);
    assert!(!recommendations.is_empty(), "Should provide recommendations");
    
    // Verify recommendations mention duplicates
    let recs_text = recommendations.join("\n");
    assert!(recs_text.contains("exact duplicate"), "Should mention exact duplicates");
    assert!(recs_text.contains("style duplicates"), "Should mention style duplicates");
}

#[test]
fn test_specificity_based_override_detection() {
    let scss_content = r#"
// Base button
.btn {
    padding: 10px 20px;
    background: #ccc;
    color: black;
}

// More specific
.container .btn {
    background: #999;
}

// Even more specific
.page .container .btn {
    background: #666;
}

// ID overrides everything
#special-btn {
    background: #f00;
}

// Complex specificity
.btn.btn-primary.active {
    background: #007bff;
}

// Pseudo-class
.btn:hover {
    background: #aaa;
}

// Attribute selector
.btn[disabled] {
    background: #eee;
    cursor: not-allowed;
}
"#;

    let mut parser = CssParser::new_scss();
    let functions = parser.extract_functions(scss_content, "test.scss").unwrap();
    
    // Test specificity calculations
    let selectors_and_expected = vec![
        (".btn", (0, 1, 0)),
        (".container .btn", (0, 2, 0)),
        (".page .container .btn", (0, 3, 0)),
        ("#special-btn", (1, 0, 0)),
        (".btn.btn-primary.active", (0, 3, 0)),
        (".btn:hover", (0, 2, 0)),
        (".btn[disabled]", (0, 2, 0)),
    ];
    
    for (selector, expected) in selectors_and_expected {
        let spec = calculate_specificity(selector);
        assert_eq!(
            (spec.ids, spec.classes, spec.types),
            expected,
            "Specificity for '{}' should be {:?}",
            selector,
            expected
        );
    }
    
    // Create rules for override analysis
    let css_rules: Vec<_> = functions.iter()
        .map(|func| convert_to_css_rule(func, scss_content))
        .collect();
    
    let analyzer = DuplicateAnalyzer::new(css_rules, 0.8);
    let result = analyzer.analyze();
    
    // Should detect specificity overrides
    assert!(!result.specificity_overrides.is_empty(), "Should detect specificity overrides");
    
    // ID selector should override class selectors
    let id_override = result.specificity_overrides.iter()
        .find(|o| {
            if let DuplicateType::SpecificityOverride { winner, .. } = &o.duplicate_type {
                winner == "#special-btn"
            } else {
                false
            }
        });
    assert!(id_override.is_some(), "ID selector should be detected as override winner");
}

#[test]
fn test_scss_nested_bem_analysis() {
    let scss_content = r#"
// SCSS with nested BEM
.header {
    background: #333;
    color: white;
    
    &__logo {
        font-size: 24px;
        font-weight: bold;
    }
    
    &__nav {
        display: flex;
        gap: 20px;
        
        &-item {
            padding: 10px;
            
            &--active {
                background: #555;
            }
        }
    }
    
    &--sticky {
        position: fixed;
        top: 0;
        width: 100%;
    }
}

// Duplicate nested structure
.header {
    background: #333;
    color: white;
    
    &__logo {
        font-size: 24px;
        font-weight: bold;
    }
}
"#;

    let mut parser = CssParser::new_scss();
    let functions = parser.extract_functions(scss_content, "test.scss").unwrap();
    
    // Should parse nested SCSS and find duplicates
    let header_rules = functions.iter()
        .filter(|f| f.name.starts_with(".header"))
        .count();
    
    assert!(header_rules >= 2, "Should find multiple header-related rules");
    
    // Check that nested selectors are properly generated
    let has_logo = functions.iter().any(|f| f.name.contains("header__logo"));
    let has_nav = functions.iter().any(|f| f.name.contains("header__nav"));
    let has_sticky = functions.iter().any(|f| f.name.contains("header--sticky"));
    
    assert!(has_logo || has_nav || has_sticky, 
            "Should parse at least some nested BEM selectors");
}