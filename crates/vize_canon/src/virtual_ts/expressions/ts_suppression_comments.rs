use std::borrow::Cow;
use vize_carton::String;
use vize_croquis::drawer::strip_js_comments;

/// Collapse line breaks before a callback's `=>` onto the property line that
/// carries its inference-only `@ts-ignore`.
///
/// `(\n  params\n) => body` otherwise leaves `params` on an unsuppressed line
/// and external content-mapper hosts report an unmapped TS7006. The scan skips
/// comments and literals so an arrow-shaped token inside either cannot end the
/// parameter prefix. `None` leaves the common single-line callback borrowed.
pub(crate) fn callback_parameters_on_ignored_line(value: &str) -> Option<String> {
    let arrow = arrow_token_offset(value)?;
    let prefix = value.get(..arrow)?;
    if !prefix.contains(['\n', '\r']) {
        return None;
    }
    let mut collapsed = String::with_capacity(value.len());
    for ch in prefix.chars() {
        collapsed.push(if ch == '\n' || ch == '\r' { ' ' } else { ch });
    }
    if let Some(rest) = value.get(arrow..) {
        collapsed.push_str(rest);
    }
    Some(collapsed)
}

fn arrow_token_offset(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut index: usize = 0;
    while bytes.get(index.saturating_add(1)).is_some() {
        let current = bytes.get(index).copied()?;
        match current {
            b'/' if bytes.get(index.saturating_add(1)) == Some(&b'/') => {
                index = index.saturating_add(2);
                while bytes.get(index).is_some_and(|byte| *byte != b'\n') {
                    index = index.saturating_add(1);
                }
            }
            b'/' if bytes.get(index.saturating_add(1)) == Some(&b'*') => {
                index = index.saturating_add(2);
                while bytes.get(index.saturating_add(1)).is_some()
                    && !(bytes.get(index) == Some(&b'*')
                        && bytes.get(index.saturating_add(1)) == Some(&b'/'))
                {
                    index = index.saturating_add(1);
                }
                index = index.saturating_add(2).min(bytes.len());
            }
            b'\'' | b'"' | b'`' => {
                index = index.saturating_add(1);
                while bytes.get(index).is_some_and(|byte| *byte != current) {
                    let skip = if bytes.get(index) == Some(&b'\\') {
                        2
                    } else {
                        1
                    };
                    index = index.saturating_add(skip);
                }
                index = index.saturating_add(1).min(bytes.len());
            }
            b'=' if bytes.get(index.saturating_add(1)) == Some(&b'>') => return Some(index),
            _ => index = index.saturating_add(1),
        }
    }
    None
}

pub(super) fn expression_source_for_typecheck(expr: &str) -> Cow<'_, str> {
    if contains_typescript_suppression_comment(expr) {
        Cow::Borrowed(expr)
    } else {
        strip_js_comments(expr)
    }
}

fn contains_typescript_suppression_comment(expr: &str) -> bool {
    let mut rest = expr;
    loop {
        rest = rest.trim_start_matches(|ch: char| ch.is_ascii_whitespace());
        let (body, tail) = if let Some(after) = rest.strip_prefix("//") {
            after.split_once('\n').unwrap_or((after, ""))
        } else if let Some(after) = rest.strip_prefix("/*") {
            after.split_once("*/").unwrap_or_else(|| {
                // An unterminated block comment is scanned up to its final character.
                let mut chars = after.chars();
                chars.next_back();
                (chars.as_str(), "")
            })
        } else {
            return false;
        };
        if is_typescript_suppression_directive(body) {
            return true;
        }
        rest = tail;
    }
}

fn is_typescript_suppression_directive(comment: &str) -> bool {
    let trimmed = comment.trim_start();
    trimmed.starts_with("@ts-ignore") || trimmed.starts_with("@ts-expect-error")
}

#[cfg(test)]
mod tests {
    use super::{
        callback_parameters_on_ignored_line, contains_typescript_suppression_comment,
        expression_source_for_typecheck,
    };

    #[test]
    fn allocates_only_when_callback_parameters_cross_lines() {
        assert_eq!(callback_parameters_on_ignored_line("(item) => item.id"), None);
        let collapsed = callback_parameters_on_ignored_line("(\n  item\n) => item.id");
        assert_eq!(
            collapsed.as_ref().map(|value| value.as_str()),
            Some("(   item ) => item.id")
        );
    }

    #[test]
    fn preserves_only_typescript_suppression_comments() {
        assert_eq!(
            expression_source_for_typecheck("// note\nvalue").as_ref(),
            "\nvalue"
        );
        assert_eq!(
            expression_source_for_typecheck("// @ts-ignore\nvalue").as_ref(),
            "// @ts-ignore\nvalue"
        );
        assert_eq!(
            expression_source_for_typecheck("/* @ts-expect-error */\nvalue").as_ref(),
            "/* @ts-expect-error */\nvalue"
        );
    }

    #[test]
    fn ignores_suppression_text_inside_literals() {
        assert!(!contains_typescript_suppression_comment(
            r#""// @ts-ignore" + value"#
        ));
        assert!(!contains_typescript_suppression_comment(
            r#"`/* @ts-expect-error */` + value"#
        ));
    }

    #[test]
    fn strips_non_leading_suppression_comments() {
        let object_literal = "{ name: 'route',\n// @ts-expect-error upstream note\nparams }";
        assert!(!contains_typescript_suppression_comment(object_literal));
        assert_eq!(
            expression_source_for_typecheck(object_literal).as_ref(),
            "{ name: 'route',\n\nparams }"
        );
    }
}
