//! String/comment tracking state and brace/paren counting utilities.
//!
//! These utilities properly handle string literals, template literals,
//! block comments, and `${...}` expressions when counting braces and
//! parentheses in JavaScript/TypeScript code.

use vize_carton::String;

/// State for tracking string/template literal/comment context across multiple lines.
/// Required because template literals (backtick strings) and block comments can span
/// multiple lines, and `${...}` expressions within template literals contain code-mode
/// braces that must be distinguished from surrounding code braces.
#[derive(Clone)]
pub(super) struct StringTrackState {
    /// Whether we're inside a string literal (regular or template literal text portion)
    pub(super) in_string: bool,
    /// The quote character of the current string ('\'' | '"' | '`')
    string_char: u8,
    /// Whether the last character was a backslash escape
    escape: bool,
    /// Stack for nested template literal `${...}` expressions.
    /// Each entry tracks the brace depth within that expression.
    /// When a `}` is encountered and the top depth is 0, the expression ends
    /// and we return to the enclosing template literal text.
    pub(super) template_expr_brace_stack: Vec<i32>,
    /// Whether we're inside a `/* ... */` block comment.
    /// Block comments can span multiple lines and may contain quote characters
    /// (e.g., `doesn't`) that must not be treated as string delimiters.
    in_block_comment: bool,
}

impl Default for StringTrackState {
    fn default() -> Self {
        Self {
            in_string: false,
            string_char: b'\0',
            escape: false,
            template_expr_brace_stack: Vec::new(),
            in_block_comment: false,
        }
    }
}

/// Count net brace depth change ({ minus }) in a line, properly tracking
/// string literals, block comments, and template literal `${...}` expressions.
/// State is carried across lines to handle multiline template literals and comments.
pub(super) fn count_braces_with_state(line: &str, state: &mut StringTrackState) -> i32 {
    let mut count: i32 = 0;
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let ch = bytes[i];

        if state.escape {
            state.escape = false;
            i += 1;
            continue;
        }

        // Inside a block comment: skip everything until */
        if state.in_block_comment {
            if ch == b'*' && i + 1 < len && bytes[i + 1] == b'/' {
                state.in_block_comment = false;
                i += 2; // Skip both * and /
                continue;
            }
            i += 1;
            continue;
        }

        if state.in_string {
            if ch == b'\\' {
                state.escape = true;
                i += 1;
                continue;
            }

            if state.string_char == b'`' {
                if ch == b'`' {
                    // Close template literal
                    state.in_string = false;
                } else if ch == b'$' && i + 1 < len && bytes[i + 1] == b'{' {
                    // Enter template expression ${...}
                    // The ${ is template syntax, not a code brace
                    state.in_string = false;
                    state.template_expr_brace_stack.push(0);
                    i += 2; // Skip both $ and {
                    continue;
                }
            } else if ch == state.string_char {
                // Close regular string (' or ")
                state.in_string = false;
            }
        } else {
            // Not in string - we're in code mode
            match ch {
                b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                    // Enter block comment /*
                    state.in_block_comment = true;
                    i += 2; // Skip both / and *
                    continue;
                }
                b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                    // Line comment // -- skip rest of line
                    break;
                }
                b'\'' | b'"' => {
                    state.in_string = true;
                    state.string_char = ch;
                }
                b'`' => {
                    state.in_string = true;
                    state.string_char = b'`';
                }
                b'{' => {
                    if let Some(depth) = state.template_expr_brace_stack.last_mut() {
                        *depth += 1;
                    }
                    count += 1;
                }
                b'}' => {
                    if let Some(&depth) = state.template_expr_brace_stack.last() {
                        if depth == 0 {
                            // This } closes the ${...} expression, not a code brace
                            state.template_expr_brace_stack.pop();
                            state.in_string = true;
                            state.string_char = b'`';
                            i += 1;
                            continue;
                        } else {
                            // Nested brace inside ${...} expression
                            *state.template_expr_brace_stack.last_mut().unwrap() -= 1;
                            count -= 1;
                        }
                    } else {
                        count -= 1;
                    }
                }
                _ => {}
            }
        }

        i += 1;
    }

    count
}

/// Count net brace depth change ({  minus }) in a line, ignoring braces inside string literals.
/// NOTE: This function does NOT track state across lines. For multiline template literals,
/// use `count_braces_with_state` instead.
#[cfg(test)]
pub(super) fn count_braces_outside_strings(line: &str) -> i32 {
    let mut state = StringTrackState::default();
    count_braces_with_state(line, &mut state)
}

/// Count net paren depth change (( minus )) in a line, properly tracking
/// string literals, block comments, and template literal `${...}` expressions.
/// State is carried across lines to handle multiline template literals and comments.
pub(super) fn count_parens_with_state(line: &str, state: &mut StringTrackState) -> i32 {
    let mut count: i32 = 0;
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let ch = bytes[i];

        if state.escape {
            state.escape = false;
            i += 1;
            continue;
        }

        // Inside a block comment: skip everything until */
        if state.in_block_comment {
            if ch == b'*' && i + 1 < len && bytes[i + 1] == b'/' {
                state.in_block_comment = false;
                i += 2; // Skip both * and /
                continue;
            }
            i += 1;
            continue;
        }

        if state.in_string {
            if ch == b'\\' {
                state.escape = true;
                i += 1;
                continue;
            }

            if state.string_char == b'`' {
                if ch == b'`' {
                    state.in_string = false;
                } else if ch == b'$' && i + 1 < len && bytes[i + 1] == b'{' {
                    state.in_string = false;
                    state.template_expr_brace_stack.push(0);
                    i += 2;
                    continue;
                }
            } else if ch == state.string_char {
                state.in_string = false;
            }
        } else {
            // Not in string - we're in code mode
            match ch {
                b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                    // Enter block comment /*
                    state.in_block_comment = true;
                    i += 2; // Skip both / and *
                    continue;
                }
                b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                    // Line comment // -- skip rest of line
                    break;
                }
                b'\'' | b'"' => {
                    state.in_string = true;
                    state.string_char = ch;
                }
                b'`' => {
                    state.in_string = true;
                    state.string_char = b'`';
                }
                b'{' => {
                    // Track braces for template expression depth even though we're counting parens
                    if let Some(depth) = state.template_expr_brace_stack.last_mut() {
                        *depth += 1;
                    }
                }
                b'}' => {
                    if let Some(&depth) = state.template_expr_brace_stack.last() {
                        if depth == 0 {
                            state.template_expr_brace_stack.pop();
                            state.in_string = true;
                            state.string_char = b'`';
                            i += 1;
                            continue;
                        } else {
                            *state.template_expr_brace_stack.last_mut().unwrap() -= 1;
                        }
                    }
                }
                b'(' => {
                    count += 1;
                }
                b')' => {
                    count -= 1;
                }
                _ => {}
            }
        }

        i += 1;
    }

    count
}

/// Compact render body by removing unnecessary line breaks inside function calls and arrays
#[allow(dead_code)]
pub(super) fn compact_render_body(render_body: &str) -> String {
    let mut result = String::default();
    let mut chars = render_body.chars().peekable();
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut in_string = false;
    let mut string_char = '\0';
    let mut in_template = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' | '\'' if !in_template => {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                } else if string_char == ch {
                    in_string = false;
                }
                result.push(ch);
            }
            '`' => {
                in_template = !in_template;
                result.push(ch);
            }
            '(' if !in_string && !in_template => {
                paren_depth += 1;
                result.push(ch);
            }
            ')' if !in_string && !in_template => {
                paren_depth = paren_depth.saturating_sub(1);
                result.push(ch);
            }
            '[' if !in_string && !in_template => {
                bracket_depth += 1;
                result.push(ch);
            }
            ']' if !in_string && !in_template => {
                bracket_depth = bracket_depth.saturating_sub(1);
                result.push(ch);
            }
            '{' if !in_string && !in_template => {
                brace_depth += 1;
                result.push(ch);
            }
            '}' if !in_string && !in_template => {
                brace_depth = brace_depth.saturating_sub(1);
                result.push(ch);
            }
            '\n' => {
                // If inside braces (block bodies), keep newlines to preserve statement separation
                if brace_depth > 0 && !in_string && !in_template {
                    result.push('\n');
                } else if (paren_depth > 0 || bracket_depth > 0) && !in_string && !in_template {
                    result.push(' ');
                    // Skip following whitespace
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_whitespace() && next_ch != '\n' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                } else {
                    // Keep newline outside of function calls/arrays or inside strings
                    result.push(ch);
                }
            }
            _ => result.push(ch),
        }
    }

    result
}
