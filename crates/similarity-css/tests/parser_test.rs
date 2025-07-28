use similarity_core::language_parser::LanguageParser;
use similarity_css::CssParser;

#[test]
fn test_parse_css_rules() {
    let content = r#"
        .button {
            background-color: blue;
            color: white;
            padding: 10px;
        }
        
        #header {
            display: flex;
            justify-content: center;
        }
        
        @media (max-width: 768px) {
            .button {
                width: 100%;
            }
        }
    "#;
    
    let mut parser = CssParser::new();
    let result = parser.extract_functions(content, "test.css");
    
    assert!(result.is_ok());
    let functions = result.unwrap();
    assert_eq!(functions.len(), 3);
    
    assert_eq!(functions[0].name, ".button");
    assert_eq!(functions[1].name, "#header");
    assert_eq!(functions[2].name, "@media");
}

#[test]
fn test_parse_scss() {
    let content = r#"
        $primary: blue;
        
        @mixin button-style {
            padding: 10px;
            border: none;
        }
        
        .button {
            @include button-style;
            background: $primary;
            color: white;
        }
    "#;
    
    let mut parser = CssParser::new_scss();
    let result = parser.extract_functions(content, "test.scss");
    
    assert!(result.is_ok());
    let functions = result.unwrap();
    
    let mixin = functions.iter().find(|f| f.name.starts_with("@mixin"));
    assert!(mixin.is_some());
    assert_eq!(mixin.unwrap().name, "@mixin button-style");
    
    let rule = functions.iter().find(|f| f.name == ".button");
    assert!(rule.is_some());
}

#[test]
fn test_parse_complex_selectors() {
    let content = r#"
        .button.primary {
            background: blue;
        }
        
        .card > .header {
            font-size: 18px;
        }
        
        input[type="text"] {
            border: 1px solid gray;
        }
        
        a:hover {
            text-decoration: underline;
        }
        
        p::first-line {
            font-weight: bold;
        }
    "#;
    
    let mut parser = CssParser::new();
    let result = parser.extract_functions(content, "test.css");
    
    assert!(result.is_ok());
    let functions = result.unwrap();
    assert_eq!(functions.len(), 5);
    
    assert!(functions[0].name.contains(".button"));
    assert!(functions[0].name.contains(".primary"));
    
    assert!(functions[1].name.contains(".card"));
    assert!(functions[1].name.contains(".header"));
    
    assert!(functions[2].name.contains("input"));
    assert!(functions[2].name.contains("["));
    
    assert!(functions[3].name.contains(":hover"));
    
    assert!(functions[4].name.contains("::first-line"));
}

#[test]
fn test_parse_empty_file() {
    let content = "";
    let mut parser = CssParser::new();
    let result = parser.extract_functions(content, "empty.css");
    
    assert!(result.is_ok());
    let functions = result.unwrap();
    assert_eq!(functions.len(), 0);
}

#[test]
fn test_parse_invalid_css() {
    let content = "{ invalid css }";
    let mut parser = CssParser::new();
    let result = parser.extract_functions(content, "invalid.css");
    
    // Tree-sitter should still parse something
    assert!(result.is_ok());
}