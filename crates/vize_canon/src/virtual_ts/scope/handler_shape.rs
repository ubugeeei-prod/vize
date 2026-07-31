//! Lightweight JS scanning that classifies an `@event` handler body.
//!
//! A handler is either a callable reference (`handler`, `form?.submit`,
//! `handlers[key]`), an inline callback (`(v) => take(v)`, `function () {}`),
//! or a statement to run as-is. The three shapes are generated differently, so
//! this scanning decides which; it deliberately stays a scanner rather than a
//! parser, matching only the spellings a template attribute can hold.

use super::expression_scanner::{matching_paren_index, skip_js_trivia, top_level_arrow_index};

pub(super) fn inline_callback_event_argument(content: &str) -> Option<&'static str> {
    let trimmed = strip_outer_parentheses(content.trim());
    if trimmed.is_empty() {
        return None;
    }

    let function = strip_async_prefix(trimmed);
    if let Some(rest) = function.strip_prefix("function")
        && !rest.chars().next().is_some_and(is_identifier_continue)
    {
        let paren_start = function.len() - rest.len() + rest.find('(')?;
        let paren_end = matching_paren_index(function, paren_start)?;
        let inner = &function[paren_start + 1..paren_end];
        return Some(if inner.trim().is_empty() {
            ""
        } else {
            "$event"
        });
    }

    if let Some(arrow_idx) = top_level_arrow_index(trimmed) {
        let before_arrow = strip_async_prefix(trimmed[..arrow_idx].trim_end()).trim();
        if before_arrow.is_empty() {
            return None;
        }

        if let Some(is_empty) = parenthesized_params_are_empty(before_arrow) {
            return Some(if is_empty { "" } else { "$event" });
        }

        return is_identifier_segment(before_arrow).then_some("$event");
    }
    None
}

fn strip_outer_parentheses(mut input: &str) -> &str {
    while input.starts_with('(')
        && matching_paren_index(input, 0).is_some_and(|close| close == input.len() - 1)
    {
        input = input[1..input.len() - 1].trim();
    }
    input
}

fn strip_async_prefix(input: &str) -> &str {
    let Some(rest) = input.strip_prefix("async") else {
        return input;
    };
    if rest.chars().next().is_some_and(is_identifier_continue) {
        input
    } else {
        let content_start = skip_js_trivia(rest, 0);
        if content_start == 0 {
            input
        } else {
            &rest[content_start..]
        }
    }
}

fn parenthesized_params_are_empty(input: &str) -> Option<bool> {
    if !input.starts_with('(') {
        return None;
    }
    let close = matching_paren_index(input, 0)?;
    let suffix = &input[close + 1..];
    if skip_js_trivia(suffix, 0) != suffix.len() {
        return None;
    }
    Some(input[1..close].trim().is_empty())
}

pub(super) fn is_callable_handler_reference(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() || trimmed == "undefined" {
        return false;
    }

    let Some(mut idx) = parse_identifier_segment(trimmed, 0) else {
        return false;
    };

    loop {
        idx = skip_ascii_whitespace(trimmed, idx);
        if idx == trimmed.len() {
            return true;
        }

        let rest = &trimmed[idx..];
        if rest.starts_with("?.[") {
            idx += 2;
            let Some(next_idx) = parse_bracket_member(trimmed, idx) else {
                return false;
            };
            idx = next_idx;
        } else if rest.starts_with("?.") {
            let Some(next_idx) = parse_identifier_segment(trimmed, idx + 2) else {
                return false;
            };
            idx = next_idx;
        } else if rest.starts_with('.') {
            let Some(next_idx) = parse_identifier_segment(trimmed, idx + 1) else {
                return false;
            };
            idx = next_idx;
        } else if rest.starts_with('[') {
            let Some(next_idx) = parse_bracket_member(trimmed, idx) else {
                return false;
            };
            idx = next_idx;
        } else {
            return false;
        }
    }
}

fn is_identifier_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first == '$' || first.is_alphabetic()) {
        return false;
    }

    chars.all(|ch| ch == '_' || ch == '$' || ch.is_alphanumeric())
}

fn parse_identifier_segment(input: &str, start: usize) -> Option<usize> {
    let mut chars = input.get(start..)?.char_indices();
    let (_, first) = chars.next()?;
    if !is_identifier_start(first) {
        return None;
    }

    let mut end = start + first.len_utf8();
    for (offset, ch) in chars {
        if !is_identifier_continue(ch) {
            break;
        }
        end = start + offset + ch.len_utf8();
    }
    Some(end)
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_alphanumeric()
}

fn skip_ascii_whitespace(input: &str, mut idx: usize) -> usize {
    while input
        .as_bytes()
        .get(idx)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        idx += 1;
    }
    idx
}

fn parse_bracket_member(input: &str, open_index: usize) -> Option<usize> {
    if input.as_bytes().get(open_index) != Some(&b'[') {
        return None;
    }

    let mut depth = 0u32;
    let mut quote = None;
    let mut escaped = false;
    for (idx, ch) in input
        .char_indices()
        .skip_while(|(idx, _)| *idx < open_index)
    {
        if let Some(quote_ch) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            '[' => depth += 1,
            ']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(idx + ch.len_utf8());
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{inline_callback_event_argument, is_callable_handler_reference};

    #[test]
    fn inline_callbacks_report_the_event_argument_they_accept() {
        // (handler body, argument the generated call must pass)
        let cases = [
            ("() => f()", Some("")),
            ("(v) => f(v)", Some("$event")),
            ("v => f(v)", Some("$event")),
            ("async (v) => f(v)", Some("$event")),
            ("async v => f(v)", Some("$event")),
            ("((v) => f(v))", Some("$event")),
            (r#"((x = ")") => x)"#, Some("$event")),
            ("(value) /* callback */ => value", Some("$event")),
            ("async /* callback */ (value) => value", Some("$event")),
            ("(fn: (x: string) => string) => fn(\"x\")", Some("$event")),
            ("(fn = (x) => x) => fn(1)", Some("$event")),
            ("function () {}", Some("")),
            ("function (v) { f(v) }", Some("$event")),
            ("function (v) { return () => v }", Some("$event")),
            ("function named(v) { f(v) }", Some("$event")),
            ("async function (v) { f(v) }", Some("$event")),
            ("(async function (v) { f(v) })", Some("$event")),
            ("handler", None),
            ("handlers[key]", None),
            ("jobs.map((job) => job.id)", None),
            ("jobs.map(function (job) { return job.id })", None),
            ("/=>/.test(value)", None),
            // Starts with the `function` keyword's letters but is a call.
            ("functionalHandler(evt)", None),
        ];

        for (content, expected) in cases {
            assert_eq!(
                inline_callback_event_argument(content),
                expected,
                "unexpected inline-callback classification for {content:?}"
            );
        }
    }

    #[test]
    fn undefined_is_not_a_callable_handler_reference() {
        assert!(!is_callable_handler_reference("undefined"));
        assert!(!is_callable_handler_reference("  undefined  "));
    }

    #[test]
    fn actual_handler_references_stay_callable() {
        assert!(is_callable_handler_reference("handler"));
        assert!(is_callable_handler_reference("handlers[key]"));
        assert!(is_callable_handler_reference("form?.submit"));
    }
}
