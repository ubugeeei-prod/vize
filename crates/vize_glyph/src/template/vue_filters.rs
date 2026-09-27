//! Vue 2 filter boundaries, following `vize_l2::expr::filter::split_filters`.
//! TODO(#6836): consume the native FilterChain payload after the formatter cutover.
//! The legacy formatter must not depend on L2 just to recognize authored pipes.

use crate::{FormatOptions, VueVersion, script};
use vize_l0::{String, ToCompactString, cstr};

pub(super) fn format_filter_expression(
    expression: &str,
    options: &FormatOptions,
    vue_version: VueVersion,
    attribute: bool,
) -> Option<String> {
    if !matches!(vue_version, VueVersion::V2 | VueVersion::V2_7) {
        return None;
    }
    let parts = split_filters(expression)?;
    Some(
        format_filter_chain(&parts, options, attribute)
            .unwrap_or_else(|| expression.trim().to_compact_string()),
    )
}

fn format_filter_chain(parts: &[&str], options: &FormatOptions, attribute: bool) -> Option<String> {
    let mut output = format_js(parts.first()?, options, attribute)?;
    // The JS printer can remove parentheses around a bitwise-OR base. Keep
    // those pipes nested so a second Vue 2 pass cannot turn them into filters.
    if split_filters(&output).is_some() {
        output = cstr!("({output})");
    }
    for segment in parts.iter().skip(1) {
        let (name, args) = match segment.split_once('(') {
            Some((name, rest)) => (name.trim(), Some(rest.strip_suffix(')')?)),
            None => (segment.trim(), None),
        };
        if !is_filter_name(name) {
            // A malformed chain remains authored text, never a guessed JS OR.
            return None;
        }
        output.push_str(" | ");
        output.push_str(name);
        if let Some(args) = args {
            // Parse only the argument payload as JavaScript. Hyphenated filter
            // names are asset names, not subtraction expressions or JS callees.
            let call = cstr!("f({args})");
            let formatted = format_js(&call, options, attribute)?;
            output.push('(');
            output.push_str(formatted.strip_prefix("f(")?.strip_suffix(')')?);
            output.push(')');
        }
    }
    Some(output)
}

fn format_js(source: &str, options: &FormatOptions, attribute: bool) -> Option<String> {
    if attribute {
        script::format_js_expression_in_attribute(source, options)
    } else {
        script::format_js_expression(source, options)
    }
}

fn is_filter_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || matches!(first, '_' | '$'))
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | '-'))
}

/// Same top-level boundary rule as VueFilterExpr: quoted, regex, nested and
/// logical-OR pipes stay inside their JS payload. Offsets are UTF-8 byte ranges.
fn split_filters(expression: &str) -> Option<Vec<&str>> {
    let bytes = expression.as_bytes();
    let mut quote = None;
    let mut regex = false;
    let (mut paren, mut square, mut curly) = (0_u32, 0_u32, 0_u32);
    let mut parts = Vec::new();
    let mut start = 0;
    let mut previous = 0;
    for (index, &byte) in bytes.iter().enumerate() {
        if let Some(delimiter) = quote {
            if byte == delimiter && previous != b'\\' {
                quote = None;
            }
        } else if regex {
            if byte == b'/' && previous != b'\\' {
                regex = false;
            }
        } else if byte == b'|'
            && bytes.get(index + 1) != Some(&b'|')
            && previous != b'|'
            && paren == 0
            && square == 0
            && curly == 0
        {
            parts.push(expression.get(start..index)?.trim());
            start = index + 1;
        } else {
            match byte {
                b'\'' | b'"' | b'`' => quote = Some(byte),
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => square += 1,
                b']' => square = square.saturating_sub(1),
                b'{' => curly += 1,
                b'}' => curly = curly.saturating_sub(1),
                b'/' => {
                    let preceding = expression.get(..index)?.chars().rev().find(|&c| c != ' ');
                    regex = preceding.is_none_or(|c| {
                        !c.is_ascii_alphanumeric()
                            && !matches!(c, '_' | ')' | '.' | '+' | '-' | '$' | ']')
                    });
                }
                _ => {}
            }
        }
        previous = byte;
    }
    if parts.is_empty() {
        return None;
    }
    parts.push(expression.get(start..)?.trim());
    Some(parts)
}
