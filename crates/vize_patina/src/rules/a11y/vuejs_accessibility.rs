//! Ports of eslint-plugin-vuejs-accessibility 2.6.3 `utils/`, shared by the
//! a11y rules that follow that plugin.
//!
//! vue-eslint-parser lower-cases attribute names and directive arguments, so
//! names compare ASCII case-insensitively against lower-case expectations.
//! `isHiddenFromScreenReader` and the other text readings live in
//! `reference_values`.

use super::aria_query::{dom_element, is_html_element_name, is_interactive_role_name};
use super::handler_source::is_handler_source;
use super::reference_values::{BoundLiteral, bound_literal, strip_parentheses};
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement};
use oxc_ast::ast::Expression;

/// Whether Vue template semantics apply. JSX keeps its legacy lookups; a
/// lowered JSX root can still be viewed as a template document.
pub(crate) fn is_vue_template(ctx: &mut MarkupContext<'_, '_>) -> bool {
    ctx.is_template() && !is_jsx_filename(ctx.lint().filename)
}

pub(crate) fn is_jsx_filename(filename: &str) -> bool {
    filename.ends_with(".jsx") || filename.ends_with(".tsx")
}

/// What `getAttributeValue` returns for an attribute that is present.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AttrValue<'a> {
    /// A valueless attribute, an empty bound value or a `null` literal.
    Null,
    Str(&'a str),
    Num(f64),
    Bool(bool),
    /// Any other bound expression; the plugin passes the AST node through.
    Expr,
}

impl<'a> AttrValue<'a> {
    pub(crate) fn is_truthy(self) -> bool {
        match self {
            Self::Null => false,
            Self::Str(value) => !value.is_empty(),
            Self::Num(value) => value != 0.0 && !value.is_nan(),
            Self::Bool(value) => value,
            Self::Expr => true,
        }
    }

    pub(crate) const fn as_str(self) -> Option<&'a str> {
        match self {
            Self::Str(value) => Some(value),
            _ => None,
        }
    }
}

/// `getElementAttribute(node, name)`: a static attribute or a `v-bind` with a
/// static argument.
fn is_named(binding: &MarkupBinding<'_>, name: &str) -> bool {
    match binding.kind() {
        MarkupBindingKind::Attribute => binding.is_unqualified_arg_eq_ignore_ascii_case(name),
        MarkupBindingKind::Bind => binding.is_static_unqualified_arg_eq_ignore_ascii_case(name),
        _ => false,
    }
}

/// `getElementAttributeValue`, keeping a present-but-empty attribute apart
/// from a missing one (`getElementAttribute` callers need that).
pub(crate) fn attribute<'a>(element: &MarkupElement<'a>, name: &str) -> Option<AttrValue<'a>> {
    let mut found = None;
    element.walk_bindings(&mut |binding| {
        if found.is_none() && is_named(&binding, name) {
            found = Some(binding_value(&binding));
        }
    });
    found
}

/// The value as a truthy JavaScript value, `null` when missing.
pub(crate) fn truthy_attribute(element: &MarkupElement<'_>, name: &str) -> bool {
    attribute(element, name).is_some_and(AttrValue::is_truthy)
}

/// A string value, static or a bound string literal.
pub(crate) fn string_attribute<'a>(element: &MarkupElement<'a>, name: &str) -> Option<&'a str> {
    attribute(element, name).and_then(AttrValue::as_str)
}

fn binding_value<'a>(binding: &MarkupBinding<'a>) -> AttrValue<'a> {
    if binding.kind() == MarkupBindingKind::Attribute {
        return binding
            .static_value()
            .map_or(AttrValue::Null, AttrValue::Str);
    }
    if let Some(retained) = binding.retained_expression() {
        return expression_value(retained.ast.without_parentheses());
    }
    if let Some(value) = binding.static_value() {
        return AttrValue::Str(value);
    }
    binding
        .expression()
        .map_or(AttrValue::Null, expression_text_value)
}

fn expression_value<'a>(expression: &'a Expression<'a>) -> AttrValue<'a> {
    match expression {
        Expression::StringLiteral(literal) => AttrValue::Str(literal.value.as_str()),
        Expression::NumericLiteral(literal) => AttrValue::Num(literal.value),
        Expression::BooleanLiteral(literal) => AttrValue::Bool(literal.value),
        Expression::NullLiteral(_) => AttrValue::Null,
        _ => AttrValue::Expr,
    }
}

/// Visit every value a non-literal bound `name` can take through conditional
/// branches. `false` when the binding is absent, literal, or any branch is
/// unknown; the plugin treats such a binding as one opaque value.
pub(crate) fn bound_branches<'a>(
    element: &MarkupElement<'a>,
    name: &str,
    visit: &mut impl FnMut(AttrValue<'a>),
) -> bool {
    let mut first = None;
    element.walk_bindings(&mut |binding| {
        if first.is_none() && is_named(&binding, name) {
            first = Some(binding);
        }
    });
    let Some(binding) = first.filter(|binding| binding.kind() == MarkupBindingKind::Bind) else {
        return false;
    };
    let Some(retained) = binding.retained_expression() else {
        return false;
    };
    let expression = retained.ast.without_parentheses();
    matches!(expression, Expression::ConditionalExpression(_)) && branches(expression, visit)
}

fn branches<'a>(expression: &'a Expression<'a>, visit: &mut impl FnMut(AttrValue<'a>)) -> bool {
    match expression.without_parentheses() {
        Expression::ConditionalExpression(conditional) => {
            branches(&conditional.consequent, visit) && branches(&conditional.alternate, visit)
        }
        Expression::Identifier(identifier) if identifier.name == "undefined" => {
            visit(AttrValue::Null);
            true
        }
        Expression::UnaryExpression(unary)
            if unary.operator == oxc_syntax::operator::UnaryOperator::UnaryNegation =>
        {
            match unary.argument.without_parentheses() {
                Expression::NumericLiteral(literal) => {
                    visit(AttrValue::Num(-literal.value));
                    true
                }
                _ => false,
            }
        }
        other => match expression_value(other) {
            AttrValue::Expr => false,
            value => {
                visit(value);
                true
            }
        },
    }
}

/// Backends without a retained AST read the authored text as one literal.
fn expression_text_value(text: &str) -> AttrValue<'_> {
    match bound_literal(text) {
        Some(BoundLiteral::String(value)) => AttrValue::Str(value),
        Some(BoundLiteral::Number(value)) => AttrValue::Num(value),
        Some(BoundLiteral::Boolean(value)) => AttrValue::Bool(value),
        Some(BoundLiteral::Null) => AttrValue::Null,
        None if strip_parentheses(text).is_empty() => AttrValue::Null,
        Some(BoundLiteral::BigInt { .. }) | None => AttrValue::Expr,
    }
}

/// JavaScript `Number(string)`.
pub(crate) fn js_string_to_number(value: &str) -> f64 {
    let value = value.trim();
    if value.is_empty() {
        return 0.0;
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(digits) = value.strip_prefix(prefix) {
            return u128::from_str_radix(digits, radix)
                .ok()
                .filter(|_| !digits.starts_with('+'))
                .map_or(f64::NAN, |number| number as f64);
        }
    }
    match value {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    if value
        .bytes()
        .any(|byte| !(byte.is_ascii_digit() || matches!(byte, b'.' | b'e' | b'E' | b'+' | b'-')))
    {
        return f64::NAN;
    }
    value.parse().unwrap_or(f64::NAN)
}

/// `makeKebabCase`.
pub(crate) fn kebab_case(value: &str) -> vize_l0::String {
    let mut out = vize_l0::String::with_capacity(value.len() + 2);
    let mut previous_is_word = false;
    for character in value.chars() {
        let character = if character == '_' { '-' } else { character };
        if character.is_ascii_uppercase() && previous_is_word {
            out.push('-');
        }
        previous_is_word = character.is_ascii_alphanumeric();
        out.extend(character.to_lowercase());
    }
    out
}

/// `getElementType`: a string `is` value or the authored tag, kebab-cased.
pub(crate) fn element_type(element: &MarkupElement<'_>) -> vize_l0::String {
    match string_attribute(element, "is").filter(|value| !value.is_empty()) {
        Some(is) => kebab_case(is),
        None => kebab_case(element.tag()),
    }
}

/// Whether the current element is in the HTML namespace, from its ancestors.
/// The Relief entry reads the parsed namespace directly.
pub(crate) fn is_html_namespace(ctx: &MarkupContext<'_, '_>, element: &MarkupElement<'_>) -> bool {
    #[derive(Clone, Copy, PartialEq)]
    enum Ns {
        Html,
        Svg,
        MathMl,
    }
    let own = |inherited: Ns, tag: &str| match inherited {
        Ns::Html if tag.eq_ignore_ascii_case("svg") => Ns::Svg,
        Ns::Html if tag.eq_ignore_ascii_case("math") => Ns::MathMl,
        ns => ns,
    };
    let mut inherited = Ns::Html;
    for ancestor in ctx.ancestor_elements() {
        let tag = ancestor.tag();
        inherited = match own(inherited, tag) {
            Ns::Svg if matches!(tag, "foreignObject" | "desc" | "title") => Ns::Html,
            Ns::MathMl if matches!(tag, "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext") => {
                Ns::Html
            }
            ns => ns,
        };
    }
    own(inherited, element.tag()) == Ns::Html
}

/// `isCustomComponent`.
pub(crate) fn is_custom_component(
    element: &MarkupElement<'_>,
    html_namespace: impl FnOnce() -> bool,
) -> bool {
    attribute(element, "is").is_some() || (!is_html_element_name(element.tag()) && html_namespace())
}

/// `isPresentationRole`.
pub(crate) fn is_presentation_role(element: &MarkupElement<'_>) -> bool {
    matches!(
        string_attribute(element, "role"),
        Some("presentation" | "none")
    )
}

/// `isInteractiveRole`: some space-separated token names a widget role.
pub(crate) fn is_interactive_role(role: Option<AttrValue<'_>>) -> bool {
    role.and_then(AttrValue::as_str).is_some_and(|role| {
        role.split(' ')
            .any(|token| is_interactive_role_name(&token.to_ascii_lowercase()))
    })
}

/// `isInteractiveElement`: a DOM element whose implicit role is a widget.
pub(crate) fn is_interactive_element(element: &MarkupElement<'_>) -> bool {
    match element_type(element).as_str() {
        "input" | "button" | "th" | "td" | "tr" | "option" | "datalist" | "progress"
        | "textarea" => true,
        "a" | "area" => truthy_attribute(element, "href"),
        // The combobox concept wants neither attribute; `size > 1` never matches.
        "select" => truthy_attribute(element, "multiple") || !truthy_attribute(element, "size"),
        _ => false,
    }
}

/// `dom.has(getElementType(node))`.
pub(crate) fn is_dom_element(element: &MarkupElement<'_>) -> bool {
    dom_element(&element_type(element)).is_some()
}

/// `hasOnDirective`: a `v-on` with a static argument whose value is an
/// identifier or a function/statement body. JSX keeps its exact-name lookup.
pub(crate) fn has_on_directive(element: &MarkupElement<'_>, name: &str, template: bool) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        if found {
            return;
        }
        found = binding.kind() == MarkupBindingKind::On
            && if template {
                binding.is_static_unqualified_arg_eq_ignore_ascii_case(name)
                    && binding.expression().is_some_and(is_handler_source)
            } else {
                binding.is_static_unqualified_arg_exact(name)
                    && (binding.expression().is_some() || binding.retained_expression().is_some())
            };
    });
    found
}

/// `hasOnDirectives`.
pub(crate) fn has_on_directives(
    element: &MarkupElement<'_>,
    names: &[&str],
    template: bool,
) -> bool {
    names
        .iter()
        .any(|name| has_on_directive(element, name, template))
}

/// `utils/interactiveHandlers.json`.
pub(crate) const INTERACTIVE_HANDLERS: &[&str] = &[
    "click",
    "contextmenu",
    "dblclick",
    "doubleclick",
    "drag",
    "dragend",
    "dragenter",
    "dragexit",
    "dragleave",
    "dragover",
    "dragstart",
    "drop",
    "keydown",
    "keypress",
    "keyup",
    "mousedown",
    "mouseenter",
    "mouseleave",
    "mousemove",
    "mouseout",
    "mouseover",
    "mouseup",
];

#[cfg(test)]
mod tests {
    use super::kebab_case;

    #[test]
    fn kebab_case_matches_make_kebab_case() {
        for (input, expected) in [
            ("div", "div"),
            ("Input", "input"),
            ("MyOption", "my-option"),
            ("DIV", "d-i-v"),
            ("my_comp", "my-comp"),
            ("a_B", "a-b"),
            ("vue:my-item", "vue:my-item"),
            ("h1", "h1"),
        ] {
            assert_eq!(kebab_case(input), expected, "{input}");
        }
    }
}
