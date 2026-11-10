//! Attribute and child readings shared with eslint-plugin-vuejs-accessibility
//! (`getElementAttributeValue`, `isHiddenFromScreenReader`, `hasAccessibleChild`).
//!
//! Three readings follow Vue and the accessible-name rules instead of the
//! reference: whitespace-only text is no name, a bound `undefined` drops the
//! attribute, and a bound literal `false` renders `aria-hidden="false"`, so it
//! does not hide.

use crate::ir::ByteRange;
use crate::markup::{
    MarkupBinding, MarkupBindingKind, MarkupElement, MarkupElementKind, MarkupNode,
};

use super::helpers::string_literal_value;

/// A bound expression that ESTree parses as one `Literal` once its
/// parentheses are removed. Substitution-free template literals count as
/// strings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BoundLiteral<'a> {
    String(&'a str),
    Number(f64),
    BigInt { zero: bool },
    Boolean(bool),
    Null,
}

impl BoundLiteral<'_> {
    fn is_truthy(self) -> bool {
        match self {
            Self::String(value) => !value.is_empty(),
            Self::Number(value) => value != 0.0 && !value.is_nan(),
            Self::BigInt { zero } => !zero,
            Self::Boolean(value) => value,
            Self::Null => false,
        }
    }
}

pub(crate) fn strip_parentheses(expression: &str) -> &str {
    let mut expression = expression.trim();
    while let Some(inner) = expression
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
    {
        expression = inner.trim();
    }
    expression
}

pub(crate) fn bound_literal(expression: &str) -> Option<BoundLiteral<'_>> {
    let expression = strip_parentheses(expression);
    match expression {
        "true" => return Some(BoundLiteral::Boolean(true)),
        "false" => return Some(BoundLiteral::Boolean(false)),
        "null" => return Some(BoundLiteral::Null),
        _ => {}
    }
    if let Some(value) = string_literal_value(expression) {
        return Some(BoundLiteral::String(value));
    }
    numeric_literal(expression)
}

fn numeric_literal(text: &str) -> Option<BoundLiteral<'static>> {
    let first = text.bytes().next()?;
    if !first.is_ascii_digit() && first != b'.' {
        return None;
    }
    let cleaned: vize_l0::String = text.chars().filter(|character| *character != '_').collect();
    let (body, bigint) = match cleaned.strip_suffix('n') {
        Some(body) => (body, true),
        None => (cleaned.as_str(), false),
    };
    let radix = [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ]
    .into_iter()
    .find_map(|(prefix, radix)| body.strip_prefix(prefix).map(|digits| (digits, radix)));
    let value = match radix {
        Some((digits, radix)) if !digits.is_empty() => {
            digits.chars().try_fold(0.0, |value, digit| {
                digit
                    .to_digit(radix)
                    .map(|digit| value * f64::from(radix) + f64::from(digit))
            })?
        }
        Some(_) => return None,
        None if bigint => {
            if !body.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            body.parse::<f64>().ok()?
        }
        None => body.parse::<f64>().ok()?,
    };
    Some(if bigint {
        BoundLiteral::BigInt { zero: value == 0.0 }
    } else {
        BoundLiteral::Number(value)
    })
}

/// Whether a bound value names text, as `getAttributeValue` truthiness reads
/// it. `allow_empty` accepts an empty string, as `<img alt="">` does.
fn bound_has_text(binding: &MarkupBinding<'_>, allow_empty: bool) -> bool {
    if let Some(value) = binding.literal_string_value() {
        return allow_empty || !value.trim().is_empty();
    }
    let Some(expression) = binding.expression().map(strip_parentheses) else {
        return binding.has_value();
    };
    if matches!(expression, "" | "undefined") {
        return false;
    }
    match bound_literal(expression) {
        Some(BoundLiteral::String(value)) => allow_empty || !value.trim().is_empty(),
        Some(literal) => literal.is_truthy(),
        None => true,
    }
}

/// The first `name` attribute or `v-bind:name` carries text.
pub(crate) fn has_attribute_text(
    element: &MarkupElement<'_>,
    name: &str,
    allow_empty: bool,
) -> bool {
    let mut seen = false;
    let mut valid = false;
    element.walk_bindings(&mut |binding| {
        if seen
            || !matches!(
                binding.kind(),
                MarkupBindingKind::Attribute | MarkupBindingKind::Bind
            )
            || !binding.is_static_unqualified_arg_eq_ignore_ascii_case(name)
        {
            return;
        }
        seen = true;
        valid = match binding.kind() {
            MarkupBindingKind::Attribute => binding
                .static_value()
                .is_some_and(|value| allow_empty || !value.trim().is_empty()),
            _ => bound_has_text(&binding, allow_empty),
        };
    });
    valid
}

pub(crate) fn has_aria_label(element: &MarkupElement<'_>) -> bool {
    has_attribute_text(element, "aria-label", false)
        || has_attribute_text(element, "aria-labelledby", false)
}

/// Any `aria-hidden` other than `"false"` hides the element, valueless included.
pub(crate) fn is_hidden_from_screen_reader(element: &MarkupElement<'_>) -> bool {
    let mut seen = false;
    let mut hidden = false;
    element.walk_bindings(&mut |binding| {
        if seen
            || !matches!(
                binding.kind(),
                MarkupBindingKind::Attribute | MarkupBindingKind::Bind
            )
            || !binding.is_static_unqualified_arg_eq_ignore_ascii_case("aria-hidden")
        {
            return;
        }
        seen = true;
        let visible = binding.literal_string_value() == Some("false")
            || binding.kind() == MarkupBindingKind::Bind
                && binding
                    .expression()
                    .and_then(bound_literal)
                    .is_some_and(|literal| {
                        matches!(
                            literal,
                            BoundLiteral::Boolean(false) | BoundLiteral::String("false")
                        )
                    });
        hidden = !visible;
    });
    hidden
}

/// `{{ undefined }}` renders nothing; any other interpolation may render text.
pub(crate) fn interpolation_has_value(source: &str, range: ByteRange) -> bool {
    let Some(authored) = source.get(range.start as usize..range.end as usize) else {
        return true;
    };
    let expression = authored
        .strip_prefix("{{")
        .and_then(|value| value.strip_suffix("}}"))
        .or_else(|| {
            authored
                .strip_prefix('{')
                .and_then(|value| value.strip_suffix('}'))
        })
        .unwrap_or(authored);
    strip_parentheses(expression) != "undefined"
}

pub(crate) fn is_slot_outlet(element: &MarkupElement<'_>) -> bool {
    element.kind() == MarkupElementKind::Slot || element.is_unqualified_tag_exact("slot")
}

/// Text, a value-bearing interpolation, a slot outlet, or such content under a
/// child that is not hidden from screen readers.
pub(crate) fn has_accessible_child(source: &str, element: &MarkupElement<'_>) -> bool {
    let mut found = false;
    element.walk_children(&mut |child| {
        if found {
            return;
        }
        found = match child {
            MarkupNode::Text(text) => text.is_significant(),
            MarkupNode::Interpolation(range) => interpolation_has_value(source, range),
            MarkupNode::Element(child) => {
                is_slot_outlet(&child)
                    || !is_hidden_from_screen_reader(&child) && has_accessible_child(source, &child)
            }
            MarkupNode::If(_) | MarkupNode::For(_) => true,
            MarkupNode::Comment(_) | MarkupNode::Other(_) => false,
        };
    });
    found
}

#[cfg(test)]
mod tests {
    use super::{BoundLiteral, bound_literal, strip_parentheses};

    #[test]
    fn bound_literals_follow_estree_literal_values() {
        assert_eq!(bound_literal("(2)"), Some(BoundLiteral::Number(2.0)));
        assert_eq!(bound_literal("1_000"), Some(BoundLiteral::Number(1000.0)));
        assert_eq!(bound_literal("0x1F"), Some(BoundLiteral::Number(31.0)));
        assert_eq!(bound_literal(".5"), Some(BoundLiteral::Number(0.5)));
        assert_eq!(
            bound_literal("0n"),
            Some(BoundLiteral::BigInt { zero: true })
        );
        assert_eq!(bound_literal("'a'"), Some(BoundLiteral::String("a")));
        assert_eq!(
            bound_literal("( false )"),
            Some(BoundLiteral::Boolean(false))
        );
        assert_eq!(bound_literal("null"), Some(BoundLiteral::Null));
        for expression in [
            "x",
            "-1",
            "+1",
            "undefined",
            "Infinity",
            "1e",
            "0x",
            "`${x}`",
            "a + b",
        ] {
            assert_eq!(bound_literal(expression), None, "{expression}");
        }
        assert_eq!(strip_parentheses(" ((undefined)) "), "undefined");
    }
}
