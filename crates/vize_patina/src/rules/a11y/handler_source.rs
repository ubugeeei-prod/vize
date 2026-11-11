//! Which `v-on` values eslint-plugin-vuejs-accessibility's `hasOnDirective`
//! counts, from how vue-eslint-parser 10 parses them.

/// vue-eslint-parser parses a function or a simple path as one expression and
/// anything else as a statement list (`VOnExpression`, which has a `body`).
/// Of the single expressions only an identifier or a function counts.
pub(super) fn is_handler_source(source: &str) -> bool {
    if is_function_expression(source) {
        return true;
    }
    if !is_simple_path(source) {
        return true;
    }
    !source.contains(['.', '[']) && !matches!(source, "null" | "true" | "false" | "this")
}

/// `IS_FUNCTION_EXPRESSION` of vue-eslint-parser 10:
/// `^\s*([\w$_]+|(async\s*)?\([^)]*?\))\s*(:[^=]+)?=>|^\s*(async\s+)?function(?:\s+[\w$]+)?\s*\(`.
fn is_function_expression(source: &str) -> bool {
    let source = source.trim_start();
    // The text after a leading run of `[\w$]`.
    fn after_word(text: &str) -> &str {
        text.trim_start_matches(|c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '$'))
    }
    let arrow_after = |rest: &str| {
        let rest = rest.trim_start();
        match rest.strip_prefix(':') {
            Some(annotation) => annotation
                .split_once('=')
                .is_some_and(|(kind, rest)| !kind.is_empty() && rest.starts_with('>')),
            None => rest.starts_with("=>"),
        }
    };
    fn parenthesized(text: &str) -> Option<&str> {
        let rest = text.strip_prefix('(')?;
        rest.split_once(')').map(|(_, after)| after)
    }
    let rest = after_word(source);
    if rest.len() < source.len() && arrow_after(rest) {
        return true;
    }
    let params = source.strip_prefix("async").map_or(source, str::trim_start);
    if parenthesized(params)
        .or_else(|| parenthesized(source))
        .is_some_and(arrow_after)
    {
        return true;
    }
    let function = source
        .strip_prefix("async")
        .filter(|rest| rest.starts_with(char::is_whitespace))
        .map_or(source, str::trim_start);
    function.strip_prefix("function").is_some_and(|rest| {
        let trimmed = rest.trim_start();
        let named = after_word(trimmed);
        let after_name = if trimmed.len() < rest.len() && named.len() < trimmed.len() {
            named
        } else {
            rest
        };
        after_name.trim_start().starts_with('(')
    })
}

/// `IS_SIMPLE_PATH` of vue-eslint-parser 10.
fn is_simple_path(source: &str) -> bool {
    let bytes = source.as_bytes();
    let is_start = |byte: u8| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$');
    let is_part = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$');
    let identifier = |at: usize| -> Option<usize> {
        if !bytes.get(at).copied().is_some_and(is_start) {
            return None;
        }
        let mut end = at + 1;
        while bytes.get(end).copied().is_some_and(is_part) {
            end += 1;
        }
        Some(end)
    };
    let Some(mut at) = identifier(0) else {
        return false;
    };
    while let Some(&byte) = bytes.get(at) {
        at = match byte {
            b'.' => match identifier(at + 1) {
                Some(end) => end,
                None => return false,
            },
            b'[' => {
                let inner = at + 1;
                let close = match bytes.get(inner) {
                    Some(&quote @ (b'\'' | b'"')) => {
                        match bytes
                            .get(inner + 1..)
                            .and_then(|tail| tail.iter().position(|&byte| byte == quote))
                        {
                            Some(offset) => inner + 1 + offset + 1,
                            None => return false,
                        }
                    }
                    Some(byte) if byte.is_ascii_digit() => {
                        let mut end = inner;
                        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                            end += 1;
                        }
                        end
                    }
                    _ => match identifier(inner) {
                        Some(end) => end,
                        None => return false,
                    },
                };
                if bytes.get(close) != Some(&b']') {
                    return false;
                }
                close + 1
            }
            _ => return false,
        };
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{is_function_expression, is_handler_source, is_simple_path};

    #[test]
    fn handler_sources_follow_vue_eslint_parser() {
        for source in [
            "onEvent",
            "undefined",
            "() => x",
            "(a, b) => a",
            "x => x",
            "async () => 1",
            "async (e) => e",
            "(e: Event): void => e",
            "function () {}",
            "async function named() {}",
            "select(option)",
            "void 0",
            "a?.b",
            "count++",
            "a; b",
        ] {
            assert!(is_handler_source(source), "{source}");
        }
        for source in [
            "keys.onKeydown",
            "a[0]",
            "a['b']",
            "a[b].c",
            "null",
            "true",
            "this",
        ] {
            assert!(!is_handler_source(source), "{source}");
        }
        assert!(is_simple_path("a.b[\"c\"][1][d]"));
        assert!(!is_simple_path("a.b()"));
        assert!(!is_function_expression("functional"));
        assert!(!is_function_expression("asyncFn"));
    }
}
