//! Minimal JSONC normalization for TypeScript configuration files.

use serde_json::Value;
use vize_carton::String;

pub(super) fn parse_jsonc(text: &str) -> Option<Value> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let without_comments = strip_json_comments(text)?;
    let normalized = strip_trailing_commas(&without_comments);
    serde_json::from_str(&normalized).ok()
}

fn strip_json_comments(text: &str) -> Option<String> {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;

    while let Some(ch) = chars.next() {
        if line_comment {
            if ch == '\n' {
                line_comment = false;
                output.push('\n');
            }
            continue;
        }

        if block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                block_comment = false;
            } else if ch == '\n' {
                output.push('\n');
            }
            continue;
        }

        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            output.push(' ');
            line_comment = true;
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            output.push(' ');
            block_comment = true;
            continue;
        }

        output.push(ch);
    }

    (!block_comment).then_some(output)
}

fn strip_trailing_commas(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    while let Some(&ch) = chars.get(index) {
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            index += 1;
            continue;
        }

        if ch == ',' {
            let mut next = index + 1;
            while chars.get(next).is_some_and(|ch| ch.is_whitespace()) {
                next += 1;
            }
            if chars.get(next).is_some_and(|ch| matches!(ch, '}' | ']')) {
                index += 1;
                continue;
            }
        }

        output.push(ch);
        index += 1;
    }

    output
}
