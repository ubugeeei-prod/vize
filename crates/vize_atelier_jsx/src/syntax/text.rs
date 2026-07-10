//! JSX's frontend-neutral text whitespace normalization.

use vize_carton::String;

/// Clean JSX text using the Babel/Vue JSX line whitespace rules.
pub(crate) fn clean_jsx_text(raw: &str) -> String {
    let lines = raw
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect::<Vec<_>>();
    let last_non_blank = lines
        .iter()
        .rposition(|line| line.bytes().any(|byte| byte != b' ' && byte != b'\t'))
        .unwrap_or(0);
    let mut result = String::default();
    for (index, line) in lines.iter().enumerate() {
        let normalized = line.replace('\t', " ");
        let mut trimmed = normalized.as_str();
        if index != 0 {
            trimmed = trimmed.trim_start_matches(' ');
        }
        if index + 1 != lines.len() {
            trimmed = trimmed.trim_end_matches(' ');
        }
        if trimmed.is_empty() {
            continue;
        }
        result.push_str(trimmed);
        if index != last_non_blank {
            result.push(' ');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::clean_jsx_text;

    #[test]
    fn collapses_indentation_between_lines() {
        assert_eq!(
            clean_jsx_text("\n      Hello\n      World\n    "),
            "Hello World"
        );
    }

    #[test]
    fn preserves_single_line_spacing() {
        assert_eq!(clean_jsx_text("a   b"), "a   b");
        assert_eq!(clean_jsx_text("  hi  "), "  hi  ");
    }

    #[test]
    fn normalizes_blank_lines_and_crlf() {
        assert_eq!(clean_jsx_text("\n   \n   \n"), "");
        assert_eq!(clean_jsx_text("a\r\n  b"), "a b");
    }
}
