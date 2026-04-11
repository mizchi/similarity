pub(crate) fn has_similarity_ignore_directive(source_text: &str, start_line: usize) -> bool {
    if start_line <= 1 {
        return false;
    }

    let lines: Vec<&str> = source_text.lines().collect();
    if start_line > lines.len() + 1 {
        return false;
    }

    let mut current = start_line - 1;
    let mut in_block_comment = false;

    while current > 0 {
        current -= 1;
        let line = lines[current].trim();

        if line.is_empty() {
            continue;
        }

        if in_block_comment {
            if line.contains("similarity-ignore") {
                return true;
            }
            if line.starts_with("/*") || line.contains("/*") {
                in_block_comment = false;
            }
            continue;
        }

        if line.starts_with("//") {
            if line.contains("similarity-ignore") {
                return true;
            }
            continue;
        }

        if line.starts_with("/*") {
            return line.contains("similarity-ignore");
        }

        if line.ends_with("*/") || line.starts_with("*/") {
            if line.contains("similarity-ignore") {
                return true;
            }
            if !line.contains("/*") {
                in_block_comment = true;
            }
            continue;
        }

        if line.starts_with('*') {
            if line.contains("similarity-ignore") {
                return true;
            }
            continue;
        }

        break;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::has_similarity_ignore_directive;

    #[test]
    fn detects_single_line_directive() {
        let source = r#"
// similarity-ignore
function test() {}
"#;

        assert!(has_similarity_ignore_directive(source, 3));
    }

    #[test]
    fn detects_block_comment_directive() {
        let source = r#"
/**
 * similarity-ignore
 */
function test() {}
"#;

        assert!(has_similarity_ignore_directive(source, 5));
    }

    #[test]
    fn stops_at_non_comment_code() {
        let source = r#"
const marker = true;
// similarity-ignore

function test() {}
"#;

        assert!(has_similarity_ignore_directive(source, 5));
        assert!(!has_similarity_ignore_directive(source, 2));
    }
}
