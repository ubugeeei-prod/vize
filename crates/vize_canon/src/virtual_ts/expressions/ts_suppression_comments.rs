use std::borrow::Cow;
use vize_croquis::drawer::strip_js_comments;

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
    use super::{contains_typescript_suppression_comment, expression_source_for_typecheck};

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
