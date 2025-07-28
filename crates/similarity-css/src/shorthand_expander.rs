
/// Expand CSS shorthand properties into their longhand equivalents
pub fn expand_shorthand_properties(declarations: &[(String, String)]) -> Vec<(String, String)> {
    let mut expanded = Vec::new();
    
    for (property, value) in declarations {
        match property.as_str() {
            "margin" => expand_box_model_shorthand(&mut expanded, "margin", value),
            "padding" => expand_box_model_shorthand(&mut expanded, "padding", value),
            "border" => expand_border_shorthand(&mut expanded, value),
            "border-radius" => expand_border_radius_shorthand(&mut expanded, value),
            "background" => expand_background_shorthand(&mut expanded, value),
            "font" => expand_font_shorthand(&mut expanded, value),
            "flex" => expand_flex_shorthand(&mut expanded, value),
            "grid" => expand_grid_shorthand(&mut expanded, value),
            "grid-template" => expand_grid_template_shorthand(&mut expanded, value),
            "gap" | "grid-gap" => expand_gap_shorthand(&mut expanded, value),
            "place-items" => expand_place_items_shorthand(&mut expanded, value),
            "place-content" => expand_place_content_shorthand(&mut expanded, value),
            "place-self" => expand_place_self_shorthand(&mut expanded, value),
            "overflow" => expand_overflow_shorthand(&mut expanded, value),
            "transition" => expand_transition_shorthand(&mut expanded, value),
            "animation" => expand_animation_shorthand(&mut expanded, value),
            _ => {
                // Not a shorthand property, keep as is
                expanded.push((property.clone(), value.clone()));
            }
        }
    }
    
    expanded
}

/// Expand margin/padding shorthand (1-4 values)
fn expand_box_model_shorthand(expanded: &mut Vec<(String, String)>, prefix: &str, value: &str) {
    let parts: Vec<&str> = value.trim().split_whitespace().collect();
    
    match parts.len() {
        1 => {
            // All sides same value
            let val = parts[0];
            expanded.push((format!("{}-top", prefix), val.to_string()));
            expanded.push((format!("{}-right", prefix), val.to_string()));
            expanded.push((format!("{}-bottom", prefix), val.to_string()));
            expanded.push((format!("{}-left", prefix), val.to_string()));
        }
        2 => {
            // vertical | horizontal
            let (vertical, horizontal) = (parts[0], parts[1]);
            expanded.push((format!("{}-top", prefix), vertical.to_string()));
            expanded.push((format!("{}-right", prefix), horizontal.to_string()));
            expanded.push((format!("{}-bottom", prefix), vertical.to_string()));
            expanded.push((format!("{}-left", prefix), horizontal.to_string()));
        }
        3 => {
            // top | horizontal | bottom
            let (top, horizontal, bottom) = (parts[0], parts[1], parts[2]);
            expanded.push((format!("{}-top", prefix), top.to_string()));
            expanded.push((format!("{}-right", prefix), horizontal.to_string()));
            expanded.push((format!("{}-bottom", prefix), bottom.to_string()));
            expanded.push((format!("{}-left", prefix), horizontal.to_string()));
        }
        4 => {
            // top | right | bottom | left
            expanded.push((format!("{}-top", prefix), parts[0].to_string()));
            expanded.push((format!("{}-right", prefix), parts[1].to_string()));
            expanded.push((format!("{}-bottom", prefix), parts[2].to_string()));
            expanded.push((format!("{}-left", prefix), parts[3].to_string()));
        }
        _ => {
            // Invalid, keep original
            expanded.push((prefix.to_string(), value.to_string()));
        }
    }
}

/// Expand border shorthand
fn expand_border_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    // Parse border: width style color
    let mut width = "medium";
    let mut style = "none";
    let mut color = "currentcolor";
    
    let parts: Vec<&str> = value.trim().split_whitespace().collect();
    
    for part in parts {
        if is_border_width(part) {
            width = part;
        } else if is_border_style(part) {
            style = part;
        } else {
            // Assume it's a color
            color = part;
        }
    }
    
    // Apply to all sides
    for side in &["top", "right", "bottom", "left"] {
        expanded.push((format!("border-{}-width", side), width.to_string()));
        expanded.push((format!("border-{}-style", side), style.to_string()));
        expanded.push((format!("border-{}-color", side), color.to_string()));
    }
}

/// Expand border-radius shorthand
fn expand_border_radius_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    let parts: Vec<&str> = value.trim().split_whitespace().collect();
    
    match parts.len() {
        1 => {
            let val = parts[0];
            expanded.push(("border-top-left-radius".to_string(), val.to_string()));
            expanded.push(("border-top-right-radius".to_string(), val.to_string()));
            expanded.push(("border-bottom-right-radius".to_string(), val.to_string()));
            expanded.push(("border-bottom-left-radius".to_string(), val.to_string()));
        }
        2 => {
            let (tl_br, tr_bl) = (parts[0], parts[1]);
            expanded.push(("border-top-left-radius".to_string(), tl_br.to_string()));
            expanded.push(("border-top-right-radius".to_string(), tr_bl.to_string()));
            expanded.push(("border-bottom-right-radius".to_string(), tl_br.to_string()));
            expanded.push(("border-bottom-left-radius".to_string(), tr_bl.to_string()));
        }
        3 => {
            let (tl, tr_bl, br) = (parts[0], parts[1], parts[2]);
            expanded.push(("border-top-left-radius".to_string(), tl.to_string()));
            expanded.push(("border-top-right-radius".to_string(), tr_bl.to_string()));
            expanded.push(("border-bottom-right-radius".to_string(), br.to_string()));
            expanded.push(("border-bottom-left-radius".to_string(), tr_bl.to_string()));
        }
        4 => {
            expanded.push(("border-top-left-radius".to_string(), parts[0].to_string()));
            expanded.push(("border-top-right-radius".to_string(), parts[1].to_string()));
            expanded.push(("border-bottom-right-radius".to_string(), parts[2].to_string()));
            expanded.push(("border-bottom-left-radius".to_string(), parts[3].to_string()));
        }
        _ => {
            expanded.push(("border-radius".to_string(), value.to_string()));
        }
    }
}

/// Expand background shorthand (simplified)
fn expand_background_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    // This is a simplified version - full background parsing is complex
    if value.contains("url(") || value.contains("linear-gradient") || value.contains("radial-gradient") {
        expanded.push(("background-image".to_string(), value.to_string()));
    } else if is_color_value(value) {
        expanded.push(("background-color".to_string(), value.to_string()));
    } else {
        // Keep original for complex cases
        expanded.push(("background".to_string(), value.to_string()));
    }
}

/// Expand font shorthand (simplified)
fn expand_font_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    // This is a simplified version
    let parts: Vec<&str> = value.trim().split_whitespace().collect();
    
    if parts.len() >= 2 {
        // Try to parse font-size/line-height font-family
        if let Some(size_pos) = parts.iter().position(|p| p.contains("px") || p.contains("em") || p.contains("rem") || p.contains("%")) {
            if size_pos < parts.len() - 1 {
                expanded.push(("font-size".to_string(), parts[size_pos].to_string()));
                let family = parts[(size_pos + 1)..].join(" ");
                expanded.push(("font-family".to_string(), family));
                return;
            }
        }
    }
    
    // Keep original for complex cases
    expanded.push(("font".to_string(), value.to_string()));
}

/// Expand flex shorthand
fn expand_flex_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    let parts: Vec<&str> = value.trim().split_whitespace().collect();
    
    match parts.len() {
        1 => {
            if parts[0] == "none" {
                expanded.push(("flex-grow".to_string(), "0".to_string()));
                expanded.push(("flex-shrink".to_string(), "0".to_string()));
                expanded.push(("flex-basis".to_string(), "auto".to_string()));
            } else if parts[0] == "auto" {
                expanded.push(("flex-grow".to_string(), "1".to_string()));
                expanded.push(("flex-shrink".to_string(), "1".to_string()));
                expanded.push(("flex-basis".to_string(), "auto".to_string()));
            } else if parts[0].parse::<f64>().is_ok() {
                // Single number is flex-grow
                expanded.push(("flex-grow".to_string(), parts[0].to_string()));
                expanded.push(("flex-shrink".to_string(), "1".to_string()));
                expanded.push(("flex-basis".to_string(), "0%".to_string()));
            } else {
                // Must be flex-basis
                expanded.push(("flex-grow".to_string(), "1".to_string()));
                expanded.push(("flex-shrink".to_string(), "1".to_string()));
                expanded.push(("flex-basis".to_string(), parts[0].to_string()));
            }
        }
        2 => {
            // flex-grow flex-shrink
            expanded.push(("flex-grow".to_string(), parts[0].to_string()));
            expanded.push(("flex-shrink".to_string(), parts[1].to_string()));
            expanded.push(("flex-basis".to_string(), "0%".to_string()));
        }
        3 => {
            // flex-grow flex-shrink flex-basis
            expanded.push(("flex-grow".to_string(), parts[0].to_string()));
            expanded.push(("flex-shrink".to_string(), parts[1].to_string()));
            expanded.push(("flex-basis".to_string(), parts[2].to_string()));
        }
        _ => {
            expanded.push(("flex".to_string(), value.to_string()));
        }
    }
}

/// Expand grid shorthand (simplified)
fn expand_grid_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    // Grid shorthand is very complex, keep simple cases
    if value.contains("/") {
        let parts: Vec<&str> = value.split("/").collect();
        if parts.len() == 2 {
            expanded.push(("grid-template-rows".to_string(), parts[0].trim().to_string()));
            expanded.push(("grid-template-columns".to_string(), parts[1].trim().to_string()));
            return;
        }
    }
    expanded.push(("grid".to_string(), value.to_string()));
}

/// Expand grid-template shorthand
fn expand_grid_template_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    if value.contains("/") {
        let parts: Vec<&str> = value.split("/").collect();
        if parts.len() == 2 {
            expanded.push(("grid-template-rows".to_string(), parts[0].trim().to_string()));
            expanded.push(("grid-template-columns".to_string(), parts[1].trim().to_string()));
            return;
        }
    }
    expanded.push(("grid-template".to_string(), value.to_string()));
}

/// Expand gap/grid-gap shorthand
fn expand_gap_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    let parts: Vec<&str> = value.trim().split_whitespace().collect();
    
    match parts.len() {
        1 => {
            expanded.push(("row-gap".to_string(), parts[0].to_string()));
            expanded.push(("column-gap".to_string(), parts[0].to_string()));
        }
        2 => {
            expanded.push(("row-gap".to_string(), parts[0].to_string()));
            expanded.push(("column-gap".to_string(), parts[1].to_string()));
        }
        _ => {
            expanded.push(("gap".to_string(), value.to_string()));
        }
    }
}

/// Expand place-items shorthand
fn expand_place_items_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    let parts: Vec<&str> = value.trim().split_whitespace().collect();
    
    match parts.len() {
        1 => {
            expanded.push(("align-items".to_string(), parts[0].to_string()));
            expanded.push(("justify-items".to_string(), parts[0].to_string()));
        }
        2 => {
            expanded.push(("align-items".to_string(), parts[0].to_string()));
            expanded.push(("justify-items".to_string(), parts[1].to_string()));
        }
        _ => {
            expanded.push(("place-items".to_string(), value.to_string()));
        }
    }
}

/// Expand place-content shorthand
fn expand_place_content_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    let parts: Vec<&str> = value.trim().split_whitespace().collect();
    
    match parts.len() {
        1 => {
            expanded.push(("align-content".to_string(), parts[0].to_string()));
            expanded.push(("justify-content".to_string(), parts[0].to_string()));
        }
        2 => {
            expanded.push(("align-content".to_string(), parts[0].to_string()));
            expanded.push(("justify-content".to_string(), parts[1].to_string()));
        }
        _ => {
            expanded.push(("place-content".to_string(), value.to_string()));
        }
    }
}

/// Expand place-self shorthand
fn expand_place_self_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    let parts: Vec<&str> = value.trim().split_whitespace().collect();
    
    match parts.len() {
        1 => {
            expanded.push(("align-self".to_string(), parts[0].to_string()));
            expanded.push(("justify-self".to_string(), parts[0].to_string()));
        }
        2 => {
            expanded.push(("align-self".to_string(), parts[0].to_string()));
            expanded.push(("justify-self".to_string(), parts[1].to_string()));
        }
        _ => {
            expanded.push(("place-self".to_string(), value.to_string()));
        }
    }
}

/// Expand overflow shorthand
fn expand_overflow_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    let parts: Vec<&str> = value.trim().split_whitespace().collect();
    
    match parts.len() {
        1 => {
            expanded.push(("overflow-x".to_string(), parts[0].to_string()));
            expanded.push(("overflow-y".to_string(), parts[0].to_string()));
        }
        2 => {
            expanded.push(("overflow-x".to_string(), parts[0].to_string()));
            expanded.push(("overflow-y".to_string(), parts[1].to_string()));
        }
        _ => {
            expanded.push(("overflow".to_string(), value.to_string()));
        }
    }
}

/// Expand transition shorthand (simplified)
fn expand_transition_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    // Transition is complex, keep simple parsing
    if value == "none" {
        expanded.push(("transition-property".to_string(), "none".to_string()));
    } else if value.contains("all") {
        expanded.push(("transition-property".to_string(), "all".to_string()));
        // Try to extract duration
        if let Some(duration) = extract_duration(value) {
            expanded.push(("transition-duration".to_string(), duration));
        }
    } else {
        expanded.push(("transition".to_string(), value.to_string()));
    }
}

/// Expand animation shorthand (simplified)
fn expand_animation_shorthand(expanded: &mut Vec<(String, String)>, value: &str) {
    // Animation is complex, keep simple parsing
    if value == "none" {
        expanded.push(("animation-name".to_string(), "none".to_string()));
    } else {
        expanded.push(("animation".to_string(), value.to_string()));
    }
}

// Helper functions

fn is_border_width(value: &str) -> bool {
    matches!(value, "thin" | "medium" | "thick") || 
    value.ends_with("px") || 
    value.ends_with("em") ||
    value.ends_with("rem")
}

fn is_border_style(value: &str) -> bool {
    matches!(
        value,
        "none" | "hidden" | "dotted" | "dashed" | "solid" | 
        "double" | "groove" | "ridge" | "inset" | "outset"
    )
}

fn is_color_value(value: &str) -> bool {
    value.starts_with('#') ||
    value.starts_with("rgb") ||
    value.starts_with("rgba") ||
    value.starts_with("hsl") ||
    value.starts_with("hsla") ||
    matches!(
        value,
        "red" | "green" | "blue" | "white" | "black" | "gray" | "grey" |
        "yellow" | "orange" | "purple" | "pink" | "transparent" | "currentcolor"
    )
}

fn extract_duration(value: &str) -> Option<String> {
    for part in value.split_whitespace() {
        if part.ends_with("s") || part.ends_with("ms") {
            return Some(part.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_margin_shorthand() {
        let decls = vec![("margin".to_string(), "10px".to_string())];
        let expanded = expand_shorthand_properties(&decls);
        assert_eq!(expanded.len(), 4);
        assert_eq!(expanded[0], ("margin-top".to_string(), "10px".to_string()));
        assert_eq!(expanded[1], ("margin-right".to_string(), "10px".to_string()));
        assert_eq!(expanded[2], ("margin-bottom".to_string(), "10px".to_string()));
        assert_eq!(expanded[3], ("margin-left".to_string(), "10px".to_string()));
    }

    #[test]
    fn test_margin_two_values() {
        let decls = vec![("margin".to_string(), "10px 20px".to_string())];
        let expanded = expand_shorthand_properties(&decls);
        assert_eq!(expanded.len(), 4);
        assert_eq!(expanded[0], ("margin-top".to_string(), "10px".to_string()));
        assert_eq!(expanded[1], ("margin-right".to_string(), "20px".to_string()));
        assert_eq!(expanded[2], ("margin-bottom".to_string(), "10px".to_string()));
        assert_eq!(expanded[3], ("margin-left".to_string(), "20px".to_string()));
    }

    #[test]
    fn test_flex_shorthand() {
        let decls = vec![("flex".to_string(), "1".to_string())];
        let expanded = expand_shorthand_properties(&decls);
        assert_eq!(expanded.len(), 3);
        assert_eq!(expanded[0], ("flex-grow".to_string(), "1".to_string()));
        assert_eq!(expanded[1], ("flex-shrink".to_string(), "1".to_string()));
        assert_eq!(expanded[2], ("flex-basis".to_string(), "0%".to_string()));
    }

    #[test]
    fn test_gap_shorthand() {
        let decls = vec![("gap".to_string(), "10px 20px".to_string())];
        let expanded = expand_shorthand_properties(&decls);
        assert_eq!(expanded.len(), 2);
        assert_eq!(expanded[0], ("row-gap".to_string(), "10px".to_string()));
        assert_eq!(expanded[1], ("column-gap".to_string(), "20px".to_string()));
    }

    #[test]
    fn test_non_shorthand() {
        let decls = vec![("color".to_string(), "red".to_string())];
        let expanded = expand_shorthand_properties(&decls);
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0], ("color".to_string(), "red".to_string()));
    }
}