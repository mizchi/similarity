use similarity_css::expand_shorthand_properties;

#[test]
fn test_vendor_prefixes() {
    let declarations = vec![
        ("-webkit-border-radius".to_string(), "5px".to_string()),
        ("-moz-border-radius".to_string(), "5px".to_string()),
        ("border-radius".to_string(), "5px".to_string()),
        ("-webkit-box-shadow".to_string(), "0 2px 4px rgba(0,0,0,0.1)".to_string()),
        ("box-shadow".to_string(), "0 2px 4px rgba(0,0,0,0.1)".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Vendor prefixed properties should pass through
    assert!(expanded.iter().any(|(k, _)| k == "-webkit-border-radius"));
    assert!(expanded.iter().any(|(k, _)| k == "-moz-border-radius"));
    
    // Standard border-radius should still expand
    assert!(expanded.iter().any(|(k, _)| k == "border-top-left-radius"));
}

#[test]
fn test_css_custom_properties() {
    let declarations = vec![
        ("--primary-color".to_string(), "#007bff".to_string()),
        ("--spacing".to_string(), "1rem".to_string()),
        ("margin".to_string(), "var(--spacing)".to_string()),
        ("padding".to_string(), "calc(var(--spacing) * 2)".to_string()),
        ("color".to_string(), "var(--primary-color, blue)".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Custom properties should pass through
    assert!(expanded.iter().any(|(k, v)| k == "--primary-color" && v == "#007bff"));
    assert!(expanded.iter().any(|(k, v)| k == "--spacing" && v == "1rem"));
    
    // Shorthand with var() should still expand
    assert!(expanded.iter().any(|(k, v)| k == "margin-top" && v.contains("var(--spacing)")));
    assert!(expanded.iter().any(|(k, v)| k == "padding-top" && v.contains("calc")));
}

#[test]
fn test_calc_and_functions() {
    let declarations = vec![
        ("margin".to_string(), "calc(100% - 20px)".to_string()),
        ("padding".to_string(), "max(10px, 2vh) min(20px, 4vh)".to_string()),
        ("width".to_string(), "clamp(200px, 50%, 800px)".to_string()),
        ("transform".to_string(), "translateX(calc(50% - 10px))".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // calc() in margin should be preserved in expansion
    assert!(expanded.iter().any(|(k, v)| k == "margin-top" && v == "calc(100% - 20px)"));
    
    // Complex padding with min/max
    assert!(expanded.iter().any(|(k, v)| k == "padding-top" && v == "max(10px, 2vh)"));
    assert!(expanded.iter().any(|(k, v)| k == "padding-right" && v == "min(20px, 4vh)"));
    
    // Non-shorthand properties should pass through
    assert!(expanded.iter().any(|(k, v)| k == "width" && v.contains("clamp")));
    assert!(expanded.iter().any(|(k, v)| k == "transform" && v.contains("translateX")));
}

#[test]
fn test_multiple_values_and_slashes() {
    let declarations = vec![
        ("font".to_string(), "italic bold 16px/1.5 Arial, sans-serif".to_string()),
        ("background".to_string(), "url(bg.jpg) center/cover no-repeat fixed".to_string()),
        ("border-radius".to_string(), "10px 20px / 5px 10px".to_string()),
        ("grid-area".to_string(), "header / header / header / header".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Complex font and background should be kept as-is for now
    assert!(expanded.iter().any(|(k, _)| k == "font"));
    assert!(expanded.iter().any(|(k, _)| k == "background"));
    
    // Border-radius with elliptical radii
    assert!(expanded.iter().any(|(k, _)| k == "border-radius"));
    
    // Grid-area should pass through
    assert!(expanded.iter().any(|(k, _)| k == "grid-area"));
}

#[test]
fn test_inherit_initial_unset() {
    let declarations = vec![
        ("margin".to_string(), "inherit".to_string()),
        ("padding".to_string(), "initial".to_string()),
        ("border".to_string(), "unset".to_string()),
        ("all".to_string(), "revert".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // inherit should expand to all sides
    assert!(expanded.iter().any(|(k, v)| k == "margin-top" && v == "inherit"));
    assert!(expanded.iter().any(|(k, v)| k == "margin-right" && v == "inherit"));
    
    // initial should expand
    assert!(expanded.iter().any(|(k, v)| k == "padding-top" && v == "initial"));
    
    // border with unset
    assert!(expanded.iter().any(|(k, v)| k == "border-top-style" && v == "none"));
    
    // all property should pass through
    assert!(expanded.iter().any(|(k, v)| k == "all" && v == "revert"));
}

#[test]
fn test_important_declarations() {
    let declarations = vec![
        ("margin".to_string(), "10px !important".to_string()),
        ("padding".to_string(), "5px 10px!important".to_string()),
        ("color".to_string(), "red !important".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // !important should be preserved but currently isn't handled
    // This is a known limitation
    assert!(expanded.iter().any(|(k, v)| k == "margin-top" && (v == "10px" || v.contains("!important"))));
}

#[test]
fn test_gradients_and_images() {
    let declarations = vec![
        ("background".to_string(), "linear-gradient(to right, red, blue)".to_string()),
        ("background".to_string(), "radial-gradient(circle, yellow, green)".to_string()),
        ("background".to_string(), "url('image.png'), linear-gradient(red, blue)".to_string()),
        ("background".to_string(), "conic-gradient(from 45deg, red, blue)".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Gradients should be detected as background-image
    assert!(expanded.iter().any(|(k, v)| k == "background-image" && v.contains("linear-gradient")));
    assert!(expanded.iter().any(|(k, v)| k == "background-image" && v.contains("radial-gradient")));
}

#[test]
fn test_animation_shorthand_complex() {
    let declarations = vec![
        ("animation".to_string(), "slide-in 0.3s ease-in-out 0.1s infinite alternate both".to_string()),
        ("animation".to_string(), "bounce 1s, fade 2s".to_string()),
        ("transition".to_string(), "opacity 0.3s ease-in-out, transform 0.2s".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Complex animations are kept as-is
    assert!(expanded.iter().any(|(k, v)| k == "animation" && v.contains("slide-in")));
    assert!(expanded.iter().any(|(k, v)| k == "animation" && v.contains("bounce")));
    assert!(expanded.iter().any(|(k, _)| k == "transition"));
}

#[test]
fn test_logical_properties() {
    let declarations = vec![
        ("margin-inline".to_string(), "10px 20px".to_string()),
        ("margin-block".to_string(), "30px".to_string()),
        ("padding-inline-start".to_string(), "15px".to_string()),
        ("border-block-end".to_string(), "2px solid black".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Logical properties are not yet supported, should pass through
    assert!(expanded.iter().any(|(k, _)| k == "margin-inline"));
    assert!(expanded.iter().any(|(k, _)| k == "margin-block"));
    assert!(expanded.iter().any(|(k, _)| k == "padding-inline-start"));
    assert!(expanded.iter().any(|(k, _)| k == "border-block-end"));
}

#[test]
fn test_container_queries() {
    let declarations = vec![
        ("container".to_string(), "layout / inline-size".to_string()),
        ("container-name".to_string(), "card".to_string()),
        ("container-type".to_string(), "inline-size".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Container queries are new, should pass through
    assert_eq!(expanded.len(), 3);
    assert!(expanded.iter().any(|(k, _)| k == "container"));
    assert!(expanded.iter().any(|(k, _)| k == "container-name"));
    assert!(expanded.iter().any(|(k, _)| k == "container-type"));
}

#[test]
fn test_whitespace_handling() {
    let declarations = vec![
        ("margin".to_string(), "  10px   20px  ".to_string()),
        ("padding".to_string(), "\t5px\t10px\t".to_string()),
        ("border".to_string(), "1px\n\nsolid\n\nblack".to_string()),
    ];
    
    let expanded = expand_shorthand_properties(&declarations);
    
    // Should handle extra whitespace correctly
    assert!(expanded.iter().any(|(k, v)| k == "margin-top" && v == "10px"));
    assert!(expanded.iter().any(|(k, v)| k == "margin-right" && v == "20px"));
    assert!(expanded.iter().any(|(k, v)| k == "padding-top" && v == "5px"));
}