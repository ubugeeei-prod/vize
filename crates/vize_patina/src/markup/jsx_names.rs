//! OXC JSX accessors shared by the OXC-backed facade variants.

use super::{MarkupBindingKind, MarkupElementKind};
use oxc_ast::ast::{JSXAttribute, JSXAttributeName, JSXAttributeValue, JSXElementName};

#[inline]
pub(super) fn jsx_element_kind(name: &JSXElementName<'_>) -> MarkupElementKind {
    if jsx_name_is_component(name) {
        MarkupElementKind::Component
    } else {
        MarkupElementKind::Element
    }
}

#[inline]
fn jsx_name_is_component(name: &JSXElementName<'_>) -> bool {
    match name {
        JSXElementName::Identifier(identifier) => !is_intrinsic_html_name(identifier.name.as_str()),
        JSXElementName::IdentifierReference(reference) => {
            !is_intrinsic_html_name(reference.name.as_str())
        }
        JSXElementName::NamespacedName(name) => !is_intrinsic_html_name(name.name.name.as_str()),
        JSXElementName::MemberExpression(_) | JSXElementName::ThisExpression(_) => true,
    }
}

#[inline]
fn is_intrinsic_html_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase())
}

#[inline]
pub(super) fn jsx_element_name<'a>(name: &'a JSXElementName<'a>) -> &'a str {
    match name {
        JSXElementName::Identifier(identifier) => identifier.name.as_str(),
        JSXElementName::IdentifierReference(reference) => reference.name.as_str(),
        JSXElementName::NamespacedName(name) => name.name.name.as_str(),
        JSXElementName::MemberExpression(expression) => expression.property.name.as_str(),
        JSXElementName::ThisExpression(_) => "this",
    }
}

#[inline]
pub(super) fn jsx_attribute_name<'a>(name: &'a JSXAttributeName<'a>) -> &'a str {
    match name {
        JSXAttributeName::Identifier(identifier) => identifier.name.as_str(),
        JSXAttributeName::NamespacedName(name) => name.name.name.as_str(),
    }
}

/// Whether a JSX attribute value is an expression rather than a static string.
#[inline]
pub(super) fn jsx_value_is_dynamic(value: Option<&JSXAttributeValue<'_>>) -> bool {
    matches!(
        value,
        Some(
            JSXAttributeValue::ExpressionContainer(_)
                | JSXAttributeValue::Element(_)
                | JSXAttributeValue::Fragment(_)
        )
    )
}

/// The static string value of a JSX attribute, when it has one.
#[inline]
pub(super) fn jsx_static_value<'a>(attr: &'a JSXAttribute<'a>) -> Option<&'a str> {
    match attr.value.as_ref() {
        Some(JSXAttributeValue::StringLiteral(value)) => Some(value.value.as_str()),
        _ => None,
    }
}

/// The [`MarkupBindingKind`] a JSX attribute projects to.
///
/// JSX classification is name/value-driven: `v-bind:x`, React-style `onX`,
/// expression-valued props, then plain static attributes.
#[inline]
pub(super) fn jsx_attribute_binding_kind(attr: &JSXAttribute<'_>) -> MarkupBindingKind {
    if let JSXAttributeName::NamespacedName(name) = &attr.name
        && name.namespace.name.as_str() == "v-bind"
    {
        return MarkupBindingKind::Bind;
    }

    let name = jsx_attribute_name(&attr.name);
    if is_jsx_event_handler_name(name) {
        MarkupBindingKind::On
    } else if jsx_value_is_dynamic(attr.value.as_ref()) {
        MarkupBindingKind::Bind
    } else {
        MarkupBindingKind::Attribute
    }
}

/// The directive-like kind for a JSX attribute, or `None` when the attribute is
/// a plain static attribute (and therefore not directive-like).
#[inline]
pub(super) fn jsx_attribute_directive_kind(attr: &JSXAttribute<'_>) -> Option<MarkupBindingKind> {
    match jsx_attribute_binding_kind(attr) {
        MarkupBindingKind::Attribute => None,
        kind => Some(kind),
    }
}

/// The event name carried by a JSX `onX` handler (`onClick` → `click`).
#[inline]
pub(super) fn jsx_attribute_arg_name<'a>(attr: &'a JSXAttribute<'a>) -> Option<&'a str> {
    let name = jsx_attribute_name(&attr.name);
    name.strip_prefix("on").filter(|rest| {
        rest.chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
    })
}

/// Whether a JSX attribute name is a React-style event handler (`onX`, where
/// `X` starts uppercase — so `online` is not mistaken for an event).
#[inline]
fn is_jsx_event_handler_name(name: &str) -> bool {
    name.strip_prefix("on").is_some_and(|rest| {
        rest.chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
    })
}
