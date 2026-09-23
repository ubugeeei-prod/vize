//! Template token collection for semantic highlighting.
//!
//! Handles Vue template constructs: directives, interpolations,
//! event handlers, and v-bind shorthand.

use super::{
    encoding::{is_ident_char, offset_to_line_col, utf16_len},
    expressions::tokenize_expression,
    template_attrs::{
        attribute_value, dynamic_argument_value, is_attribute_name_boundary, is_attribute_start,
        shorthand_name_end,
    },
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
        while let Some(found) = template.get(pos..).and_then(|rest| rest.find(directive)) {
            let abs_pos = pos + found;
            let directive_end = abs_pos + directive.len();
            let is_directive_attr = is_attribute_start(template, abs_pos)
                && is_attribute_name_boundary(template, directive_end);

            if !is_directive_attr {
                pos = directive_end;
                continue;
            }

            let (line, col) = offset_to_line_col(template, abs_pos);

            tokens.push(AbsoluteToken {
                line: base_line + line,
                start: col,
                length: utf16_len(directive),
                token_type: TokenType::Keyword as u32,
                modifiers: 0,
            });

            pos = directive_end;
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
    while let Some(start) = template.get(pos..).and_then(|rest| rest.find("{{")) {
        let abs_start = pos + start;
        if let Some(end) = template.get(abs_start..).and_then(|rest| rest.find("}}")) {
            let expr_start = abs_start + 2;
            let expr_end = abs_start + end;
            let expr = template.get(expr_start..expr_end).unwrap_or_default();

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
    while let Some(start) = template.get(pos..).and_then(|rest| rest.find('@')) {
        let abs_start = pos + start;
        if !is_attribute_start(template, abs_start) {
            pos = abs_start + 1;
            continue;
        }

        if let Some(token_end) = shorthand_name_end(
            template,
            abs_start,
            |ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == ':' || ch == '.',
            true,
        ) {
            let (line, col) = offset_to_line_col(template, abs_start);

            tokens.push(AbsoluteToken {
                line: base_line + line,
                start: col,
                length: utf16_len(template.get(abs_start..token_end).unwrap_or_default()),
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
    while let Some(start) = template.get(pos..).and_then(|rest| rest.find(':')) {
        let abs_start = pos + start;

        // Skip :: (CSS pseudo-elements)
        if template.as_bytes().get(abs_start + 1) == Some(&b':') {
            pos = abs_start + 2;
            continue;
        }

        // Check if it's in an attribute context.
        if is_attribute_start(template, abs_start)
            && let Some(token_end) = shorthand_name_end(
                template,
                abs_start,
                |ch| ch.is_ascii_alphanumeric() || ch == '-',
                false,
            )
        {
            let (line, col) = offset_to_line_col(template, abs_start);

            tokens.push(AbsoluteToken {
                line: base_line + line,
                start: col,
                length: utf16_len(template.get(abs_start..token_end).unwrap_or_default()),
                token_type: TokenType::Property as u32,
                modifiers: 0,
            });
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

    while let Some(&byte) = bytes.get(pos) {
        // Look for attribute patterns
        let attr_start = if byte == b':' || byte == b'@' {
            // Shorthand :prop or @event
            if is_attribute_start(template, pos) {
                Some(pos)
            } else {
                None
            }
        } else if pos + 2 < bytes.len() && byte == b'v' && bytes.get(pos + 1) == Some(&b'-') {
            // v-* directive
            if is_attribute_start(template, pos) {
                Some(pos)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(start) = attr_start {
            if let Some((arg_start, arg_end)) = dynamic_argument_value(template, start) {
                let arg = template.get(arg_start..arg_end).unwrap_or_default();
                tokenize_expression(arg, template, arg_start, base_line, tokens);
            }

            if let Some((expr_start, expr_end)) = attribute_value(template, start) {
                let expr = template.get(expr_start..expr_end).unwrap_or_default();
                tokenize_expression(expr, template, expr_start, base_line, tokens);

                pos = expr_end + 1;
                continue;
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
        "defineArt",
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

    let modifiers = TokenModifier::encode(&[TokenModifier::DefaultLibrary]);
    // Highlight compiler macros with Macro token type
    push_call_tokens(
        script,
        &compiler_macros,
        TokenType::Macro,
        modifiers,
        base_line,
        tokens,
    );
    // Highlight Vue functions with Function token type
    push_call_tokens(
        script,
        &vue_functions,
        TokenType::Function,
        modifiers,
        base_line,
        tokens,
    );
}

/// Push a token for every `name(` call whose name starts at a word boundary.
fn push_call_tokens(
    script: &str,
    names: &[&str],
    token_type: TokenType,
    modifiers: u32,
    base_line: u32,
    tokens: &mut Vec<AbsoluteToken>,
) {
    for name in names {
        let mut pos = 0;
        while let Some(found) = script.get(pos..).and_then(|rest| rest.find(name)) {
            let abs_pos = pos + found;
            let name_end = abs_pos + name.len();
            if !script
                .get(name_end..)
                .is_some_and(|rest| rest.starts_with('('))
            {
                pos = abs_pos + 1;
                continue;
            }

            // Check word boundary
            let is_start = abs_pos
                .checked_sub(1)
                .and_then(|prev| script.as_bytes().get(prev))
                .is_none_or(|&byte| !is_ident_char(byte as char));

            if is_start {
                let (line, col) = offset_to_line_col(script, abs_pos);

                tokens.push(AbsoluteToken {
                    line: base_line + line,
                    start: col,
                    length: utf16_len(name),
                    token_type: token_type as u32,
                    modifiers,
                });
            }

            pos = name_end;
        }
    }
}
