//! Rust-syntax scan of `lint_template` / `run_over_template` calls.
//!
//! Call arguments are split on commas only at depth zero, and string literals
//! are decoded with Rust's cooked escapes (`\xNN`, `\u{...}`, continuation).
//! A textual comma search stops inside `make_rule(a, b)` and drops the template.

use vize_s0::String;

use super::fixture_cursor::Cursor;

pub(super) struct FixtureCall {
    /// 1-based line of the callee.
    pub line: usize,
    /// Decoded template argument, or `None` when it is not a string literal.
    pub template: Option<String>,
}

/// Every direct call in `source`. Definitions, comments, and string bodies
/// are not calls. Nested calls are still visited.
pub(super) fn fixture_calls(source: &str) -> std::vec::Vec<FixtureCall> {
    let mut out = std::vec::Vec::new();
    let mut cur = Cursor::new(source);
    let mut after_fn = false;
    while !cur.eof() {
        cur.skip_ws_and_comments();
        if cur.eof() {
            break;
        }
        if cur.skip_string_token() || cur.skip_char_or_lifetime() {
            after_fn = false;
            continue;
        }
        if let Some(start) = cur.ident_start() {
            let ident = cur.bump_ident();
            if ident == "fn" {
                after_fn = true;
                continue;
            }
            if !after_fn
                && (ident == "lint_template" || ident == "run_over_template")
                && let Some(open) = cur.call_paren()
            {
                let index = usize::from(ident == "run_over_template");
                let template = arg_ranges(source, open)
                    .and_then(|args| args.get(index).copied())
                    .and_then(|(from, to)| decode_template_arg(&source[from..to]));
                out.push(FixtureCall {
                    line: cur.line_at(start),
                    template,
                });
            }
            after_fn = false;
            continue;
        }
        after_fn = false;
        cur.bump();
    }
    out
}

/// The template argument: one string literal, or several adjacent ones
/// (Rust concatenates them). Anything else is not the literal the test runs.
fn decode_template_arg(arg: &str) -> Option<String> {
    let mut cur = Cursor::new(arg);
    let mut out = String::default();
    let mut saw = false;
    loop {
        cur.skip_ws_and_comments();
        if cur.eof() {
            break;
        }
        out.push_str(cur.decode_one_str()?.as_str());
        saw = true;
    }
    saw.then_some(out)
}

fn arg_ranges(source: &str, open: usize) -> Option<std::vec::Vec<(usize, usize)>> {
    let mut cur = Cursor::new(source);
    cur.i = open;
    if cur.peek() != '(' {
        return None;
    }
    cur.bump();
    let mut paren = 1i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    let mut start = cur.i;
    let mut args = std::vec::Vec::new();
    while !cur.eof() && paren > 0 {
        cur.skip_ws_and_comments();
        if cur.eof() {
            break;
        }
        if cur.skip_string_token() || cur.skip_char_or_lifetime() {
            continue;
        }
        match cur.peek() {
            '(' => {
                paren += 1;
                cur.bump();
            }
            ')' => {
                paren -= 1;
                if paren == 0 {
                    args.push((start, cur.i));
                    return Some(args);
                }
                cur.bump();
            }
            '[' => {
                bracket += 1;
                cur.bump();
            }
            ']' => {
                bracket -= 1;
                cur.bump();
            }
            '{' => {
                brace += 1;
                cur.bump();
            }
            '}' => {
                brace -= 1;
                cur.bump();
            }
            ',' if paren == 1 && bracket == 0 && brace == 0 => {
                args.push((start, cur.i));
                cur.bump();
                start = cur.i;
            }
            _ => cur.bump(),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::fixture_calls;

    #[test]
    fn nested_comma_keeps_the_template_and_rust_escapes() {
        let source = r##"
            fn run_over_template(rule: &R, source: &str) {}
            fn lint_template(source: &str, filename: &str) {}
            run_over_template(make_rule(a, b), "<div>");
            run_over_template(&rule, r#"<a, b>"#);
            lint_template("\x3cp\u{2022}\u{00_41}", "test.vue");
            lint_template(source, "test.vue");
            // run_over_template(make_rule(a, b), "<comment>");
            let hidden = "run_over_template(make_rule(a, b), \"<string>\")";
        "##;
        let calls = fixture_calls(source);
        let templates: std::vec::Vec<&str> = calls
            .iter()
            .filter_map(|call| call.template.as_deref())
            .collect();
        assert_eq!(templates, ["<div>", "<a, b>", "<p\u{2022}A"]);
        assert_eq!(
            calls.len(),
            4,
            "definitions, comments, and strings are not calls"
        );
        assert_eq!(
            calls.iter().filter(|call| call.template.is_none()).count(),
            1
        );
    }

    #[test]
    fn string_continuation_drops_the_newline_and_following_spaces() {
        let source = "lint_template(\"foo\\\n \tbar\", \"f\");\n";
        let calls = fixture_calls(source);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].template.as_deref(), Some("foobar"));
    }
}
