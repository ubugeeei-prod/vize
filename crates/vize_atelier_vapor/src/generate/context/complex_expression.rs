//! Text-scanning fallback for object/array literal expressions.
//!
//! Walks the raw expression source and prefixes only the identifiers that sit
//! in value position, leaving literals and object keys untouched.

use vize_carton::String;

use super::GenerateContext;

/// Resolve complex expressions (object/array literals) by prefixing identifiers inside
pub(super) fn resolve_complex_expression_fallback(ctx: &GenerateContext<'_>, expr: &str) -> String {
    let mut result = String::default();
    let mut chars = expr.chars().peekable();
    let mut in_string = false;
    let mut string_char = ' ';
    // Track whether we're in key position (after { or ,) vs value position (after :)
    let mut in_object = false;
    let mut is_key_position = false;

    while let Some(&ch) = chars.peek() {
        if in_string {
            result.push(ch);
            chars.next();
            if ch == string_char {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = ch;
                result.push(ch);
                chars.next();
            }
            '{' => {
                in_object = true;
                is_key_position = true;
                result.push(ch);
                chars.next();
            }
            '}' => {
                in_object = false;
                result.push(ch);
                chars.next();
            }
            ':' => {
                is_key_position = false;
                result.push(ch);
                chars.next();
            }
            ',' => {
                if in_object {
                    is_key_position = true;
                }
                result.push(ch);
                chars.next();
            }
            '[' => {
                // Computed property key: [expr] - contents should be prefixed
                if in_object && is_key_position {
                    // Save state, temporarily treat as value position
                    is_key_position = false;
                }
                result.push(ch);
                chars.next();
            }
            ']' => {
                // After computed key, we're back to key position until ':'
                if in_object {
                    is_key_position = true;
                }
                result.push(ch);
                chars.next();
            }
            ' ' | '\n' | '\t' => {
                result.push(ch);
                chars.next();
            }
            _ => {
                // Collect identifier/value
                let mut ident = String::default();
                while let Some(&c) = chars.peek() {
                    if c == ','
                        || c == '}'
                        || c == ']'
                        || c == ':'
                        || c == ' '
                        || c == '\n'
                        || c == '\t'
                    {
                        break;
                    }
                    ident.push(c);
                    chars.next();
                }
                let trimmed_ident = ident.trim();
                if trimmed_ident.is_empty()
                    || trimmed_ident == "true"
                    || trimmed_ident == "false"
                    || trimmed_ident == "null"
                    || trimmed_ident == "undefined"
                    || trimmed_ident.parse::<f64>().is_ok()
                    || (in_object && is_key_position)
                {
                    // Don't prefix: literals, empty, or object keys
                    result.push_str(&ident);
                } else {
                    result.push_str(&ctx.resolve_expression(trimmed_ident));
                }
            }
        }
    }
    result
}
