//! Template token collection for semantic highlighting.
//!
//! Handles Vue template constructs: directives, interpolations,
//! event handlers, and v-bind shorthand.

use super::{
    encoding::{is_ident_char, offset_to_line_col},
    expressions::tokenize_expression,
    types::{AbsoluteToken, TokenType},
};

/// Collect tokens from template content.
pub(crate) fn collect_template_tokens(
    template: &str,
    base_line: u32,
    tokens: &mut Vec<AbsoluteToken>,
) {
    // Find Vue directives
    collect_directive_tokens(template, base_line, tokens);

    // Find interpolations {{ expr }}
    collect_interpolation_tokens(template, base_line, tokens);

    // Find event handlers @event
    collect_event_tokens(template, base_line, tokens);

    // Find v-bind :prop
    collect_bind_tokens(template, base_line, tokens);

    // Find directive attribute expressions (v-bind="expr", v-if="expr", :prop="expr", @click="expr")
    collect_directive_expression_tokens(template, base_line, tokens);
}

/// Collect directive tokens (v-if, v-for, v-model, etc.)
fn collect_directive_tokens(template: &str, base_line: u32, tokens: &mut Vec<AbsoluteToken>) {
    let directives = [
        "v-if",
        "v-else-if",
        "v-else",
        "v-for",
        "v-show",
        "v-model",
        "v-bind",
        "v-on",
        "v-slot",
        "v-pre",
        "v-once",
        "v-memo",
        "v-cloak",
    ];

    for directive in directives {
        let mut pos = 0;
        while let Some(found) = template[pos..].find(directive) {
            let abs_pos = pos + found;
            let (line, col) = offset_to_line_col(template, abs_pos);

            tokens.push(AbsoluteToken {
                line: base_line + line - 1,
                start: col,
                length: directive.len() as u32,
                token_type: TokenType::Keyword as u32,
                modifiers: 0,
            });

            pos = abs_pos + directive.len();
        }
    }
}

/// Collect interpolation tokens {{ expr }}.
pub(crate) fn collect_interpolation_tokens(
    template: &str,
    base_line: u32,
    tokens: &mut Vec<AbsoluteToken>,
) {
    let mut pos = 0;
    while let Some(start) = template[pos..].find("{{") {
        let abs_start = pos + start;
        if let Some(end) = template[abs_start..].find("}}") {
            let expr_start = abs_start + 2;
            let expr_end = abs_start + end;
            let expr = &template[expr_start..expr_end];

            // Tokenize the entire expression
            tokenize_expression(expr, template, expr_start, base_line, tokens);

            pos = abs_start + end + 2;
        } else {
            break;
        }
    }
}

/// Collect event handler tokens (@click, @input, etc.)
fn collect_event_tokens(template: &str, base_line: u32, tokens: &mut Vec<AbsoluteToken>) {
    let mut pos = 0;
    while let Some(start) = template[pos..].find('@') {
        let abs_start = pos + start;
        let remaining = &template[abs_start + 1..];

        // Find the event name
        let event_end = remaining
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != ':' && c != '.')
            .unwrap_or(remaining.len());

        if event_end > 0 {
            let (line, col) = offset_to_line_col(template, abs_start);

            tokens.push(AbsoluteToken {
                line: base_line + line - 1,
                start: col,
                length: (event_end + 1) as u32, // +1 for @
                token_type: TokenType::Event as u32,
                modifiers: 0,
            });
        }

        pos = abs_start + 1;
    }
}

/// Collect v-bind tokens (:prop, :class, etc.)
fn collect_bind_tokens(template: &str, base_line: u32, tokens: &mut Vec<AbsoluteToken>) {
    // Find :prop patterns (but not ::)
    let mut pos = 0;
    while let Some(start) = template[pos..].find(':') {
        let abs_start = pos + start;

        // Skip :: (CSS pseudo-elements)
        if abs_start + 1 < template.len() && template.as_bytes()[abs_start + 1] == b':' {
            pos = abs_start + 2;
            continue;
        }

        // Check if it's in an attribute context (after a space or tag name)
        if abs_start > 0 {
            let before = template.as_bytes()[abs_start - 1];
            if before == b' ' || before == b'\n' || before == b'\t' {
                let remaining = &template[abs_start + 1..];
                let prop_end = remaining
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                    .unwrap_or(remaining.len());

                if prop_end > 0 {
                    let (line, col) = offset_to_line_col(template, abs_start);

                    tokens.push(AbsoluteToken {
                        line: base_line + line - 1,
                        start: col,
                        length: (prop_end + 1) as u32, // +1 for :
                        token_type: TokenType::Property as u32,
                        modifiers: 0,
                    });
                }
            }
        }

        pos = abs_start + 1;
    }
}

/// Collect tokens from directive expressions (v-bind="expr", v-if="expr", :prop="expr", @click="expr")
pub(crate) fn collect_directive_expression_tokens(
    template: &str,
    base_line: u32,
    tokens: &mut Vec<AbsoluteToken>,
) {
    let bytes = template.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() {
        // Look for attribute patterns
        let attr_start = if bytes[pos] == b':' || bytes[pos] == b'@' {
            // Shorthand :prop or @event
            if pos > 0
                && (bytes[pos - 1] == b' ' || bytes[pos - 1] == b'\n' || bytes[pos - 1] == b'\t')
            {
                Some(pos)
            } else {
                None
            }
        } else if pos + 2 < bytes.len() && bytes[pos] == b'v' && bytes[pos + 1] == b'-' {
            // v-* directive
            if pos == 0
                || bytes[pos - 1] == b' '
                || bytes[pos - 1] == b'\n'
                || bytes[pos - 1] == b'\t'
            {
                Some(pos)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(start) = attr_start {
            // Find the = and the quoted value
            let remaining = &template[start..];
            if let Some(eq_pos) = remaining.find('=') {
                let after_eq = &remaining[eq_pos + 1..];
                let after_eq_trimmed = after_eq.trim_start();
                let skip_ws = after_eq.len() - after_eq_trimmed.len();

                // Check for quote
                if !after_eq_trimmed.is_empty() {
                    let quote = after_eq_trimmed.as_bytes()[0];
                    if quote == b'"' || quote == b'\'' {
                        // Find closing quote
                        let expr_start = eq_pos + 1 + skip_ws + 1;
                        if let Some(end) = remaining[expr_start..].find(quote as char) {
                            let expr = &remaining[expr_start..expr_start + end];

                            // Tokenize the entire expression
                            tokenize_expression(
                                expr,
                                template,
                                start + expr_start,
                                base_line,
                                tokens,
                            );

                            pos = start + expr_start + end + 1;
                            continue;
                        }
                    }
                }
            }
        }

        pos += 1;
    }
}

/// Collect tokens from script content (compiler macros and Vue functions).
pub(crate) fn collect_script_tokens(script: &str, base_line: u32, tokens: &mut Vec<AbsoluteToken>) {
    use super::types::TokenModifier;

    // Vue compiler macros (special highlighting)
    let compiler_macros = [
        "defineProps",
        "defineEmits",
        "defineExpose",
        "defineModel",
        "defineOptions",
        "defineSlots",
        "withDefaults",
    ];

    // Vue composition API functions
    let vue_functions = [
        "ref",
        "reactive",
        "computed",
        "watch",
        "watchEffect",
        "onMounted",
        "onUnmounted",
        "onBeforeMount",
        "onBeforeUnmount",
        "onUpdated",
        "onBeforeUpdate",
        "provide",
        "inject",
    ];

    // Highlight compiler macros with Macro token type
    for macro_name in compiler_macros {
        #[allow(clippy::disallowed_macros)]
        let pattern = format!("{}(", macro_name);
        let mut pos = 0;
        while let Some(found) = script[pos..].find(pattern.as_str()) {
            let abs_pos = pos + found;

            // Check word boundary
            let is_start = abs_pos == 0 || !is_ident_char(script.as_bytes()[abs_pos - 1] as char);

            if is_start {
                let (line, col) = offset_to_line_col(script, abs_pos);

                tokens.push(AbsoluteToken {
                    line: base_line + line - 1,
                    start: col,
                    length: macro_name.len() as u32,
                    token_type: TokenType::Macro as u32,
                    modifiers: TokenModifier::encode(&[TokenModifier::DefaultLibrary]),
                });
            }

            pos = abs_pos + macro_name.len();
        }
    }

    // Highlight Vue functions with Function token type
    for func in vue_functions {
        #[allow(clippy::disallowed_macros)]
        let pattern = format!("{}(", func);
        let mut pos = 0;
        while let Some(found) = script[pos..].find(pattern.as_str()) {
            let abs_pos = pos + found;

            // Check word boundary
            let is_start = abs_pos == 0 || !is_ident_char(script.as_bytes()[abs_pos - 1] as char);

            if is_start {
                let (line, col) = offset_to_line_col(script, abs_pos);

                tokens.push(AbsoluteToken {
                    line: base_line + line - 1,
                    start: col,
                    length: func.len() as u32,
                    token_type: TokenType::Function as u32,
                    modifiers: TokenModifier::encode(&[TokenModifier::DefaultLibrary]),
                });
            }

            pos = abs_pos + func.len();
        }
    }
}
