//! Template token collection for semantic highlighting.
//!
//! Handles Vue template constructs: directives, interpolations,
//! event handlers, and v-bind shorthand.

use super::{
    encoding::{is_ident_char, offset_to_line_col, utf16_len},
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
                length: utf16_len(&template[abs_start..token_end]),
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
                length: utf16_len(&template[abs_start..token_end]),
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

    while pos < bytes.len() {
        // Look for attribute patterns
        let attr_start = if bytes[pos] == b':' || bytes[pos] == b'@' {
            // Shorthand :prop or @event
            if is_attribute_start(template, pos) {
                Some(pos)
            } else {
                None
            }
        } else if pos + 2 < bytes.len() && bytes[pos] == b'v' && bytes[pos + 1] == b'-' {
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
                let arg = &template[arg_start..arg_end];
                tokenize_expression(arg, template, arg_start, base_line, tokens);
            }

            if let Some((expr_start, expr_end)) = attribute_value(template, start) {
                let expr = &template[expr_start..expr_end];
                tokenize_expression(expr, template, expr_start, base_line, tokens);

                pos = expr_end + 1;
                continue;
            }
        }

        pos += 1;
    }
}

fn shorthand_name_end(
    template: &str,
    attr_start: usize,
    is_plain_name_char: impl Fn(char) -> bool,
    include_modifiers: bool,
) -> Option<usize> {
    let mut pos = attr_start + 1;
    if pos >= template.len() {
        return None;
    }

    if template[pos..].starts_with('[') {
        pos = find_matching_square_bracket(template, pos)? + 1;
        if include_modifiers {
            pos = consume_modifier_suffix(template, pos);
        }
        return Some(pos);
    }

    let name_start = pos;
    while pos < template.len() {
        let ch = template[pos..].chars().next()?;
        if !is_plain_name_char(ch) {
            break;
        }
        pos += ch.len_utf8();
    }

    if pos == name_start { None } else { Some(pos) }
}

fn consume_modifier_suffix(template: &str, mut pos: usize) -> usize {
    while pos < template.len() && template[pos..].starts_with('.') {
        pos += 1;
        while pos < template.len() {
            let Some(ch) = template[pos..].chars().next() else {
                break;
            };
            if !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_' {
                break;
            }
            pos += ch.len_utf8();
        }
    }
    pos
}

fn dynamic_argument_value(template: &str, attr_start: usize) -> Option<(usize, usize)> {
    let name_end = attribute_name_end(template, attr_start);
    let search = &template[attr_start..name_end];
    let bracket_offset = search.find('[')? + attr_start;
    let bracket_end = find_matching_square_bracket(template, bracket_offset)?;
    Some((bracket_offset + 1, bracket_end))
}

fn find_matching_square_bracket(template: &str, open_offset: usize) -> Option<usize> {
    if !template[open_offset..].starts_with('[') {
        return None;
    }

    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';

    for (relative, ch) in template[open_offset..].char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_offset + relative);
                }
            }
            _ => {}
        }
        prev = ch;
    }

    None
}

fn is_attribute_start(template: &str, offset: usize) -> bool {
    if offset >= template.len() || !template.is_char_boundary(offset) {
        return false;
    }

    let Some(prev) = template[..offset].chars().next_back() else {
        return false;
    };
    if !prev.is_ascii_whitespace() {
        return false;
    }

    let Some(tag_start) = template[..offset].rfind('<') else {
        return false;
    };

    let mut quote = None;
    for ch in template[tag_start + 1..offset].chars() {
        if let Some(open_quote) = quote {
            if ch == open_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '>' => return false,
            _ => {}
        }
    }

    if quote.is_some() {
        return false;
    }

    let tag_body = template[tag_start + 1..offset].trim_start();
    !tag_body.is_empty()
        && !tag_body.starts_with('/')
        && !tag_body.starts_with('!')
        && !tag_body.starts_with('?')
}

fn is_attribute_name_boundary(template: &str, offset: usize) -> bool {
    template[offset..].chars().next().is_none_or(|ch| {
        matches!(ch, '=' | ':' | '.' | '/' | '>' | '"' | '\'') || ch.is_ascii_whitespace()
    })
}

fn attribute_name_end(template: &str, start: usize) -> usize {
    let mut end = start;
    for (relative, ch) in template[start..].char_indices() {
        if ch == '=' || ch == '/' || ch == '>' || ch.is_ascii_whitespace() {
            break;
        }
        end = start + relative + ch.len_utf8();
    }
    end
}

fn attribute_value(template: &str, attr_start: usize) -> Option<(usize, usize)> {
    let mut pos = attribute_name_end(template, attr_start);

    while pos < template.len() {
        let ch = template[pos..].chars().next()?;
        if !ch.is_ascii_whitespace() {
            break;
        }
        pos += ch.len_utf8();
    }

    if template[pos..].chars().next()? != '=' {
        return None;
    }
    pos += 1;

    while pos < template.len() {
        let ch = template[pos..].chars().next()?;
        if !ch.is_ascii_whitespace() {
            break;
        }
        pos += ch.len_utf8();
    }

    let quote = template.as_bytes().get(pos).copied()?;
    if quote == b'"' || quote == b'\'' {
        let value_start = pos + 1;
        let quote_char = quote as char;
        let value_end = template[value_start..].find(quote_char)? + value_start;
        return Some((value_start, value_end));
    }

    let value_start = pos;
    while pos < template.len() {
        let ch = template[pos..].chars().next()?;
        if ch.is_ascii_whitespace()
            || ch == '>'
            || (ch == '/' && template[pos + ch.len_utf8()..].starts_with('>'))
        {
            break;
        }
        pos += ch.len_utf8();
    }

    if pos == value_start {
        None
    } else {
        Some((value_start, pos))
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
                    line: base_line + line,
                    start: col,
                    length: utf16_len(macro_name),
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
                    line: base_line + line,
                    start: col,
                    length: utf16_len(func),
                    token_type: TokenType::Function as u32,
                    modifiers: TokenModifier::encode(&[TokenModifier::DefaultLibrary]),
                });
            }

            pos = abs_pos + func.len();
        }
    }
}
