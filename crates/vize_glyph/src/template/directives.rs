//! Directive normalization and expression formatting.
//!
//! Handles Vue directive shorthand normalization (`v-bind:` -> `:`, `v-on:` -> `@`,
//! `v-slot:` -> `#`) and JS expression formatting in directive values.

use crate::{options::FormatOptions, script};
use vize_s0::{String, ToCompactString};

use super::attributes::attribute_priority;
use super::helpers::template_literal_state_after_line_from;

/// Normalize directive shorthands and assign sort priority.
#[allow(clippy::disallowed_macros)]
pub(crate) fn normalize_attribute(
    name: &str,
    value: Option<String>,
    options: &FormatOptions,
) -> (String, Option<String>, u8, bool) {
    // Normalize directive shorthands (only if enabled)
    let normalized_name: String = if options.normalize_directive_shorthands {
        if let Some(rest) = name.strip_prefix("v-bind:") {
            format!(":{rest}").into()
        } else if let Some(rest) = name.strip_prefix("v-on:") {
            format!("@{rest}").into()
        } else if let Some(rest) = name.strip_prefix("v-slot:") {
            format!("#{rest}").into()
        } else {
            name.to_compact_string()
        }
    } else {
        name.to_compact_string()
    };

    // Format JS expressions in directive values
    let mut indent_multiline_value = false;
    let formatted_value = value.map(|v| {
        if should_format_expression(&normalized_name) {
            let (formatted, should_indent) = format_directive_value(&normalized_name, &v, options);
            indent_multiline_value = should_indent;
            formatted
        } else {
            v
        }
    });

    let priority = if let Some(ref groups) = options.attribute_groups {
        custom_attribute_priority(&normalized_name, groups)
    } else {
        attribute_priority(&normalized_name)
    };

    (
        normalized_name,
        formatted_value,
        priority,
        indent_multiline_value,
    )
}

/// Determine if an attribute's value should be formatted as a JS expression.
fn should_format_expression(name: &str) -> bool {
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
fn format_directive_value(name: &str, value: &str, options: &FormatOptions) -> (String, bool) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return (value.to_compact_string(), false);
    }

    // v-for has special syntax: "(item, index) in items"
    if name == "v-for" {
        return (format_v_for_expression(trimmed), false);
    }

    let decoded = decode_expression_attribute_entities(trimmed);
    let expression = decoded.as_deref().unwrap_or(trimmed);

    // Try to format as JS expression via oxc_formatter
    match script::format_js_expression(expression, options) {
        Some(formatted) => {
            let indent_multiline_value = formatted.contains('\n');
            (formatted, indent_multiline_value)
        }
        None => reanchor_continuation_lines(value, options),
    }
}

/// Re-derive the continuation indentation of a multi-line directive value the
/// expression formatter could not parse — a statement sequence such as
/// `foo(1); bar(2)`, or otherwise unparsable source.
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
fn reanchor_continuation_lines(value: &str, options: &FormatOptions) -> (String, bool) {
    let Some(first_break) = value.find('\n') else {
        return (value.to_compact_string(), false);
    };
    let (first_line, rest) = (&value[..first_break], &value[first_break + 1..]);
    if first_line.trim().is_empty() {
        return (value.to_compact_string(), false);
    }

    let common_indent = common_continuation_indent(first_line, rest);
    let indent = options.indent_string();
    let mut reanchored = String::with_capacity(value.len() + indent.len());
    reanchored.push_str(first_line);
    let mut state = ContinuationScan::new(first_line);
    for line in rest.split('\n') {
        let line = line.trim_end_matches('\r');
        if state.line_holds_code(line) {
            reanchored.push('\n');
            reanchored.push_str(&indent);
            reanchored.push_str(dedent(line, common_indent));
        } else if state.in_template_literal {
            // Raw template-literal content: these bytes belong to the rendered
            // string value, so the printer keeps them as they are.
            reanchored.push('\n');
            reanchored.push_str(line);
        }
        // Anything else is a blank line outside a template literal: it holds
        // nothing to anchor and would print as bare indentation, so it is
        // dropped, exactly as a formatted expression comes back without it.
        state.advance(line);
    }
    (reanchored, true)
}

/// Tracks whether a continuation line starts inside a multiline template
/// literal, mirroring how `write_rendered_attribute` renders the same lines.
struct ContinuationScan {
    in_template_literal: bool,
}

impl ContinuationScan {
    fn new(first_line: &str) -> Self {
        Self {
            in_template_literal: template_literal_state_after_line_from(false, first_line),
        }
    }

    /// A line whose indentation is the formatter's to choose: outside any
    /// template literal and not blank (a blank line has nothing to anchor).
    fn line_holds_code(&self, line: &str) -> bool {
        !self.in_template_literal && !line.trim().is_empty()
    }

    fn advance(&mut self, line: &str) {
        self.in_template_literal =
            template_literal_state_after_line_from(self.in_template_literal, line);
    }
}

/// The narrowest indentation shared by the continuation lines that hold code,
/// which becomes the value's zero column when they are re-anchored.
fn common_continuation_indent(first_line: &str, rest: &str) -> usize {
    let mut state = ContinuationScan::new(first_line);
    let mut common: Option<usize> = None;
    for line in rest.split('\n') {
        let line = line.trim_end_matches('\r');
        if state.line_holds_code(line) {
            let width = blank_prefix_len(line);
            common = Some(common.map_or(width, |narrowest| narrowest.min(width)));
        }
        state.advance(line);
    }
    common.unwrap_or(0)
}

fn dedent(line: &str, columns: usize) -> &str {
    &line[blank_prefix_len(line).min(columns)..]
}

fn blank_prefix_len(line: &str) -> usize {
    line.len() - line.trim_start_matches([' ', '\t']).len()
}

fn decode_expression_attribute_entities(value: &str) -> Option<String> {
    if !value.contains('&') {
        return None;
    }

    let mut decoded = String::with_capacity(value.len());
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
            decoded.push('&');
            rest = tail;
            changed = true;
        } else {
            let ch = rest
                .chars()
                .next()
                .expect("non-empty string must have a next char");
            decoded.push(ch);
            rest = &rest[ch.len_utf8()..];
        }
    }

    changed.then_some(decoded)
}

/// Format `v-for` expression: normalize spacing in `(item, index) in items`.
#[allow(clippy::disallowed_macros)]
pub(crate) fn format_v_for_expression(expr: &str) -> String {
    // Split on " in " or " of " (respecting nested parens/brackets)
    let (iterator_part, keyword, collection_part) =
        if let Some(idx) = find_v_for_keyword(expr, " in ") {
            (&expr[..idx], " in ", &expr[idx + 4..])
        } else if let Some(idx) = find_v_for_keyword(expr, " of ") {
            (&expr[..idx], " of ", &expr[idx + 4..])
        } else {
            return expr.to_compact_string();
        };

    let iter_trimmed = iterator_part.trim();
    let collection_trimmed = collection_part.trim();

    // Normalize parenthesized destructuring: "(item,index)" -> "(item, index)"
    let normalized_iter: String = if iter_trimmed.starts_with('(') && iter_trimmed.ends_with(')') {
        let inner = &iter_trimmed[1..iter_trimmed.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        format!("({})", parts.join(", ")).into()
    } else {
        iter_trimmed.to_compact_string()
    };

    format!("{normalized_iter}{keyword}{collection_trimmed}").into()
}

/// Find `keyword` in a v-for expression while respecting nested parens/brackets.
fn find_v_for_keyword(expr: &str, keyword: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let kw_bytes = keyword.as_bytes();
    let mut depth = 0i32;

    for i in 0..bytes.len() {
        match bytes[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            _ => {}
        }
        if depth == 0
            && i + kw_bytes.len() <= bytes.len()
            && &bytes[i..i + kw_bytes.len()] == kw_bytes
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
