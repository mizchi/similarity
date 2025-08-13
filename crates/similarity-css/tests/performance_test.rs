use similarity_css::expand_shorthand_properties;
use std::time::Instant;

#[test]
#[ignore] // Run with: cargo test --package similarity-css performance_test -- --ignored
fn test_expansion_performance() {
    let mut declarations = Vec::new();

    // Generate a large number of declarations
    for i in 0..1000 {
        declarations.push((format!("margin-{i}"), "10px 20px 30px 40px".to_string()));
        declarations.push((format!("padding-{i}"), "5px 10px".to_string()));
        declarations.push((format!("border-{i}"), "1px solid black".to_string()));
        declarations.push((format!("flex-{i}"), "1 1 auto".to_string()));
        declarations.push((format!("gap-{i}"), "10px 20px".to_string()));
    }

    let start = Instant::now();
    let expanded = expand_shorthand_properties(&declarations);
    let duration = start.elapsed();

    println!(
        "Expanded {} declarations to {} in {:?}",
        declarations.len(),
        expanded.len(),
        duration
    );

    // Basic sanity checks
    assert!(expanded.len() > declarations.len());
    assert!(duration.as_millis() < 100); // Should be fast
}

#[test]
fn test_memory_efficiency() {
    // Test that expansion doesn't create excessive allocations
    let declarations = vec![
        ("margin".to_string(), "10px".to_string()),
        ("padding".to_string(), "20px 30px".to_string()),
        ("border".to_string(), "1px solid red".to_string()),
    ];

    let expanded = expand_shorthand_properties(&declarations);

    // Count total string bytes
    let original_bytes: usize = declarations.iter().map(|(k, v)| k.len() + v.len()).sum();

    let expanded_bytes: usize = expanded.iter().map(|(k, v)| k.len() + v.len()).sum();

    // Expansion should not create excessive memory usage
    // (property names are longer but values are reused)
    assert!(expanded_bytes < original_bytes * 10);
}

#[test]
fn test_repeated_properties() {
    // Test handling of repeated properties (CSS cascade)
    let declarations = vec![
        ("margin".to_string(), "10px".to_string()),
        ("margin".to_string(), "20px".to_string()),
        ("margin-top".to_string(), "30px".to_string()),
        ("margin".to_string(), "40px".to_string()),
    ];

    let expanded = expand_shorthand_properties(&declarations);

    // Should preserve all declarations in order
    let margin_tops: Vec<&String> =
        expanded.iter().filter(|(k, _)| k == "margin-top").map(|(_, v)| v).collect();

    // We should have expansions from all margin shorthands plus the explicit one
    assert_eq!(margin_tops.len(), 4); // 3 from expansions + 1 explicit
}

#[test]
fn test_deeply_nested_values() {
    // Test complex nested function values
    let declarations = vec![
        ("margin".to_string(), "calc(max(10px, min(2vw, 20px)))".to_string()),
        ("padding".to_string(), "clamp(5px, 2%, 20px) clamp(10px, 4%, 40px)".to_string()),
        ("transform".to_string(), "translateX(calc(var(--x) * 1px))".to_string()),
    ];

    let expanded = expand_shorthand_properties(&declarations);

    // Complex calc expressions should be preserved
    assert!(expanded.iter().any(|(k, v)| k == "margin-top"
        && v.contains("calc")
        && v.contains("max")
        && v.contains("min")));

    // Clamp values should be distributed correctly
    assert!(expanded.iter().any(|(k, v)| k == "padding-top" && v.contains("clamp(5px")));
}

#[test]
fn test_stress_all_shorthands() {
    // Test all supported shorthands at once
    let declarations = vec![
        ("margin".to_string(), "1px 2px 3px 4px".to_string()),
        ("padding".to_string(), "5px 6px 7px 8px".to_string()),
        ("border".to_string(), "9px dotted blue".to_string()),
        ("border-radius".to_string(), "10px 11px 12px 13px".to_string()),
        ("background".to_string(), "#ff0000".to_string()),
        ("font".to_string(), "italic 16px Arial".to_string()),
        ("flex".to_string(), "2 3 100px".to_string()),
        ("grid".to_string(), "auto / 1fr 2fr".to_string()),
        ("grid-template".to_string(), "100px 200px / 50% 50%".to_string()),
        ("gap".to_string(), "14px 15px".to_string()),
        ("place-items".to_string(), "start end".to_string()),
        ("place-content".to_string(), "center space-around".to_string()),
        ("place-self".to_string(), "auto center".to_string()),
        ("overflow".to_string(), "scroll hidden".to_string()),
        ("transition".to_string(), "all 0.3s".to_string()),
        ("animation".to_string(), "none".to_string()),
    ];

    let expanded = expand_shorthand_properties(&declarations);

    // Should have many more properties after expansion
    assert!(expanded.len() >= 50);

    // Spot check some expansions
    assert!(expanded.iter().any(|(k, v)| k == "margin-top" && v == "1px"));
    assert!(expanded.iter().any(|(k, v)| k == "padding-right" && v == "6px"));
    assert!(expanded.iter().any(|(k, v)| k == "flex-grow" && v == "2"));
    assert!(expanded.iter().any(|(k, v)| k == "row-gap" && v == "14px"));
}

#[test]
fn test_empty_and_invalid_input() {
    // Empty declarations
    let empty_decls: Vec<(String, String)> = vec![];
    let expanded_empty = expand_shorthand_properties(&empty_decls);
    assert_eq!(expanded_empty.len(), 0);

    // Invalid but safe inputs
    let invalid_decls = vec![
        ("".to_string(), "value".to_string()),
        ("property".to_string(), "".to_string()),
        ("margin".to_string(), "".to_string()),
        ("padding".to_string(), "   ".to_string()),
    ];

    let expanded_invalid = expand_shorthand_properties(&invalid_decls);

    // Should handle gracefully
    assert!(expanded_invalid.iter().any(|(k, _)| k.is_empty()));
    assert!(expanded_invalid.iter().any(|(_, v)| v.is_empty()));
}
