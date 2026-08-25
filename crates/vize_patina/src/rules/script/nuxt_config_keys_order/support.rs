use std::cmp::Ordering;

use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey};
use oxc_span::{GetSpan, Span};
use vize_carton::{String, ToCompactString};

// Exact @nuxt/eslint-plugin 1.16.0 order. The differential corpus reverses
// every entry, making this complete list an executable upstream pin.
const ORDER_KEYS: &[&str] = &[
    "appId",
    "buildId",
    "extends",
    "theme",
    "modules",
    "plugins",
    "$",
    "ssr",
    "pages",
    "components",
    "imports",
    "devtools",
    "app",
    "css",
    "vue",
    "router",
    "unhead",
    "site",
    "colorMode",
    "content",
    "mdc",
    "ui",
    "spaLoadingTemplate",
    "appConfig",
    "runtimeConfig",
    "dir",
    "rootDir",
    "srcDir",
    "appDir",
    "workspaceDir",
    "serverDir",
    "buildDir",
    "modulesDir",
    "analyzeDir",
    "alias",
    "extensions",
    "ignore",
    "ignoreOptions",
    "ignorePrefix",
    "builder",
    "build",
    "generate",
    "routeRules",
    "sourcemap",
    "optimization",
    "dev",
    "devServer",
    "watch",
    "watchers",
    "future",
    "features",
    "experimental",
    "compatibilityDate",
    "nitro",
    "hub",
    "serverHandlers",
    "devServerHandlers",
    "vite",
    "webpack",
    "typescript",
    "postcss",
    "test",
    "telemetry",
    "debug",
    "logLevel",
    "hooks",
];

pub(super) fn sort_named_segments(reordered: &mut [usize], names: &[Option<String>]) {
    let mut start = None;
    for index in 0..names.len() {
        if names[index].is_some() {
            start.get_or_insert(index);
        } else if let Some(segment_start) = start.take() {
            sort_segment(&mut reordered[segment_start..index], names);
        }
    }
    if let Some(segment_start) = start
        && segment_start + 1 < names.len()
    {
        sort_segment(&mut reordered[segment_start..], names);
    }
}

/// Return the first adjacent pair whose authored order contradicts the Nuxt
/// comparator. Unnamed properties (notably spreads) remain sorting boundaries,
/// matching `sort_named_segments` and the fix that the diagnostic accompanies.
pub(super) fn first_order_inversion(names: &[Option<String>]) -> Option<(usize, usize)> {
    let mut previous: Option<usize> = None;
    for (index, name) in names.iter().enumerate() {
        let Some(name) = name.as_deref() else {
            previous = None;
            continue;
        };
        if let Some(previous_index) = previous {
            let previous_name = names[previous_index].as_deref()?;
            if compare_names(previous_name, name).is_gt() {
                return Some((previous_index, index));
            }
        }
        previous = Some(index);
    }
    None
}

fn sort_segment(segment: &mut [usize], names: &[Option<String>]) {
    segment.sort_by(|left, right| {
        compare_names(
            names[*left].as_deref().unwrap(),
            names[*right].as_deref().unwrap(),
        )
    });
}

fn compare_names(left: &str, right: &str) -> Ordering {
    match (order_index(left), order_index(right)) {
        (Some(left_index), Some(right_index)) => left_index
            .cmp(&right_index)
            .then_with(|| locale_compare(left, right)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => locale_compare(left, right),
    }
}

fn order_index(name: &str) -> Option<usize> {
    if name.starts_with('$') {
        return ORDER_KEYS.iter().position(|key| *key == "$");
    }
    ORDER_KEYS.iter().position(|key| *key == name)
}

// The upstream comparator is localeCompare. Nuxt config keys are ASCII; this
// reproduces its observable punctuation-insensitive, lowercase-first order.
fn locale_compare(left: &str, right: &str) -> Ordering {
    let prefix = |value: &str| match value.as_bytes().first() {
        Some(b'\'') => 0,
        Some(b'"') => 1,
        _ => 2,
    };
    let key = |value: &str| {
        value
            .chars()
            .filter(|character| character.is_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };
    prefix(left)
        .cmp(&prefix(right))
        .then_with(|| key(left).cmp(&key(right)))
        .then_with(|| {
            left.chars()
                .zip(right.chars())
                .find_map(|(left, right)| {
                    (left.eq_ignore_ascii_case(&right) && left != right).then(|| {
                        match (left.is_ascii_lowercase(), right.is_ascii_lowercase()) {
                            (true, false) => Ordering::Less,
                            (false, true) => Ordering::Greater,
                            _ => left.cmp(&right),
                        }
                    })
                })
                .unwrap_or_else(|| left.cmp(right))
        })
}

pub(super) fn property_name(property: &ObjectPropertyKind<'_>, source: &str) -> Option<String> {
    let ObjectPropertyKind::ObjectProperty(property) = property else {
        return None;
    };
    key_name(&property.key, source)
}

pub(super) fn property_display_name(
    property: &ObjectPropertyKind<'_>,
    source: &str,
) -> Option<String> {
    let ObjectPropertyKind::ObjectProperty(property) = property else {
        return None;
    };
    display_key_name(&property.key, source)
}

fn display_key_name(key: &PropertyKey<'_>, source: &str) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.to_compact_string()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.to_compact_string()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.to_compact_string()),
        PropertyKey::ParenthesizedExpression(parenthesized) => {
            display_expression_key_name(&parenthesized.expression, source)
        }
        _ => Some(span_text(key.span(), source)),
    }
}

fn display_expression_key_name(expression: &Expression<'_>, source: &str) -> Option<String> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_compact_string()),
        Expression::StringLiteral(literal) => Some(literal.value.to_compact_string()),
        Expression::ParenthesizedExpression(parenthesized) => {
            display_expression_key_name(&parenthesized.expression, source)
        }
        _ => Some(span_text(expression.span(), source)),
    }
}

fn key_name(key: &PropertyKey<'_>, source: &str) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.to_compact_string()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.to_compact_string()),
        PropertyKey::StringLiteral(_)
        | PropertyKey::NumericLiteral(_)
        | PropertyKey::BigIntLiteral(_)
        | PropertyKey::BooleanLiteral(_)
        | PropertyKey::NullLiteral(_)
        | PropertyKey::RegExpLiteral(_) => Some(span_text(key.span(), source)),
        PropertyKey::ParenthesizedExpression(parenthesized) => {
            expression_key_name(&parenthesized.expression, source)
        }
        _ => None,
    }
}

fn expression_key_name(expression: &Expression<'_>, source: &str) -> Option<String> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_compact_string()),
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::RegExpLiteral(_) => Some(span_text(expression.span(), source)),
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_key_name(&parenthesized.expression, source)
        }
        _ => None,
    }
}

fn span_text(span: Span, source: &str) -> String {
    source[span.start as usize..span.end as usize].to_compact_string()
}

pub(super) fn property_text_ranges(
    object: &ObjectExpression<'_>,
    source: &str,
) -> (usize, usize, Vec<String>) {
    let first_start = object.properties[0].span().start as usize;
    let line_start = source[..first_start]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let range_start = (object.span.start as usize + 1).max(line_start);
    let close_brace = object.span.end as usize - 1;
    let mut range_end = range_start;
    let mut pieces = Vec::with_capacity(object.properties.len());

    for property in &object.properties {
        let property_end = property.span().end as usize;
        let next_token = next_token_offset(source, property_end, close_brace);
        let has_comma = source.as_bytes().get(next_token) == Some(&b',');
        let mut last_range = if has_comma {
            next_token + 1
        } else {
            property_end
        };
        let mut text = source[range_end..last_range].to_compact_string();
        if !has_comma && next_token == close_brace {
            text.push(',');
        }
        if source.as_bytes().get(last_range) == Some(&b'\n') {
            last_range += 1;
            text.push('\n');
        }
        pieces.push(text);
        range_end = last_range;
    }
    (range_start, range_end, pieces)
}

fn next_token_offset(source: &str, mut offset: usize, limit: usize) -> usize {
    while offset < limit {
        let tail = &source[offset..];
        let character = tail.chars().next().unwrap();
        if character.is_whitespace() {
            offset += character.len_utf8();
        } else if tail.starts_with("//") {
            offset = tail.find('\n').map_or(limit, |index| offset + index + 1);
        } else if tail.starts_with("/*") {
            offset = tail.find("*/").map_or(limit, |index| offset + index + 2);
        } else {
            break;
        }
    }
    offset
}
