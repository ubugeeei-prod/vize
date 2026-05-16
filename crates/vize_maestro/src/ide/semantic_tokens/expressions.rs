//! Expression tokenization for semantic highlighting.
//!
//! Handles tokenization of JavaScript/TypeScript expressions found in
//! template interpolations, directive values, and event handlers.

use super::{
    encoding::{is_ident_char, is_ident_start, offset_to_line_col},
    types::{AbsoluteToken, TokenType},
};

/// Tokenize a JavaScript/TypeScript expression for syntax highlighting.
pub(crate) fn tokenize_expression(
    expr: &str,
    template: &str,
    expr_offset: usize,
    base_line: u32,
    tokens: &mut Vec<AbsoluteToken>,
) {
    let bytes = expr.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i] as char;

        // Skip whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Numbers
        if c.is_ascii_digit()
            || (c == '.' && i + 1 < bytes.len() && (bytes[i + 1] as char).is_ascii_digit())
        {
            let start = i;
            while i < bytes.len() {
                let ch = bytes[i] as char;
                if ch.is_ascii_digit()
                    || ch == '.'
                    || ch == 'e'
                    || ch == 'E'
                    || ch == '_'
                    || (ch == '-' && i > start && (bytes[i - 1] == b'e' || bytes[i - 1] == b'E'))
                {
                    i += 1;
                } else {
                    break;
                }
            }
            let abs_offset = expr_offset + start;
            let (line, col) = offset_to_line_col(template, abs_offset);
            tokens.push(AbsoluteToken {
                line: base_line + line - 1,
                start: col,
                length: (i - start) as u32,
                token_type: TokenType::Number as u32,
                modifiers: 0,
            });
            continue;
        }

        // String literals within expressions ('...' or `...`)
        if c == '\'' || c == '`' {
            let quote = c;
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i] as char != quote {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2; // skip escaped char
                } else {
                    i += 1;
                }
            }
            if i < bytes.len() {
                i += 1; // closing quote
            }
            let abs_offset = expr_offset + start;
            let (line, col) = offset_to_line_col(template, abs_offset);
            tokens.push(AbsoluteToken {
                line: base_line + line - 1,
                start: col,
                length: (i - start) as u32,
                token_type: TokenType::String as u32,
                modifiers: 0,
            });
            continue;
        }

        // Identifiers and keywords
        if is_ident_start(c) {
            let start = i;
            while i < bytes.len() && is_ident_char(bytes[i] as char) {
                i += 1;
            }
            let ident = &expr[start..i];
            let abs_offset = expr_offset + start;
            let (line, col) = offset_to_line_col(template, abs_offset);

            // Determine token type
            let token_type = if is_keyword(ident) || is_boolean_or_null(ident) {
                TokenType::Keyword
            } else if looks_like_function_call(expr, start) {
                TokenType::Function
            } else if looks_like_property_access(expr, start) {
                TokenType::Property
            } else {
                TokenType::Variable
            };

            tokens.push(AbsoluteToken {
                line: base_line + line - 1,
                start: col,
                length: ident.len() as u32,
                token_type: token_type as u32,
                modifiers: 0,
            });
            continue;
        }

        // Operators
        if is_operator_start(c) {
            let start = i;
            // Multi-character operators: ===, !==, >=, <=, ==, !=, &&, ||, ??, ?., +=, -=, etc.
            let op_len = operator_length(&expr[i..]);
            i += op_len;
            let abs_offset = expr_offset + start;
            let (line, col) = offset_to_line_col(template, abs_offset);
            tokens.push(AbsoluteToken {
                line: base_line + line - 1,
                start: col,
                length: op_len as u32,
                token_type: TokenType::Operator as u32,
                modifiers: 0,
            });
            continue;
        }

        // Skip other characters (parentheses, brackets, commas, etc.)
        i += 1;
    }
}

/// Check if character starts an operator.
fn is_operator_start(c: char) -> bool {
    matches!(
        c,
        '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '?' | ':' | '^' | '~'
    )
}

/// Get the length of an operator at the start of the string.
fn operator_length(s: &str) -> usize {
    let bytes = s.as_bytes();
    if bytes.len() >= 3 {
        let three = &s[..3];
        if matches!(
            three,
            "===" | "!==" | ">>>" | "<<=" | ">>=" | "&&=" | "||=" | "??="
        ) {
            return 3;
        }
    }
    if bytes.len() >= 2 {
        let two = &s[..2];
        if matches!(
            two,
            "==" | "!="
                | "<="
                | ">="
                | "&&"
                | "||"
                | "??"
                | "?."
                | "++"
                | "--"
                | "+="
                | "-="
                | "*="
                | "/="
                | "%="
                | "<<"
                | ">>"
                | "=>"
                | "**"
        ) {
            return 2;
        }
    }
    1
}

/// Check if identifier is a JavaScript keyword.
pub(crate) fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "if" | "else"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "return"
            | "throw"
            | "try"
            | "catch"
            | "finally"
            | "new"
            | "delete"
            | "typeof"
            | "instanceof"
            | "in"
            | "of"
            | "void"
            | "this"
            | "super"
            | "class"
            | "extends"
            | "const"
            | "let"
            | "var"
            | "function"
            | "async"
            | "await"
            | "yield"
            | "import"
            | "export"
            | "default"
            | "from"
            | "as"
    )
}

/// Check if identifier is a boolean or null literal.
pub(crate) fn is_boolean_or_null(s: &str) -> bool {
    matches!(s, "true" | "false" | "null" | "undefined")
}

/// Check if identifier looks like a property access (preceded by `.`).
fn looks_like_property_access(expr: &str, offset: usize) -> bool {
    if offset == 0 {
        return false;
    }

    let bytes = expr.as_bytes();
    let mut i = offset - 1;

    // Skip whitespace
    while i > 0 && (bytes[i] as char).is_whitespace() {
        i -= 1;
    }

    // Check for dot
    bytes[i] == b'.'
}

/// Check if identifier looks like a function call.
pub(crate) fn looks_like_function_call(expr: &str, offset: usize) -> bool {
    let bytes = expr.as_bytes();
    let mut i = offset;

    // Skip the identifier
    while i < bytes.len() && is_ident_char(bytes[i] as char) {
        i += 1;
    }

    // Skip whitespace
    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
        i += 1;
    }

    // Check for opening paren
    i < bytes.len() && bytes[i] == b'('
}

/// Extract identifiers from an expression.
#[cfg(test)]
pub(crate) fn extract_identifiers(expr: &str) -> Vec<(&str, usize)> {
    let mut identifiers = Vec::new();
    let bytes = expr.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Skip non-identifier characters
        while i < bytes.len() && !is_ident_start(bytes[i] as char) {
            i += 1;
        }

        if i >= bytes.len() {
            break;
        }

        let start = i;

        // Read the identifier
        while i < bytes.len() && is_ident_char(bytes[i] as char) {
            i += 1;
        }

        if start < i {
            let ident = &expr[start..i];
            // Skip keywords and literals
            if !super::encoding::is_keyword_or_literal(ident) {
                identifiers.push((ident, start));
            }
        }
    }

    identifiers
}
