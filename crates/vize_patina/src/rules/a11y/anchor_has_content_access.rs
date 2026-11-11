//! Which `aria-hidden` values and interpolations still expose anchor content.

use crate::markup::{MarkupBindingKind, MarkupElement};
use crate::rules::a11y::helpers::{is_aria_hidden_true, string_literal_value};
use crate::rules::a11y::markup_helpers;
use vize_relief::{ElementNode, ExpressionNode, PropNode};

pub(super) fn expression_has_content(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || matches!(value, "null" | "undefined") {
        return false;
    }
    string_literal_value(value).is_none_or(|literal| !literal.is_empty())
}

pub(super) fn interpolation_source_has_content(source: &str, start: u32, end: u32) -> bool {
    let Some(authored) = source.get(start as usize..end as usize) else {
        return true;
    };
    let authored = authored.trim();
    let inner = authored
        .strip_prefix("{{")
        .and_then(|rest| rest.strip_suffix("}}"))
        .map(str::trim)
        .unwrap_or(authored);
    expression_has_content(inner)
}

fn expression_is_false(value: &str) -> bool {
    let value = value.trim();
    value == "false"
        || string_literal_value(value).is_some_and(|literal| literal.eq_ignore_ascii_case("false"))
}

pub(super) fn relief_hides_content(element: &ElementNode) -> bool {
    if is_aria_hidden_true(element) {
        return true;
    }
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) if attr.name == "aria-hidden" => attr
            .value
            .as_ref()
            .is_none_or(|value| !value.content.eq_ignore_ascii_case("false")),
        PropNode::Directive(dir) if dir.name == "bind" => {
            let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
                return false;
            };
            if !arg.is_static || arg.content != "aria-hidden" {
                return false;
            }
            dir.exp.as_ref().is_none_or(|expression| match expression {
                ExpressionNode::Simple(simple) => !expression_is_false(simple.content),
                ExpressionNode::Compound(_) => true,
            })
        }
        _ => false,
    })
}

pub(super) fn markup_hides_content(element: &MarkupElement<'_>) -> bool {
    if markup_helpers::is_aria_hidden_true_markup(element) {
        return true;
    }
    let mut hidden = false;
    element.walk_bindings(&mut |binding| {
        let aria = match binding.kind() {
            MarkupBindingKind::Attribute => binding.is_unqualified_arg_exact("aria-hidden"),
            MarkupBindingKind::Bind => binding.is_static_unqualified_arg_exact("aria-hidden"),
            _ => false,
        };
        if !aria {
            return;
        }
        if binding
            .static_value()
            .is_some_and(|value| value.eq_ignore_ascii_case("false"))
        {
            return;
        }
        if binding.expression().is_some_and(expression_is_false) {
            return;
        }
        hidden = true;
    });
    hidden
}
