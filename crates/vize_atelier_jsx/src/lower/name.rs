//! Resolving JSX element names into Vize tag strings and element kinds.

use oxc_ast::ast::{JSXElementName, JSXMemberExpression, JSXMemberExpressionObject};
use vize_carton::String;

/// The Vize tag string for a JSX element name.
///
/// Member expressions keep their full dotted path (`Foo.Bar`), namespaced names
/// keep the `ns:name` form, and `this.X` resolves to `this.X`.
pub(crate) fn element_tag(name: &JSXElementName<'_>) -> String {
    match name {
        JSXElementName::Identifier(id) => String::from(id.name.as_str()),
        JSXElementName::IdentifierReference(reference) => String::from(reference.name.as_str()),
        JSXElementName::NamespacedName(named) => {
            let mut tag = String::from(named.namespace.name.as_str());
            tag.push(':');
            tag.push_str(named.name.name.as_str());
            tag
        }
        JSXElementName::MemberExpression(member) => member_to_string(member),
        JSXElementName::ThisExpression(_) => String::from("this"),
    }
}

/// Whether a JSX element name refers to a component rather than an intrinsic
/// (HTML/SVG) element.
///
/// Mirrors the Vue JSX convention also used by `vize_patina`: a tag whose name
/// begins with a lowercase ASCII letter is intrinsic; everything else
/// (capitalized identifiers, member expressions, `this`) is a component.
pub(crate) fn is_component(name: &JSXElementName<'_>) -> bool {
    match name {
        JSXElementName::Identifier(id) => !is_intrinsic(id.name.as_str()),
        JSXElementName::IdentifierReference(reference) => !is_intrinsic(reference.name.as_str()),
        JSXElementName::NamespacedName(named) => !is_intrinsic(named.name.name.as_str()),
        JSXElementName::MemberExpression(_) | JSXElementName::ThisExpression(_) => true,
    }
}

fn is_intrinsic(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase())
}

fn member_to_string(member: &JSXMemberExpression<'_>) -> String {
    let mut base = member_object_to_string(&member.object);
    base.push('.');
    base.push_str(member.property.name.as_str());
    base
}

fn member_object_to_string(object: &JSXMemberExpressionObject<'_>) -> String {
    match object {
        JSXMemberExpressionObject::IdentifierReference(reference) => {
            String::from(reference.name.as_str())
        }
        JSXMemberExpressionObject::MemberExpression(member) => member_to_string(member),
        JSXMemberExpressionObject::ThisExpression(_) => String::from("this"),
    }
}
