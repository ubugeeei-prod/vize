//! Shared helper functions for accessibility rules.
//!
//! These helpers are extracted from common patterns used across a11y rules
//! to avoid code duplication.

use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode};

/// Normalize a configured component name once, outside template traversal.
pub(crate) fn canonical_component_name(name: &str) -> vize_l0::String {
    vize_l0::hyphenate(name).to_ascii_lowercase()
}

/// Compare an authored tag with pre-normalized component names without allocating.
pub(crate) fn matches_configured_component(tag: &str, configured: &[vize_l0::String]) -> bool {
    configured
        .iter()
        .any(|canonical| tag_matches_canonical_component(tag, canonical))
}

fn tag_matches_canonical_component(tag: &str, canonical: &str) -> bool {
    let mut expected = canonical.bytes();
    for (index, byte) in tag.bytes().enumerate() {
        if byte.is_ascii_uppercase() && index > 0 && expected.next() != Some(b'-') {
            return false;
        }
        if expected.next() != Some(byte.to_ascii_lowercase()) {
            return false;
        }
    }
    expected.next().is_none()
}

/// Check if an element represents a Vue slot outlet.
pub fn is_slot_element(element: &ElementNode) -> bool {
    matches!(element.tag_type, ElementType::Slot) || element.tag == "slot"
}

/// Check if an element is natively interactive (has implicit keyboard support)
pub fn is_interactive_element(tag: &str) -> bool {
    matches!(
        tag,
        "a" | "button"
            | "input"
            | "select"
            | "textarea"
            | "details"
            | "summary"
            | "video"
            | "audio"
    )
}

/// Check if a role name is an interactive ARIA role
pub fn is_interactive_role(role: &str) -> bool {
    matches!(
        role,
        "button"
            | "link"
            | "checkbox"
            | "menuitem"
            | "menuitemcheckbox"
            | "menuitemradio"
            | "option"
            | "radio"
            | "searchbox"
            | "switch"
            | "textbox"
            | "tab"
            | "treeitem"
            | "gridcell"
            | "combobox"
            | "listbox"
            | "slider"
            | "spinbutton"
            | "scrollbar"
            | "tree"
            | "tablist"
            | "menu"
            | "row"
            | "columnheader"
    )
}

/// Events that make an otherwise static element behave interactively.
pub(crate) const INTERACTIVE_HANDLER_EVENTS: &[&str] = &[
    "click",
    "dblclick",
    "mousedown",
    "mouseup",
    "keydown",
    "keyup",
    "keypress",
];

/// Get the static (non-dynamic) value of an attribute on an element.
/// Returns None if the attribute is not found or is dynamically bound.
pub fn get_static_attribute_value<'a>(element: &'a ElementNode, name: &str) -> Option<&'a str> {
    for prop in &element.props {
        if let PropNode::Attribute(attr) = prop
            && attr.name == name
        {
            return attr.value.as_ref().map(|v| v.content);
        }
    }
    None
}

/// Get a static attribute value, including v-bind expressions that are string literals.
///
/// This is useful for rules that need exact attribute values but should not warn when
/// Vue's bind syntax is only wrapping a literal value, e.g. `:type="'hidden'"`.
pub fn get_static_or_bound_literal_attribute_value<'a>(
    element: &'a ElementNode,
    name: &str,
) -> Option<&'a str> {
    for prop in &element.props {
        match prop {
            PropNode::Attribute(attr) if attr.name == name => {
                return attr.value.as_ref().map(|v| v.content);
            }
            PropNode::Directive(dir) if dir.name == "bind" => {
                let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
                    continue;
                };
                if arg.content != name {
                    continue;
                }
                let Some(ExpressionNode::Simple(exp)) = &dir.exp else {
                    continue;
                };
                if let Some(value) = string_literal_value(exp.content) {
                    return Some(value);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether the element is explicitly hidden from the accessibility tree.
pub fn is_aria_hidden_true(element: &ElementNode) -> bool {
    if get_static_or_bound_literal_attribute_value(element, "aria-hidden") == Some("true") {
        return true;
    }

    for prop in &element.props {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        if dir.name != "bind" {
            continue;
        }
        let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
            continue;
        };
        let Some(ExpressionNode::Simple(exp)) = &dir.exp else {
            continue;
        };
        if arg.is_static && arg.content == "aria-hidden" && exp.content.trim() == "true" {
            return true;
        }
    }
    false
}

pub fn string_literal_value(content: &str) -> Option<&str> {
    let content = content.trim();
    let quote = content.as_bytes().first()?;
    if content.len() < 2
        || !matches!(quote, b'\'' | b'"' | b'`')
        || content.as_bytes().last() != Some(quote)
    {
        return None;
    }

    let inner = content.get(1..content.len() - 1)?;
    if !is_single_literal_body(inner, *quote) {
        return None;
    }

    Some(inner)
}

fn is_single_literal_body(inner: &str, quote: u8) -> bool {
    let mut escaped = false;
    let mut bytes = inner.as_bytes().iter().copied().peekable();
    while let Some(byte) = bytes.next() {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if byte == quote {
            return false;
        }
        if quote == b'`' && byte == b'$' && bytes.peek() == Some(&b'{') {
            return false;
        }
    }

    true
}

/// Elements that do not support ARIA attributes
pub const ARIA_UNSUPPORTED_ELEMENTS: &[&str] = &[
    "base", "col", "colgroup", "head", "html", "link", "meta", "noembed", "noscript", "param",
    "picture", "script", "source", "style", "title", "track",
];

/// A static attribute, a string-literal binding, or some other present binding.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AuthoredAttr<'a> {
    Missing,
    Static(&'a str),
    /// Present, and not a string literal. `null` and `undefined` stay [`Missing`].
    Dynamic,
}

/// Mapping of elements to their implicit ARIA roles.
///
/// `nav`, `section`, and `form` are landmarks only when they have an accessible
/// name. A `select` is a combobox until `multiple` or `size` greater than 1
/// makes it a listbox. An `<input>` with a dynamic `type` has no static role.
pub fn get_implicit_role(tag: &str, element: &ElementNode) -> Option<&'static str> {
    get_implicit_role_by_attr(tag, |name| relief_authored_attr(element, name))
}

pub(super) fn relief_authored_attr<'a>(element: &'a ElementNode, name: &str) -> AuthoredAttr<'a> {
    for prop in &element.props {
        match prop {
            PropNode::Attribute(attr) if attr.name == name => {
                return AuthoredAttr::Static(
                    attr.value.as_ref().map(|value| value.content).unwrap_or(""),
                );
            }
            PropNode::Directive(dir) if dir.name == "bind" => {
                let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
                    continue;
                };
                if !arg.is_static || arg.content != name {
                    continue;
                }
                return match &dir.exp {
                    Some(ExpressionNode::Simple(simple)) => authored_expression(simple.content),
                    Some(ExpressionNode::Compound(_)) | None => AuthoredAttr::Dynamic,
                };
            }
            _ => {}
        }
    }
    AuthoredAttr::Missing
}

fn authored_expression(content: &str) -> AuthoredAttr<'_> {
    let content = content.trim();
    if content.is_empty() || matches!(content, "null" | "undefined") {
        return AuthoredAttr::Missing;
    }
    match string_literal_value(content) {
        Some(literal) => AuthoredAttr::Static(literal),
        None => AuthoredAttr::Dynamic,
    }
}

fn has_accessible_name<'a>(mut attr: impl FnMut(&str) -> AuthoredAttr<'a>) -> bool {
    ["aria-label", "aria-labelledby", "title"]
        .into_iter()
        .any(|name| match attr(name) {
            AuthoredAttr::Static(value) => !value.trim().is_empty(),
            AuthoredAttr::Dynamic => true,
            AuthoredAttr::Missing => false,
        })
}

pub(super) fn get_implicit_role_by_attr<'a>(
    tag: &str,
    mut attr: impl FnMut(&str) -> AuthoredAttr<'a>,
) -> Option<&'static str> {
    match tag {
        "a" | "area" => match attr("href") {
            AuthoredAttr::Static(_) => Some("link"),
            AuthoredAttr::Missing | AuthoredAttr::Dynamic => None,
        },
        "article" => Some("article"),
        "aside" => Some("complementary"),
        "button" => Some("button"),
        "datalist" => Some("listbox"),
        "details" => Some("group"),
        "dialog" => Some("dialog"),
        "div" | "span" => Some("generic"),
        "fieldset" => Some("group"),
        "figure" => Some("figure"),
        "footer" => Some("contentinfo"),
        "form" => has_accessible_name(&mut attr).then_some("form"),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some("heading"),
        "header" => Some("banner"),
        "hr" => Some("separator"),
        "img" => match attr("alt") {
            AuthoredAttr::Static("") => Some("presentation"),
            _ => Some("img"),
        },
        "input" => {
            let input_type = match attr("type") {
                AuthoredAttr::Missing => "text",
                AuthoredAttr::Static(value) => value,
                AuthoredAttr::Dynamic => return None,
            };
            let role = match input_type {
                "button" | "image" | "reset" | "submit" => Some("button"),
                "checkbox" => Some("checkbox"),
                "radio" => Some("radio"),
                "range" => Some("slider"),
                "search" => Some("searchbox"),
                "email" | "tel" | "text" | "url" | "" => Some("textbox"),
                "number" => Some("spinbutton"),
                _ => None,
            };
            if role == Some("textbox") && !matches!(attr("list"), AuthoredAttr::Missing) {
                Some("combobox")
            } else {
                role
            }
        }
        "li" => Some("listitem"),
        "main" => Some("main"),
        "menu" => Some("list"),
        "meter" => Some("meter"),
        "nav" => has_accessible_name(&mut attr).then_some("navigation"),
        "ol" | "ul" => Some("list"),
        "optgroup" => Some("group"),
        "option" => Some("option"),
        "output" => Some("status"),
        "p" => Some("paragraph"),
        "progress" => Some("progressbar"),
        "section" => has_accessible_name(&mut attr).then_some("region"),
        "select" => {
            let multiple = !matches!(attr("multiple"), AuthoredAttr::Missing);
            let sized = matches!(attr("size"), AuthoredAttr::Static(value) if value.parse::<u32>().ok().is_some_and(|size| size > 1));
            if multiple || sized {
                Some("listbox")
            } else {
                Some("combobox")
            }
        }
        "summary" => Some("button"),
        "table" => Some("table"),
        "tbody" | "tfoot" | "thead" => Some("rowgroup"),
        "td" => Some("cell"),
        "textarea" => Some("textbox"),
        "th" => Some("columnheader"),
        "tr" => Some("row"),
        _ => None,
    }
}

/// Required ARIA properties for specific roles
pub fn get_required_aria_props(role: &str) -> &'static [&'static str] {
    match role {
        "checkbox" => &["aria-checked"],
        "combobox" => &["aria-controls", "aria-expanded"],
        "heading" => &["aria-level"],
        "meter" => &["aria-valuenow"],
        "menuitemcheckbox" | "menuitemradio" => &["aria-checked"],
        "option" => &["aria-selected"],
        "radio" => &["aria-checked"],
        "scrollbar" => &["aria-controls", "aria-valuenow"],
        "separator" => &["aria-valuenow"],
        "slider" => &["aria-valuenow"],
        "switch" => &["aria-checked"],
        "treeitem" => &["aria-selected"],
        _ => &[],
    }
}
