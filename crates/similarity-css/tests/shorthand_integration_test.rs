use similarity_css::expand_shorthand_properties;

#[test]
fn test_complex_shorthand_expansion() {
    // Test all major shorthand properties
    let declarations = vec![
        ("margin".to_string(), "10px 20px 30px 40px".to_string()),
        ("padding".to_string(), "5px 10px".to_string()),
        ("border".to_string(), "2px dashed red".to_string()),
        ("border-radius".to_string(), "5px 10px".to_string()),
        ("flex".to_string(), "2 1 auto".to_string()),
        ("gap".to_string(), "20px 30px".to_string()),
        ("place-items".to_string(), "center start".to_string()),
        ("overflow".to_string(), "scroll".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Verify expansion happened
    assert!(expanded.len() > declarations.len());
    
    // Check margin expansion
    assert!(expanded.iter().any(|(k, v)| k == "margin-top" && v == "10px"));
    assert!(expanded.iter().any(|(k, v)| k == "margin-right" && v == "20px"));
    assert!(expanded.iter().any(|(k, v)| k == "margin-bottom" && v == "30px"));
    assert!(expanded.iter().any(|(k, v)| k == "margin-left" && v == "40px"));
    
    // Check padding expansion
    assert!(expanded.iter().any(|(k, v)| k == "padding-top" && v == "5px"));
    assert!(expanded.iter().any(|(k, v)| k == "padding-right" && v == "10px"));
    
    // Check flex expansion
    assert!(expanded.iter().any(|(k, v)| k == "flex-grow" && v == "2"));
    assert!(expanded.iter().any(|(k, v)| k == "flex-shrink" && v == "1"));
    assert!(expanded.iter().any(|(k, v)| k == "flex-basis" && v == "auto"));
    
    // Check gap expansion
    assert!(expanded.iter().any(|(k, v)| k == "row-gap" && v == "20px"));
    assert!(expanded.iter().any(|(k, v)| k == "column-gap" && v == "30px"));
    
    // Check place-items expansion
    assert!(expanded.iter().any(|(k, v)| k == "align-items" && v == "center"));
    assert!(expanded.iter().any(|(k, v)| k == "justify-items" && v == "start"));
    
    // Check overflow expansion
    assert!(expanded.iter().any(|(k, v)| k == "overflow-x" && v == "scroll"));
    assert!(expanded.iter().any(|(k, v)| k == "overflow-y" && v == "scroll"));
}

#[test]
fn test_edge_cases() {
    // Test edge cases and special values
    let declarations = vec![
        ("margin".to_string(), "auto".to_string()),
        ("flex".to_string(), "none".to_string()),
        ("border".to_string(), "none".to_string()),
        ("overflow".to_string(), "visible hidden".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // margin: auto should expand to all sides
    assert!(expanded.iter().any(|(k, v)| k == "margin-top" && v == "auto"));
    assert!(expanded.iter().any(|(k, v)| k == "margin-right" && v == "auto"));
    
    // flex: none should expand correctly
    assert!(expanded.iter().any(|(k, v)| k == "flex-grow" && v == "0"));
    assert!(expanded.iter().any(|(k, v)| k == "flex-shrink" && v == "0"));
    assert!(expanded.iter().any(|(k, v)| k == "flex-basis" && v == "auto"));
    
    // overflow with different x/y values
    assert!(expanded.iter().any(|(k, v)| k == "overflow-x" && v == "visible"));
    assert!(expanded.iter().any(|(k, v)| k == "overflow-y" && v == "hidden"));
}

#[test]
fn test_invalid_shorthand_handling() {
    // Test that invalid shorthands are preserved
    let declarations = vec![
        ("margin".to_string(), "10px 20px 30px 40px 50px".to_string()), // Too many values
        ("flex".to_string(), "invalid value here".to_string()),
        ("unknown-property".to_string(), "some value".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Invalid margin should be kept as-is
    assert!(expanded.iter().any(|(k, v)| k == "margin" && v.contains("50px")));
    
    // Invalid flex should be kept as-is
    assert!(expanded.iter().any(|(k, v)| k == "flex" && v.contains("invalid")));
    
    // Unknown property should pass through
    assert!(expanded.iter().any(|(k, _v)| k == "unknown-property"));
}

#[test]
fn test_mixed_shorthand_and_longhand() {
    // Test mixing shorthand and longhand properties
    let declarations = vec![
        ("margin".to_string(), "10px".to_string()),
        ("margin-top".to_string(), "20px".to_string()), // This should override
        ("padding-left".to_string(), "5px".to_string()),
        ("padding".to_string(), "15px".to_string()), // This expands but doesn't override existing
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Should have both the expanded margin and the specific margin-top
    let margin_tops: Vec<_> = expanded.iter()
        .filter(|(k, _)| k == "margin-top")
        .collect();
    assert_eq!(margin_tops.len(), 2); // One from expansion, one explicit
    
    // Should have both padding-left values
    let padding_lefts: Vec<_> = expanded.iter()
        .filter(|(k, _)| k == "padding-left")
        .collect();
    assert_eq!(padding_lefts.len(), 2);
}

#[test]
fn test_color_normalization() {
    let declarations = vec![
        ("border".to_string(), "1px solid black".to_string()),
        ("background".to_string(), "#000000".to_string()),
        ("color".to_string(), "rgb(255, 0, 0)".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Border should expand with color preserved
    assert!(expanded.iter().any(|(k, v)| k == "border-top-color" && v == "black"));
    
    // Background color detection
    assert!(expanded.iter().any(|(k, v)| k == "background-color" && v == "#000000"));
    
    // Non-shorthand color should pass through
    assert!(expanded.iter().any(|(k, v)| k == "color" && v.contains("rgb")));
}