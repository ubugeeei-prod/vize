//! Resolving JSX element names into Vize tag strings and element kinds.

use oxc_ast::ast::JSXElementName;
use oxc_span::Span;
use vize_s0::String;

/// The XML namespace prefixes a JSX tag may legitimately carry.
///
/// `<svg:circle/>` and `<math:mi/>` are the qualified spellings of the only two
/// foreign-content namespaces the HTML parser knows, and Vize keeps them
/// verbatim. Every other prefix names a namespace nothing downstream resolves:
/// `@vue/babel-plugin-jsx` rejects namespaced tags outright with
/// `getTag: JSXNamespacedName is not supported`.
const KNOWN_TAG_NAMESPACES: [&str; 2] = ["svg", "math"];

/// The Vize tag string for a JSX element name. Namespaced names keep the
/// `ns:name` form.
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
        // Panic path by lowering invariant: `lower_element_node` asks
        // `expression_tag_span` first and routes these into a dynamic
        // component's `:is` binding, because they name a *value*, not a
        // component name. Reaching this arm would mean that guard was bypassed,
        // and the dotted path would be emitted as a component name — the
        // `resolveComponent("a.b.c")` lookup of a component nobody registers
        // that #3421 removed.
        JSXElementName::MemberExpression(_) | JSXElementName::ThisExpression(_) => {
            unreachable!("member-expression tags lower to a dynamic component")
        }
    }
}

/// The authored name of an unqualified JSX identifier.
pub(crate) fn identifier_name<'n>(name: &'n JSXElementName<'_>) -> Option<&'n str> {
    match name {
        JSXElementName::Identifier(id) => Some(id.name.as_str()),
        JSXElementName::IdentifierReference(reference) => Some(reference.name.as_str()),
        JSXElementName::NamespacedName(_)
        | JSXElementName::MemberExpression(_)
        | JSXElementName::ThisExpression(_) => None,
    }
}

/// Whether a JSX element name refers to a component rather than an intrinsic
/// (HTML/SVG) element.
///
/// Native Vize JSX follows the casing convention also used by `vize_patina`:
/// a tag beginning with a lowercase ASCII letter is intrinsic. Babel
/// compatibility instead follows `@vue/babel-plugin-jsx`: only known HTML and
/// SVG identifiers are intrinsic, so unknown lowercase and MathML identifiers
/// resolve as components. Namespaced tags keep Vize's explicit namespace
/// handling in either mode.
pub(crate) fn is_component(name: &JSXElementName<'_>, babel_compat: bool) -> bool {
    match name {
        JSXElementName::Identifier(id) => is_identifier_component(id.name.as_str(), babel_compat),
        JSXElementName::IdentifierReference(reference) => {
            is_identifier_component(reference.name.as_str(), babel_compat)
        }
        JSXElementName::NamespacedName(named) => !is_intrinsic(named.name.name.as_str()),
        JSXElementName::MemberExpression(_) | JSXElementName::ThisExpression(_) => true,
    }
}

fn is_identifier_component(name: &str, babel_compat: bool) -> bool {
    if babel_compat {
        !vize_s0::is_html_tag(name) && !vize_s0::is_svg_tag(name)
    } else {
        !is_intrinsic(name)
    }
}

/// The namespace prefix of a JSX tag Vize cannot lower, with the span to point
/// a diagnostic at. `None` for an unprefixed tag or a known namespace.
pub(crate) fn unsupported_namespace<'n>(name: &'n JSXElementName<'_>) -> Option<(&'n str, Span)> {
    let JSXElementName::NamespacedName(named) = name else {
        return None;
    };
    let namespace = named.namespace.name.as_str();
    if KNOWN_TAG_NAMESPACES.contains(&namespace) {
        return None;
    }
    Some((namespace, named.span))
}

/// The span of a JSX tag that names a **JavaScript expression** rather than a
/// component name: `<a.b.c/>`, `<this.Dyn/>`, `<this/>`.
///
/// Slicing the span rather than rebuilding the path from the AST keeps the
/// emitted expression byte-identical to what the author wrote.
pub(crate) fn expression_tag_span(name: &JSXElementName<'_>) -> Option<Span> {
    match name {
        JSXElementName::MemberExpression(member) => Some(member.span),
        JSXElementName::ThisExpression(this) => Some(this.span),
        JSXElementName::Identifier(_)
        | JSXElementName::IdentifierReference(_)
        | JSXElementName::NamespacedName(_) => None,
    }
}

fn is_intrinsic(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase())
}
