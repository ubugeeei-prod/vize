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
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut index = 0;

    while index < len {
        while index < len && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index + 1 >= len || bytes[index] != b'/' {
            return false;
        }
        {
            let next = bytes[index + 1];
            if next == b'/' {
                let start = index + 2;
                let mut end = start;
                while end < len && bytes[end] != b'\n' {
                    end += 1;
                }
                if is_typescript_suppression_directive(&expr[start..end]) {
                    return true;
                }
                index = end;
                if index < len {
                    index += 1;
                }
                continue;
            }

            if next == b'*' {
                let start = index + 2;
                index += 2;
                while index + 1 < len && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                    index += 1;
                }
                let end = index.min(len);
                if is_typescript_suppression_directive(&expr[start..end]) {
                    return true;
                }
                index = (index + 2).min(len);
                continue;
            }
        }

        return false;
    }

    false
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
        assert!(
            !expression_source_for_typecheck(object_literal)
                .as_ref()
                .contains("@ts-expect-error")
        );
    }
}
