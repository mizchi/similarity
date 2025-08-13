use similarity_core::language_parser::LanguageParser;
use similarity_css::{convert_to_css_rule, CssParser, DuplicateAnalyzer};
use std::time::Instant;

#[test]
#[ignore] // Run with: cargo test --package similarity-css performance -- --ignored
fn test_large_scss_file_performance() {
    // Generate a large SCSS file
    let mut scss_content = String::new();

    // Add base styles
    scss_content.push_str(
        r#"
// Base variables
$primary-color: #3498db;
$secondary-color: #2ecc71;
$base-padding: 1rem;
$base-margin: 1rem;

"#,
    );

    // Generate many utility classes
    for i in 0..100 {
        scss_content.push_str(&format!(
            r#"
.component-{} {{
    padding: $base-padding;
    margin: $base-margin;
    background-color: lighten($primary-color, {}%);
    
    &__header {{
        font-size: {}px;
        font-weight: bold;
        margin-bottom: 0.5rem;
    }}
    
    &__body {{
        font-size: {}px;
        line-height: 1.5;
        
        p {{
            margin-bottom: 1rem;
            
            &:last-child {{
                margin-bottom: 0;
            }}
        }}
    }}
    
    &__footer {{
        display: flex;
        justify-content: space-between;
        padding-top: 1rem;
        border-top: 1px solid #e0e0e0;
        
        button {{
            padding: 0.5rem 1rem;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            
            &.primary {{
                background-color: $primary-color;
                color: white;
            }}
            
            &.secondary {{
                background-color: $secondary-color;
                color: white;
            }}
        }}
    }}
    
    &--large {{
        padding: calc($base-padding * 2);
        
        .component-{i}__header {{
            font-size: {}px;
        }}
    }}
    
    &--small {{
        padding: calc($base-padding / 2);
        
        .component-{i}__header {{
            font-size: {}px;
        }}
    }}
}}
"#,
            i,
            i % 50,
            18 + (i % 8),
            14 + (i % 4),
            24 + (i % 6),
            12 + (i % 4)
        ));
    }

    // Parse and flatten
    let start = Instant::now();
    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(&scss_content, "large.scss").unwrap();
    let parse_time = start.elapsed();

    println!("Parsing {} bytes took {:?}", scss_content.len(), parse_time);
    println!("Generated {} rules", rules.len());

    // Convert to CSS rules
    let start = Instant::now();
    let css_rules: Vec<_> =
        rules.iter().map(|func| convert_to_css_rule(func, &scss_content)).collect();
    let convert_time = start.elapsed();

    println!("Converting to CSS rules took {convert_time:?}");

    // Analyze duplicates
    let start = Instant::now();
    let analyzer = DuplicateAnalyzer::new(css_rules, 0.85);
    let result = analyzer.analyze();
    let analyze_time = start.elapsed();

    println!("Duplicate analysis took {analyze_time:?}");
    println!("Found {} exact duplicates", result.exact_duplicates.len());
    println!("Found {} style duplicates", result.style_duplicates.len());
    println!("Found {} BEM variations", result.bem_variations.len());

    // Performance assertions
    assert!(parse_time.as_millis() < 1000, "Parsing should be fast");
    assert!(analyze_time.as_secs() < 5, "Analysis should complete within 5 seconds");
}

#[test]
fn test_real_world_bootstrap_patterns() {
    let scss_content = r#"
// Bootstrap-like grid system
$grid-columns: 12;
$grid-gutter-width: 30px;
$container-max-widths: (
  sm: 540px,
  md: 720px,
  lg: 960px,
  xl: 1140px,
  xxl: 1320px
);

.container {
    width: 100%;
    padding-right: calc($grid-gutter-width / 2);
    padding-left: calc($grid-gutter-width / 2);
    margin-right: auto;
    margin-left: auto;
    
    @media (min-width: 576px) {
        max-width: 540px;
    }
    
    @media (min-width: 768px) {
        max-width: 720px;
    }
    
    @media (min-width: 992px) {
        max-width: 960px;
    }
    
    @media (min-width: 1200px) {
        max-width: 1140px;
    }
}

.row {
    display: flex;
    flex-wrap: wrap;
    margin-right: calc($grid-gutter-width / -2);
    margin-left: calc($grid-gutter-width / -2);
    
    > * {
        flex-shrink: 0;
        width: 100%;
        max-width: 100%;
        padding-right: calc($grid-gutter-width / 2);
        padding-left: calc($grid-gutter-width / 2);
    }
}

// Column classes
@for $i from 1 through $grid-columns {
    .col-#{$i} {
        flex: 0 0 auto;
        width: percentage($i / $grid-columns);
    }
    
    .col-sm-#{$i} {
        @media (min-width: 576px) {
            flex: 0 0 auto;
            width: percentage($i / $grid-columns);
        }
    }
    
    .col-md-#{$i} {
        @media (min-width: 768px) {
            flex: 0 0 auto;
            width: percentage($i / $grid-columns);
        }
    }
}

// Bootstrap-like buttons
.btn {
    display: inline-block;
    font-weight: 400;
    text-align: center;
    white-space: nowrap;
    vertical-align: middle;
    user-select: none;
    border: 1px solid transparent;
    padding: 0.375rem 0.75rem;
    font-size: 1rem;
    line-height: 1.5;
    border-radius: 0.25rem;
    transition: color 0.15s ease-in-out, background-color 0.15s ease-in-out,
                border-color 0.15s ease-in-out, box-shadow 0.15s ease-in-out;
    
    &:hover {
        text-decoration: none;
    }
    
    &:focus {
        outline: 0;
        box-shadow: 0 0 0 0.25rem rgba(13, 110, 253, 0.25);
    }
    
    &:disabled {
        opacity: 0.65;
        pointer-events: none;
    }
}

.btn-primary {
    @extend .btn;
    color: #fff;
    background-color: #0d6efd;
    border-color: #0d6efd;
    
    &:hover {
        color: #fff;
        background-color: #0b5ed7;
        border-color: #0a58ca;
    }
    
    &:focus {
        color: #fff;
        background-color: #0b5ed7;
        border-color: #0a58ca;
        box-shadow: 0 0 0 0.25rem rgba(49, 132, 253, 0.5);
    }
}

.btn-secondary {
    @extend .btn;
    color: #fff;
    background-color: #6c757d;
    border-color: #6c757d;
    
    &:hover {
        color: #fff;
        background-color: #5c636a;
        border-color: #565e64;
    }
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "bootstrap.scss").unwrap();

    let css_rules: Vec<_> =
        rules.iter().map(|func| convert_to_css_rule(func, scss_content)).collect();

    // Analyze patterns
    let analyzer = DuplicateAnalyzer::new(css_rules.clone(), 0.8);
    let result = analyzer.analyze();

    println!("Bootstrap pattern analysis:");
    println!("- Total rules: {}", css_rules.len());
    println!(
        "- Grid column rules: {}",
        css_rules.iter().filter(|r| r.selector.contains("col-")).count()
    );
    println!("- Button rules: {}", css_rules.iter().filter(|r| r.selector.contains("btn")).count());
    println!("- Media query rules: {}", rules.iter().filter(|r| r.name.contains("@media")).count());

    // Get recommendations
    let recommendations = analyzer.get_recommendations(&result);
    println!("\nRecommendations:");
    for rec in recommendations {
        println!("{rec}");
    }

    // Should find button pattern duplications
    assert!(!result.style_duplicates.is_empty(), "Should find style duplicates in button patterns");
}

#[test]
fn test_tailwind_like_utilities() {
    let scss_content = r#"
// Tailwind-like utility classes
$spacing-scale: (
  0: 0,
  1: 0.25rem,
  2: 0.5rem,
  3: 0.75rem,
  4: 1rem,
  5: 1.25rem,
  6: 1.5rem,
  8: 2rem,
  10: 2.5rem,
  12: 3rem,
  16: 4rem,
  20: 5rem,
  24: 6rem,
  32: 8rem,
  40: 10rem,
  48: 12rem,
  56: 14rem,
  64: 16rem
);

// Margin utilities
@each $key, $value in $spacing-scale {
    .m-#{$key} { margin: $value; }
    .mt-#{$key} { margin-top: $value; }
    .mr-#{$key} { margin-right: $value; }
    .mb-#{$key} { margin-bottom: $value; }
    .ml-#{$key} { margin-left: $value; }
    .mx-#{$key} { 
        margin-left: $value;
        margin-right: $value;
    }
    .my-#{$key} {
        margin-top: $value;
        margin-bottom: $value;
    }
}

// Padding utilities
@each $key, $value in $spacing-scale {
    .p-#{$key} { padding: $value; }
    .pt-#{$key} { padding-top: $value; }
    .pr-#{$key} { padding-right: $value; }
    .pb-#{$key} { padding-bottom: $value; }
    .pl-#{$key} { padding-left: $value; }
    .px-#{$key} {
        padding-left: $value;
        padding-right: $value;
    }
    .py-#{$key} {
        padding-top: $value;
        padding-bottom: $value;
    }
}

// Display utilities
.block { display: block; }
.inline-block { display: inline-block; }
.inline { display: inline; }
.flex { display: flex; }
.inline-flex { display: inline-flex; }
.grid { display: grid; }
.inline-grid { display: inline-grid; }
.hidden { display: none; }

// Flexbox utilities
.flex-row { flex-direction: row; }
.flex-row-reverse { flex-direction: row-reverse; }
.flex-col { flex-direction: column; }
.flex-col-reverse { flex-direction: column-reverse; }

.flex-wrap { flex-wrap: wrap; }
.flex-wrap-reverse { flex-wrap: wrap-reverse; }
.flex-nowrap { flex-wrap: nowrap; }

.justify-start { justify-content: flex-start; }
.justify-end { justify-content: flex-end; }
.justify-center { justify-content: center; }
.justify-between { justify-content: space-between; }
.justify-around { justify-content: space-around; }
.justify-evenly { justify-content: space-evenly; }

.items-start { align-items: flex-start; }
.items-end { align-items: flex-end; }
.items-center { align-items: center; }
.items-baseline { align-items: baseline; }
.items-stretch { align-items: stretch; }

// Text utilities
.text-xs { font-size: 0.75rem; line-height: 1rem; }
.text-sm { font-size: 0.875rem; line-height: 1.25rem; }
.text-base { font-size: 1rem; line-height: 1.5rem; }
.text-lg { font-size: 1.125rem; line-height: 1.75rem; }
.text-xl { font-size: 1.25rem; line-height: 1.75rem; }
.text-2xl { font-size: 1.5rem; line-height: 2rem; }
.text-3xl { font-size: 1.875rem; line-height: 2.25rem; }
.text-4xl { font-size: 2.25rem; line-height: 2.5rem; }

// Responsive variants
@media (min-width: 640px) {
    .sm\:block { display: block; }
    .sm\:flex { display: flex; }
    .sm\:hidden { display: none; }
    .sm\:text-base { font-size: 1rem; line-height: 1.5rem; }
    .sm\:text-lg { font-size: 1.125rem; line-height: 1.75rem; }
}

@media (min-width: 768px) {
    .md\:block { display: block; }
    .md\:flex { display: flex; }
    .md\:hidden { display: none; }
    .md\:text-lg { font-size: 1.125rem; line-height: 1.75rem; }
    .md\:text-xl { font-size: 1.25rem; line-height: 1.75rem; }
}
"#;

    let mut parser = CssParser::new_scss();
    let start = Instant::now();
    let rules = parser.extract_functions(scss_content, "tailwind.scss").unwrap();
    let parse_time = start.elapsed();

    println!("Parsing Tailwind-like utilities took {parse_time:?}");
    println!("Generated {} utility classes", rules.len());

    // Count different types
    let margin_count = rules.iter().filter(|r| r.name.starts_with(".m")).count();
    let padding_count = rules.iter().filter(|r| r.name.starts_with(".p")).count();
    let flex_count = rules
        .iter()
        .filter(|r| {
            r.name.contains("flex") || r.name.contains("justify") || r.name.contains("items")
        })
        .count();
    let text_count = rules.iter().filter(|r| r.name.contains("text")).count();
    let responsive_count =
        rules.iter().filter(|r| r.name.contains("sm\\:") || r.name.contains("md\\:")).count();

    println!("Margin utilities: {margin_count}");
    println!("Padding utilities: {padding_count}");
    println!("Flexbox utilities: {flex_count}");
    println!("Text utilities: {text_count}");
    println!("Responsive utilities: {responsive_count}");

    // Utilities should parse quickly
    assert!(parse_time.as_millis() < 500, "Utility parsing should be fast");
    assert!(rules.len() > 100, "Should generate many utility classes");
}
