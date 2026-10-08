//! Directive normalization and expression formatting.
//!
//! Handles Vue directive shorthand normalization (`v-bind:` -> `:`, `v-on:` -> `@`,
//! `v-slot:` -> `#`) and JS expression formatting in directive values.

use crate::{options::FormatOptions, script};
use vize_l0::{SmallVec, String, ToCompactString, cstr};

use super::literal_lines::{LiteralLineState, Representation, encode_reference_data};

mod normalize;
pub(crate) use normalize::normalize_attribute_with_vue_version;

/// Determine if an attribute's value should be formatted as a JS expression.
pub(crate) fn should_format_expression(name: &str) -> bool {
    name.starts_with(':')
        || name.starts_with('@')
        || name.starts_with("v-if")
        || name.starts_with("v-else-if")
        || name.starts_with("v-show")
        || name.starts_with("v-for")
        || name.starts_with("v-model")
        || name.starts_with("v-bind")
        || name.starts_with("v-on")
        || name == "v-html"
        || name == "v-text"
}

/// Format a directive value expression.
pub(super) fn format_directive_value(
    name: &str,
    value: &str,
    options: &FormatOptions,
    vue_version: crate::VueVersion,
) -> (String, bool) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return (value.to_compact_string(), false);
    }

    // Formatting a leading line comment as a standalone JS expression moves
    // it onto the opening quote and may rewrap the expression beneath it.
    // Keep the authored tokens and quote placement, but rederive same-line
    // continuations before the attribute and SFC printers add their depth.
    // Values starting on the following line remain verbatim (#6694).
    if value.contains('\n') && trimmed.starts_with("//") {
        return reanchor_continuation_lines(value, options, Representation::Html);
    }

    // v-for has special syntax: "(item, index) in items"
    if name == "v-for" {
        return (format_v_for_expression(trimmed), false);
    }

    let decoded = decode_expression_attribute_entities(trimmed);
    let expression = decoded
        .as_ref()
        .map_or(trimmed, |value| value.code.as_str());

    // Vue 2 filters apply to interpolations and bound values, never events.
    if (name.starts_with(':') || name.starts_with("v-bind"))
        && let Some(formatted) =
            super::vue_filters::format_filter_expression(expression, options, vue_version, true)
    {
        let multiline = formatted.contains('\n');
        return (formatted, multiline);
    }

    // Try to format as JS expression via oxc_formatter
    match script::format_js_expression_in_attribute_with_layout(expression, options) {
        Some(formatted) if formatted.retained_bare_sequence => {
            // These authored bytes retain absolute source indentation. Rebase
            // code continuations before the attribute printer adds its depth.
            // The bare tag retains the exact decoded source bytes. Protect only
            // amp DATA produced by decoding before reanchor changes byte offsets.
            let code = encode_reference_data(
                formatted.code,
                decoded
                    .as_ref()
                    .map_or(&[], |value| value.amp_positions.as_slice()),
            );
            reanchor_continuation_lines(&code, options, Representation::JavaScript)
        }
        Some(formatted) => {
            let indent_multiline_value = formatted.code.contains('\n');
            (formatted.code, indent_multiline_value)
        }
        None => reanchor_continuation_lines(value, options, Representation::Html),
    }
}

/// Re-derive the continuation indentation of a multi-line directive value the
/// expression formatter could not parse, or retained as authored bare sequence
/// bytes — a statement sequence such as `foo(1); bar(2)`, or comma expressions.
///
/// The value's own bytes are kept, but the leading whitespace of every
/// continuation line is rebuilt from the value's common indentation, so the
/// printed line is `attribute indent + one level + relative depth` whatever the
/// previous pass wrote. `write_rendered_attribute` then anchors those lines to
/// the attribute's depth, exactly like the lines a formatted expression
/// produces. Without the rebuild the value carries its absolute indentation and
/// the SFC indent step adds one more level on top of it, so the line drifts two
/// columns further on every `vize fmt` run (#3346).
///
/// A value whose first line is blank starts on the line *after* the attribute
/// name. `compute_raw_line_mask` keeps every line of that shape verbatim, so it
/// never receives SFC indentation and must not be re-anchored here.
pub(super) fn reanchor_continuation_lines(
    value: &str,
    options: &FormatOptions,
    representation: Representation,
) -> (String, bool) {
    let Some(first_break) = value.find('\n') else {
        return (value.to_compact_string(), false);
    };
    let (first_line, rest) = (
        value.get(..first_break).unwrap_or_default(),
        value.get(first_break + 1..).unwrap_or_default(),
    );
    if first_line.trim().is_empty() {
        return (value.to_compact_string(), false);
    }

    let common_indent = common_continuation_indent(first_line, rest, representation);
    let indent = options.indent_string();
    let mut reanchored = String::with_capacity(value.len() + indent.len());
    reanchored.push_str(first_line);
    let mut state = LiteralLineState::from_line(first_line, representation);
    for line in rest.split('\n') {
        let line = line.trim_end_matches('\r');
        if state.line_holds_code(line) {
            reanchored.push('\n');
            reanchored.push_str(&indent);
            reanchored.push_str(dedent(line, common_indent));
        } else if state.line_is_raw() {
            // Literal or escaped quoted-string continuation: these bytes
            // belong to the runtime value, so both printers keep them raw.
            reanchored.push('\n');
            reanchored.push_str(line);
        }
        // Anything else is a blank line outside a template literal: it holds
        // nothing to anchor and would print as bare indentation, so it is
        // dropped, exactly as a formatted expression comes back without it.
        state.advance_line(line);
    }
    (reanchored, true)
}

/// The narrowest indentation shared by the continuation lines that hold code,
/// which becomes the value's zero column when they are re-anchored.
fn common_continuation_indent(
    first_line: &str,
    rest: &str,
    representation: Representation,
) -> usize {
    let mut state = LiteralLineState::from_line(first_line, representation);
    let mut common: Option<usize> = None;
    for line in rest.split('\n') {
        let line = line.trim_end_matches('\r');
        if state.line_holds_code(line) {
            let width = blank_prefix_len(line);
            common = Some(common.map_or(width, |narrowest| narrowest.min(width)));
        }
        state.advance_line(line);
    }
    common.unwrap_or(0)
}

fn dedent(line: &str, columns: usize) -> &str {
    line.get(blank_prefix_len(line).min(columns)..)
        .unwrap_or_default()
}

fn blank_prefix_len(line: &str) -> usize {
    line.len() - line.trim_start_matches([' ', '\t']).len()
}

struct DecodedExpression {
    code: String,
    amp_positions: SmallVec<[usize; 2]>,
}

fn decode_expression_attribute_entities(value: &str) -> Option<DecodedExpression> {
    if !value.contains('&') {
        return None;
    }

    let mut decoded = String::with_capacity(value.len());
    let mut amp_positions = SmallVec::new();
    let mut changed = false;
    let mut rest = value;
    while !rest.is_empty() {
        if let Some(tail) = rest.strip_prefix("&quot;") {
            decoded.push('"');
            rest = tail;
            changed = true;
        } else if let Some(tail) = rest
            .strip_prefix("&#34;")
            .or_else(|| rest.strip_prefix("&#x22;"))
            .or_else(|| rest.strip_prefix("&#X22;"))
        {
            decoded.push('"');
            rest = tail;
            changed = true;
        } else if let Some(tail) = rest.strip_prefix("&apos;") {
            decoded.push('\'');
            rest = tail;
            changed = true;
        } else if let Some(tail) = rest
            .strip_prefix("&#39;")
            .or_else(|| rest.strip_prefix("&#x27;"))
            .or_else(|| rest.strip_prefix("&#X27;"))
        {
            decoded.push('\'');
            rest = tail;
            changed = true;
        } else if let Some(tail) = rest.strip_prefix("&amp;") {
            amp_positions.push(decoded.len());
            decoded.push('&');
            rest = tail;
            changed = true;
        } else {
            let Some(ch) = rest.chars().next() else {
                break;
            };
            decoded.push(ch);
            rest = rest.get(ch.len_utf8()..).unwrap_or_default();
        }
    }

    changed.then_some(DecodedExpression {
        code: decoded,
        amp_positions,
    })
}

/// Format `v-for` expression: normalize spacing in `(item, index) in items`.
pub(crate) fn format_v_for_expression(expr: &str) -> String {
    // Split on " in " or " of " (respecting nested parens/brackets)
    let (iterator_part, keyword, collection_part) =
        if let Some(idx) = find_v_for_keyword(expr, " in ") {
            (
                expr.get(..idx).unwrap_or_default(),
                " in ",
                expr.get(idx + 4..).unwrap_or_default(),
            )
        } else if let Some(idx) = find_v_for_keyword(expr, " of ") {
            (
                expr.get(..idx).unwrap_or_default(),
                " of ",
                expr.get(idx + 4..).unwrap_or_default(),
            )
        } else {
            return expr.to_compact_string();
        };

    let iter_trimmed = iterator_part.trim();
    let collection_trimmed = collection_part.trim();

    // Normalize parenthesized destructuring: "(item,index)" -> "(item, index)"
    let normalized_iter: String = if iter_trimmed.starts_with('(') && iter_trimmed.ends_with(')') {
        let inner = iter_trimmed
            .get(1..iter_trimmed.len() - 1)
            .unwrap_or_default();
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        cstr!("({})", parts.join(", "))
    } else {
        iter_trimmed.to_compact_string()
    };

    cstr!("{normalized_iter}{keyword}{collection_trimmed}")
}

/// Find `keyword` in a v-for expression while respecting nested parens/brackets.
fn find_v_for_keyword(expr: &str, keyword: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let kw_bytes = keyword.as_bytes();
    let mut depth = 0i32;

    for (i, &byte) in bytes.iter().enumerate() {
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            _ => {}
        }
        if depth == 0
            && i + kw_bytes.len() <= bytes.len()
            && bytes.get(i..i + kw_bytes.len()).unwrap_or_default() == kw_bytes
        {
            return Some(i);
        }
    }
    None
}

/// Determine attribute priority based on custom attribute groups.
///
/// Each group in `groups` is a list of patterns. Groups are matched in order (index = priority).
/// Patterns: exact name (`id`), prefix glob (`v-*`, `:*`, `@*`), or `*` catch-all.
/// Unmatched attributes get priority `groups.len()` (last).
pub(crate) fn custom_attribute_priority(name: &str, groups: &[Vec<String>]) -> u8 {
    for (i, group) in groups.iter().enumerate() {
        for pattern in group {
            if matches_attr_pattern(name, pattern) {
                return i as u8;
            }
        }
    }
    groups.len() as u8
}

/// Match an attribute name against a pattern.
///
/// - `*` matches everything
/// - `prefix*` matches names starting with `prefix`
/// - exact string matches the name exactly
pub(crate) fn matches_attr_pattern(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return name.starts_with(prefix);
    }
    name == pattern
}
