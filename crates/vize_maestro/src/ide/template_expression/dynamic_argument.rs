//! Cursor boundaries for dynamic Vue directive arguments, including edits
//! whose JavaScript expression or closing bracket is not complete yet.

pub(super) fn contains_cursor(content: &str, open: usize, cursor: usize) -> bool {
    let prefix = content[..open]
        .rsplit(|ch: char| ch.is_ascii_whitespace() || matches!(ch, '<' | '>'))
        .next()
        .unwrap_or_default();
    if !matches!(prefix, ":" | "@" | "#") && !(prefix.starts_with("v-") && prefix.ends_with(':')) {
        return false;
    }
    let mut depth = 1;
    let mut quote = None;
    let mut escaped = false;
    for byte in content.as_bytes()[open + 1..cursor].iter().copied() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
        } else {
            match byte {
                b'\'' | b'"' | b'`' => quote = Some(byte),
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        return false;
                    }
                }
                b'>' | b'<' => return false,
                _ => {}
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::super::is_in_vue_template_expression;

    #[test]
    fn dynamic_arguments_share_expression_routing_for_all_directives() {
        for prefix in [":", "@", "#", "v-bind:", "v-on:", "v-slot:", "v-custom:"] {
            for expression in ["names.", "names.cur", "names[0].", "names[']']."] {
                let source = vize_s0::cstr!("<Comp {prefix}[{expression}] />");
                let cursor = source.find(expression).unwrap() + expression.len();
                assert!(is_in_vue_template_expression(&source, cursor), "{source}");
                assert!(
                    is_in_vue_template_expression(&source[..cursor], cursor),
                    "{source}"
                );
            }
        }
    }

    #[test]
    fn dynamic_arguments_do_not_capture_modifiers_values_or_text() {
        for source in [
            "<Comp :[names.current].prop",
            "<Comp data-[names.",
            "<Comp title=\"#[names.",
            "<Comp /> #[names.",
            "<!-- <Comp #[names.",
        ] {
            assert!(
                !is_in_vue_template_expression(source, source.len()),
                "{source}"
            );
        }
    }
}
