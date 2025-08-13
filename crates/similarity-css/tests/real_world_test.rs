use similarity_core::language_parser::LanguageParser;
use similarity_core::tree::TreeNode;
use similarity_css::{CssParser, CssRule};
use std::rc::Rc;

#[allow(dead_code)]
/// Helper to create CSS rules from parsed functions
fn create_rules_from_functions(
    functions: Vec<similarity_core::language_parser::GenericFunctionDef>,
) -> Vec<CssRule> {
    functions
        .into_iter()
        .map(|func| {
            // Parse the selector to extract declarations
            let declarations = extract_declarations_from_name(&func.name);

            // Create a simple tree for testing
            let tree = TreeNode::new("rule".to_string(), func.name.clone(), 0);

            CssRule {
                selector: func.name,
                declarations,
                tree: Rc::new(tree),
                start_line: func.body_start_line as usize,
                end_line: func.body_end_line as usize,
            }
        })
        .collect()
}

#[allow(dead_code)]
/// Simple declaration extractor for testing
fn extract_declarations_from_name(_selector: &str) -> Vec<(String, String)> {
    // For testing purposes, we'll return empty declarations
    // In real usage, these would be extracted from the parsed CSS
    vec![]
}

#[test]
fn test_bootstrap_like_components() {
    let css1 = r#"
        .btn {
            display: inline-block;
            padding: 0.375rem 0.75rem;
            margin: 0;
            font-size: 1rem;
            font-weight: 400;
            line-height: 1.5;
            text-align: center;
            text-decoration: none;
            vertical-align: middle;
            cursor: pointer;
            border: 1px solid transparent;
            border-radius: 0.25rem;
            transition: all 0.15s ease-in-out;
        }
        
        .btn-primary {
            color: #fff;
            background-color: #007bff;
            border-color: #007bff;
        }
    "#;

    let css2 = r#"
        .button {
            display: inline-block;
            padding: 6px 12px; /* Same as 0.375rem 0.75rem with 16px base */
            margin: 0 0 0 0;
            font-size: 16px; /* Same as 1rem */
            font-weight: 400;
            line-height: 1.5;
            text-align: center;
            text-decoration: none;
            vertical-align: middle;
            cursor: pointer;
            border-width: 1px;
            border-style: solid;
            border-color: transparent;
            border-radius: 4px; /* Same as 0.25rem */
            transition-property: all;
            transition-duration: 0.15s;
            transition-timing-function: ease-in-out;
        }
        
        .button--primary {
            color: white;
            background-color: #007bff;
            border-color: #007bff;
        }
    "#;

    let mut parser = CssParser::new();
    let funcs1 = parser.extract_functions(css1, "bootstrap.css").unwrap();
    let funcs2 = parser.extract_functions(css2, "custom.css").unwrap();

    assert!(!funcs1.is_empty());
    assert!(!funcs2.is_empty());

    // The button classes should be detected
    assert!(funcs1.iter().any(|f| f.name.contains(".btn")));
    assert!(funcs2.iter().any(|f| f.name.contains(".button")));
}

#[test]
fn test_flexbox_grid_patterns() {
    let css1 = r#"
        .container {
            display: flex;
            flex-direction: row;
            flex-wrap: wrap;
            justify-content: space-between;
            align-items: center;
            gap: 1rem;
        }
        
        .flex-item {
            flex: 1 1 auto;
            margin: 0.5rem;
        }
    "#;

    let css2 = r#"
        .wrapper {
            display: flex;
            flex-flow: row wrap; /* Shorthand for direction + wrap */
            justify-content: space-between;
            align-items: center;
            row-gap: 1rem;
            column-gap: 1rem;
        }
        
        .flex-child {
            flex-grow: 1;
            flex-shrink: 1;
            flex-basis: auto;
            margin: 8px; /* Same as 0.5rem with 16px base */
        }
    "#;

    let mut parser = CssParser::new();
    let funcs1 = parser.extract_functions(css1, "flex1.css").unwrap();
    let funcs2 = parser.extract_functions(css2, "flex2.css").unwrap();

    // Both should have container and item rules
    assert_eq!(funcs1.len(), 2);
    assert_eq!(funcs2.len(), 2);
}

#[test]
fn test_grid_layout_patterns() {
    let css1 = r#"
        .grid-container {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            grid-template-rows: auto auto;
            gap: 20px 10px;
            place-items: center;
        }
        
        .grid-item {
            padding: 20px;
            background: #f0f0f0;
            border: 1px solid #ddd;
        }
    "#;

    let css2 = r#"
        .grid-wrapper {
            display: grid;
            grid-template: auto auto / repeat(3, 1fr); /* rows / columns */
            row-gap: 20px;
            column-gap: 10px;
            align-items: center;
            justify-items: center;
        }
        
        .grid-cell {
            padding: 20px 20px 20px 20px;
            background-color: #f0f0f0;
            border-width: 1px;
            border-style: solid;
            border-color: #ddd;
        }
    "#;

    let mut parser = CssParser::new();
    let funcs1 = parser.extract_functions(css1, "grid1.css").unwrap();
    let funcs2 = parser.extract_functions(css2, "grid2.css").unwrap();

    assert_eq!(funcs1.len(), 2);
    assert_eq!(funcs2.len(), 2);
}

#[test]
fn test_responsive_utilities() {
    let css = r#"
        .hidden { display: none; }
        .d-none { display: none; }
        
        .visible { visibility: visible; }
        .invisible { visibility: hidden; }
        
        .overflow-hidden { overflow: hidden; }
        .overflow-x-hidden { overflow-x: hidden; overflow-y: visible; }
        .overflow-y-hidden { overflow-x: visible; overflow-y: hidden; }
        
        @media (min-width: 768px) {
            .md\:hidden { display: none; }
            .md\:block { display: block; }
        }
        
        @media (max-width: 767px) {
            .mobile-only { display: block; }
        }
    "#;

    let mut parser = CssParser::new();
    let functions = parser.extract_functions(css, "utilities.css").unwrap();

    // Should detect regular rules and media queries
    assert!(functions.len() > 7); // All utility classes plus media queries

    // Check for media queries
    assert!(functions.iter().any(|f| f.name.contains("@media")));
}

#[test]
fn test_animation_and_transition_patterns() {
    let css1 = r#"
        .fade-in {
            animation: fadeIn 0.3s ease-in;
            transition: opacity 0.3s ease-in;
        }
        
        .slide-up {
            animation-name: slideUp;
            animation-duration: 0.5s;
            animation-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
            animation-fill-mode: both;
        }
    "#;

    let css2 = r#"
        .fade-enter {
            animation: fadeIn 300ms ease-in;
            transition-property: opacity;
            transition-duration: 300ms;
            transition-timing-function: ease-in;
        }
        
        .slide-in {
            animation: slideUp 500ms cubic-bezier(0.4, 0, 0.2, 1) both;
        }
    "#;

    let mut parser = CssParser::new();
    let funcs1 = parser.extract_functions(css1, "anim1.css").unwrap();
    let funcs2 = parser.extract_functions(css2, "anim2.css").unwrap();

    assert_eq!(funcs1.len(), 2);
    assert_eq!(funcs2.len(), 2);
}

#[test]
fn test_scss_nesting_and_mixins() {
    let scss = r#"
        .card {
            background: white;
            padding: 1rem;
            border-radius: 8px;
            
            &__header {
                font-size: 1.25rem;
                font-weight: bold;
                margin-bottom: 0.5rem;
            }
            
            &__body {
                color: #333;
                line-height: 1.6;
            }
            
            &:hover {
                box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
            }
        }
        
        @mixin button-style($bg-color: #007bff) {
            display: inline-block;
            padding: 0.5rem 1rem;
            background-color: $bg-color;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            
            &:hover {
                opacity: 0.9;
            }
        }
        
        .btn-primary {
            @include button-style();
        }
        
        .btn-success {
            @include button-style(#28a745);
        }
    "#;

    let mut parser = CssParser::new_scss();
    let functions = parser.extract_functions(scss, "component.scss").unwrap();

    // Should detect nested rules and mixins
    assert!(functions.len() >= 3); // At least .card, mixin, and button classes
    assert!(functions.iter().any(|f| f.name.contains("@mixin")));
}

#[test]
fn test_complex_selectors() {
    let css = r#"
        /* Descendant and child combinators */
        .nav ul li { list-style: none; }
        .nav > ul > li { display: inline-block; }
        
        /* Attribute selectors */
        input[type="text"] { border: 1px solid #ccc; }
        input[type="email"] { border: 1px solid #ccc; }
        a[href^="https://"] { color: green; }
        
        /* Pseudo-classes and pseudo-elements */
        a:hover { text-decoration: underline; }
        a:visited { color: purple; }
        p::first-line { font-weight: bold; }
        li::marker { color: #007bff; }
        
        /* Complex combinators */
        .form-group:not(:last-child) { margin-bottom: 1rem; }
        .menu li + li { border-left: 1px solid #ddd; }
        h2 ~ p { margin-top: 0.5rem; }
        
        /* Multiple selectors */
        h1, h2, h3, h4, h5, h6 { font-family: "Helvetica", sans-serif; }
        .btn, .button, [role="button"] { cursor: pointer; }
    "#;

    let mut parser = CssParser::new();
    let functions = parser.extract_functions(css, "selectors.css").unwrap();

    // Should handle all complex selectors
    assert!(functions.len() >= 10);

    // Check various selector types are captured
    assert!(functions.iter().any(|f| f.name.contains("["))); // Attribute
    assert!(functions.iter().any(|f| f.name.contains(":"))); // Pseudo-class
    assert!(functions.iter().any(|f| f.name.contains("::"))); // Pseudo-element
    assert!(functions.iter().any(|f| f.name.contains("+"))); // Adjacent sibling
    assert!(functions.iter().any(|f| f.name.contains("~"))); // General sibling
}
