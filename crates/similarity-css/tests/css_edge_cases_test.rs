use similarity_core::language_parser::LanguageParser;
use similarity_css::{calculate_specificity, convert_to_css_rule, CssParser, DuplicateAnalyzer};

#[test]
fn test_pseudo_elements_and_classes() {
    let scss_content = r#"
// Pseudo-elements
.content {
    &::before {
        content: "";
        display: block;
        width: 100%;
        height: 2px;
        background: linear-gradient(to right, #3498db, #2ecc71);
    }
    
    &::after {
        content: "";
        display: block;
        width: 100%;
        height: 2px;
        background: linear-gradient(to right, #2ecc71, #3498db);
    }
    
    &::first-line {
        font-weight: bold;
        color: #2c3e50;
    }
    
    &::selection {
        background: #3498db;
        color: white;
    }
}

// Complex pseudo-classes
.link {
    color: #3498db;
    text-decoration: none;
    
    &:hover {
        text-decoration: underline;
    }
    
    &:visited {
        color: #9b59b6;
    }
    
    &:active {
        color: #2980b9;
    }
    
    &:focus {
        outline: 2px solid #3498db;
        outline-offset: 2px;
    }
    
    &:focus-visible {
        outline: 3px solid #3498db;
    }
    
    &:focus:not(:focus-visible) {
        outline: none;
    }
}

// Structural pseudo-classes
.list-item {
    padding: 10px;
    border-bottom: 1px solid #ecf0f1;
    
    &:first-child {
        border-top: 1px solid #ecf0f1;
    }
    
    &:last-child {
        border-bottom: none;
    }
    
    &:nth-child(even) {
        background: #f8f9fa;
    }
    
    &:nth-child(3n+1) {
        font-weight: bold;
    }
    
    &:only-child {
        border: 2px solid #3498db;
    }
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "test.scss").unwrap();

    // Check pseudo-elements (double colon)
    let pseudo_elements: Vec<_> =
        rules.iter().filter(|r| r.name.contains("::")).map(|r| &r.name).collect();

    println!("Pseudo-elements found:");
    for elem in &pseudo_elements {
        println!("  - {elem}");
    }

    assert_eq!(pseudo_elements.len(), 4, "Should find 4 pseudo-elements");

    // Check pseudo-classes
    let pseudo_classes: Vec<_> = rules
        .iter()
        .filter(|r| r.name.contains(":") && !r.name.contains("::"))
        .map(|r| &r.name)
        .collect();

    assert!(!pseudo_classes.is_empty(), "Should find pseudo-classes");

    // Test specificity of complex selectors
    let complex_selector = ".link:focus:not(:focus-visible)";
    let spec = calculate_specificity(complex_selector);
    println!("Specificity of '{complex_selector}': {spec}");
    assert_eq!(spec.classes, 3, "Should count :focus, :not, and :focus-visible as classes");
}

#[test]
fn test_keyframes_and_animations() {
    let scss_content = r#"
// Keyframe animations
@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(20px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

@keyframes spin {
    0% {
        transform: rotate(0deg);
    }
    100% {
        transform: rotate(360deg);
    }
}

@keyframes pulse {
    0%, 100% {
        transform: scale(1);
        opacity: 1;
    }
    50% {
        transform: scale(1.1);
        opacity: 0.8;
    }
}

// Using animations
.fade-in {
    animation: fadeIn 0.5s ease-out;
}

.spinner {
    animation: spin 2s linear infinite;
}

.pulse {
    animation: pulse 2s ease-in-out infinite;
}

// Complex animation with multiple properties
.complex-animation {
    animation-name: fadeIn, pulse;
    animation-duration: 0.5s, 2s;
    animation-timing-function: ease-out, ease-in-out;
    animation-iteration-count: 1, infinite;
    animation-delay: 0s, 0.5s;
}

// Duplicate animation usage
.fade-in-duplicate {
    animation: fadeIn 0.5s ease-out;
}

.spinner-duplicate {
    animation: spin 2s linear infinite;
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "test.scss").unwrap();

    let css_rules: Vec<_> =
        rules.iter().map(|func| convert_to_css_rule(func, scss_content)).collect();

    // Find animation rules
    let animation_rules: Vec<_> = css_rules
        .iter()
        .filter(|r| r.declarations.iter().any(|(k, _)| k.contains("animation")))
        .collect();

    println!("Animation rules found: {}", animation_rules.len());
    assert!(!animation_rules.is_empty(), "Should find animation rules");

    // Check for duplicate animations
    let analyzer = DuplicateAnalyzer::new(css_rules, 0.8);
    let result = analyzer.analyze();

    let animation_duplicates: Vec<_> = result
        .exact_duplicates
        .iter()
        .filter(|d| d.rule1.selector.contains("fade-in") || d.rule1.selector.contains("spinner"))
        .collect();

    assert!(!animation_duplicates.is_empty(), "Should find duplicate animation usages");
}

#[test]
fn test_css_grid_and_flexbox_complex() {
    let scss_content = r#"
// Complex Grid Layout
.grid-container {
    display: grid;
    grid-template-columns: [start] 1fr [content-start] 3fr [content-end] 1fr [end];
    grid-template-rows: [header] auto [main] 1fr [footer] auto;
    grid-template-areas:
        "header header header"
        "sidebar main aside"
        "footer footer footer";
    gap: 20px;
    
    @container (min-width: 800px) {
        grid-template-columns: [start] 200px [content-start] 1fr [content-end] 200px [end];
        gap: 30px;
    }
    
    .header {
        grid-area: header;
        grid-column: start / end;
    }
    
    .sidebar {
        grid-area: sidebar;
        
        @supports (display: grid) {
            display: grid;
            grid-template-rows: repeat(auto-fit, minmax(100px, 1fr));
            gap: 10px;
        }
    }
    
    .main {
        grid-area: main;
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
        gap: 20px;
    }
}

// Complex Flexbox Layout
.flex-container {
    display: flex;
    flex-flow: row wrap;
    justify-content: space-between;
    align-items: stretch;
    align-content: space-around;
    gap: 20px;
    
    .flex-item {
        flex: 1 1 300px;
        min-width: 0; // Prevent overflow
        
        &:first-child {
            flex: 2 1 400px;
        }
        
        &:last-child {
            flex: 0 0 200px;
        }
        
        &.grow {
            flex-grow: 3;
        }
        
        &.shrink {
            flex-shrink: 2;
        }
        
        &.center {
            align-self: center;
        }
    }
}

// Duplicate with shorthand variations
.grid-simple {
    display: grid;
    grid: auto / repeat(3, 1fr);
    gap: 20px;
}

.flex-simple {
    display: flex;
    flex: 1;
    gap: 20px;
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "test.scss").unwrap();

    let css_rules: Vec<_> =
        rules.iter().map(|func| convert_to_css_rule(func, scss_content)).collect();

    // Check for grid properties
    let grid_rules: Vec<_> = css_rules
        .iter()
        .filter(|r| r.declarations.iter().any(|(k, _)| k.starts_with("grid")))
        .collect();

    assert!(!grid_rules.is_empty(), "Should find grid rules");

    // Check for flex properties
    let flex_rules: Vec<_> = css_rules
        .iter()
        .filter(|r| r.declarations.iter().any(|(k, _)| k.starts_with("flex")))
        .collect();

    assert!(!flex_rules.is_empty(), "Should find flex rules");

    // Check named grid lines and areas
    let named_grid_rules: Vec<_> = css_rules
        .iter()
        .filter(|r| r.declarations.iter().any(|(_, v)| v.contains("[") && v.contains("]")))
        .collect();

    println!("Rules with named grid lines: {}", named_grid_rules.len());
    assert!(!named_grid_rules.is_empty(), "Should find named grid lines");
}

#[test]
fn test_css_functions_and_modern_features() {
    let scss_content = r#"
// Modern CSS functions
.modern-features {
    // Clamp for responsive sizing
    font-size: clamp(1rem, 2vw + 1rem, 3rem);
    padding: clamp(1rem, 5vw, 3rem);
    
    // Min/Max functions
    width: min(100%, 1200px);
    height: max(50vh, 400px);
    
    // Aspect ratio
    aspect-ratio: 16 / 9;
    
    // Logical properties
    margin-inline: auto;
    padding-block: 2rem;
    border-inline-start: 3px solid currentColor;
    
    // Color functions
    color: rgb(255 255 255 / 0.9);
    background: hsl(210deg 50% 50% / 0.8);
    border-color: hwb(210 30% 40%);
    
    // Filter and backdrop-filter
    filter: blur(5px) contrast(1.2) brightness(1.1);
    backdrop-filter: blur(10px) saturate(1.5);
    
    // Custom properties with fallbacks
    color: var(--text-color, var(--fallback-color, #333));
    
    // Container queries
    container-type: inline-size;
    container-name: card;
}

@container card (min-width: 400px) {
    .modern-features {
        display: grid;
        grid-template-columns: 1fr 2fr;
    }
}

// Subgrid
.grid-parent {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    grid-template-rows: repeat(3, 100px);
    
    .grid-child {
        display: grid;
        grid-template-columns: subgrid;
        grid-template-rows: subgrid;
        grid-column: span 2;
        grid-row: span 2;
    }
}

// Scroll snap
.scroll-container {
    scroll-snap-type: x mandatory;
    scroll-behavior: smooth;
    overscroll-behavior: contain;
    
    .scroll-item {
        scroll-snap-align: center;
        scroll-snap-stop: always;
        scroll-margin: 20px;
    }
}

// CSS Houdini properties
.houdini {
    @property --gradient-angle {
        syntax: '<angle>';
        initial-value: 0deg;
        inherits: false;
    }
    
    background: linear-gradient(var(--gradient-angle), #3498db, #2ecc71);
    animation: rotate-gradient 4s linear infinite;
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "test.scss").unwrap();

    let css_rules: Vec<_> =
        rules.iter().map(|func| convert_to_css_rule(func, scss_content)).collect();

    // Check modern CSS functions
    let modern_functions = ["clamp(", "min(", "max(", "rgb(", "hsl(", "hwb(", "var("];

    for func_name in &modern_functions {
        let rules_with_func: Vec<_> = css_rules
            .iter()
            .filter(|r| r.declarations.iter().any(|(_, v)| v.contains(func_name)))
            .collect();

        println!("Rules using {}: {}", func_name, rules_with_func.len());
        assert!(!rules_with_func.is_empty(), "Should find rules using {func_name}");
    }

    // Check logical properties
    let logical_props = ["margin-inline", "padding-block", "border-inline-start"];

    for prop in &logical_props {
        let rules_with_prop: Vec<_> =
            css_rules.iter().filter(|r| r.declarations.iter().any(|(k, _)| k == prop)).collect();

        assert!(!rules_with_prop.is_empty(), "Should find rules using {prop}");
    }
}

#[test]
fn test_multiple_selectors_and_grouping() {
    let scss_content = r#"
// Multiple selectors with nesting
h1, h2, h3,
h4, h5, h6 {
    font-family: 'Helvetica Neue', sans-serif;
    line-height: 1.2;
    margin-bottom: 1rem;
    
    &:first-child {
        margin-top: 0;
    }
    
    a {
        color: inherit;
        text-decoration: none;
        
        &:hover {
            text-decoration: underline;
        }
    }
}

// Complex grouping
input[type="text"],
input[type="email"],
input[type="password"],
input[type="number"],
input[type="tel"],
input[type="url"],
textarea,
select {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-family: inherit;
    font-size: inherit;
    
    &:focus {
        outline: none;
        border-color: #007bff;
        box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.25);
    }
    
    &:disabled {
        background-color: #e9ecef;
        cursor: not-allowed;
    }
    
    &::placeholder {
        color: #6c757d;
        opacity: 1;
    }
}

// Nested grouping
.card,
.panel,
.box {
    background: white;
    border-radius: 8px;
    padding: 1.5rem;
    
    h1, h2, h3 {
        margin-top: 0;
        
        + p {
            margin-top: 0.5rem;
        }
    }
    
    p, ul, ol {
        &:last-child {
            margin-bottom: 0;
        }
    }
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "test.scss").unwrap();

    println!("Total rules generated from grouped selectors: {}", rules.len());

    // Check heading rules
    let heading_rules: Vec<_> = rules
        .iter()
        .filter(|r| {
            r.name.starts_with("h") && r.name.chars().nth(1).is_some_and(|c| c.is_numeric())
        })
        .collect();

    assert!(heading_rules.len() >= 6, "Should generate rules for all heading levels");

    // Check input rules
    let input_rules: Vec<_> = rules.iter().filter(|r| r.name.contains("input[type=")).collect();

    assert!(!input_rules.is_empty(), "Should generate rules for input types");

    // Check nested combinations
    let nested_heading_rules: Vec<_> = rules
        .iter()
        .filter(|r| {
            (r.name.contains(".card") || r.name.contains(".panel") || r.name.contains(".box"))
                && r.name.contains(" h")
        })
        .collect();

    println!("Nested heading rules: {}", nested_heading_rules.len());
    assert!(!nested_heading_rules.is_empty(), "Should generate nested heading rules");
}

#[test]
fn test_import_and_use_patterns() {
    let scss_content = r#"
// Variables that would normally be imported
$breakpoint-sm: 576px;
$breakpoint-md: 768px;
$breakpoint-lg: 992px;
$breakpoint-xl: 1200px;

// Responsive utilities
@media (min-width: #{$breakpoint-sm}) {
    .hide-sm { display: none !important; }
    .show-sm { display: block !important; }
}

@media (min-width: #{$breakpoint-md}) {
    .hide-md { display: none !important; }
    .show-md { display: block !important; }
}

@media (min-width: #{$breakpoint-lg}) {
    .hide-lg { display: none !important; }
    .show-lg { display: block !important; }
}

// Mixins simulation
.button-base {
    display: inline-block;
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    text-decoration: none;
    transition: all 0.3s ease;
    
    &:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
    }
    
    &:active {
        transform: translateY(0);
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.15);
    }
}

.button-primary {
    @extend .button-base;
    background-color: #007bff;
    color: white;
    
    &:hover {
        background-color: #0056b3;
    }
}

.button-secondary {
    @extend .button-base;
    background-color: #6c757d;
    color: white;
    
    &:hover {
        background-color: #545b62;
    }
}

// Placeholder selectors
%clearfix {
    &::after {
        content: "";
        display: table;
        clear: both;
    }
}

.container {
    @extend %clearfix;
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 15px;
}

.row {
    @extend %clearfix;
    margin-left: -15px;
    margin-right: -15px;
}
"#;

    let mut parser = CssParser::new_scss();
    let rules = parser.extract_functions(scss_content, "test.scss").unwrap();

    let css_rules: Vec<_> =
        rules.iter().map(|func| convert_to_css_rule(func, scss_content)).collect();

    // Check media query rules
    let media_rules: Vec<_> = rules.iter().filter(|r| r.name.contains("@media")).collect();

    println!("Media query rules: {}", media_rules.len());

    // Analyze button patterns
    let _button_rules: Vec<_> =
        css_rules.iter().filter(|r| r.selector.contains("button")).collect();

    let analyzer = DuplicateAnalyzer::new(css_rules.clone(), 0.7);
    let result = analyzer.analyze();

    // Should find similar button styles
    let button_similarities: Vec<_> = result
        .style_duplicates
        .iter()
        .filter(|d| d.rule1.selector.contains("button") && d.rule2.selector.contains("button"))
        .collect();

    println!("Button style similarities found: {}", button_similarities.len());
    assert!(!button_similarities.is_empty(), "Should find similar button styles");
}
