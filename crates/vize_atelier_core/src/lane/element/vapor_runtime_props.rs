//! Vapor: static attributes the template string cannot carry (upstream's
//! `isRuntimeOnlyProp`) become bindings of their literal value, so the element
//! is referenced and a runtime setter applies them. `innerHTML` and
//! `textContent` write content, `<textarea>` and `<select>` ignore a `value`
//! attribute, and the server drops a checkbox's `true-value` / `false-value`.

use vize_l0::Box;

use crate::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, SimpleExpressionNode,
};

use super::TransformContext;

pub(super) fn bind_runtime_only_attrs<'a>(
    ctx: &TransformContext<'a>,
    el: &mut Box<'a, ElementNode<'a>>,
) {
    if el.tag_type != ElementType::Element {
        return;
    }
    let form = match el.tag {
        "input" => Some(true),
        "textarea" | "select" => Some(false),
        _ => None,
    };
    for index in 0..el.props.len() {
        let Some(PropNode::Attribute(attr)) = el.props.get(index) else {
            continue;
        };
        if !runtime_only(el, attr.name, form) {
            continue;
        }
        let mut dir = DirectiveNode::new(ctx.allocator, "bind", attr.loc.clone());
        let arg = SimpleExpressionNode::new(attr.name, true, attr.name_loc.clone());
        let value = attr.value.as_ref().map_or_else(
            || SimpleExpressionNode::new("", true, attr.loc.clone()),
            |value| SimpleExpressionNode::new(value.content, true, value.loc.clone()),
        );
        dir.arg = Some(ExpressionNode::Simple(Box::new_in(arg, &ctx.allocator)));
        dir.exp = Some(ExpressionNode::Simple(Box::new_in(value, &ctx.allocator)));
        if let Some(prop) = el.props.get_mut(index) {
            *prop = PropNode::Directive(Box::new_in(dir, &ctx.allocator));
        }
    }
}

/// Dispatches on the first byte: most attributes fail there, before any
/// comparison (wide static elements pay one branch per attribute).
/// `form` is `Some(true)` on an `<input>`, `Some(false)` on a `<textarea>` or
/// `<select>`.
fn runtime_only(el: &ElementNode<'_>, name: &str, form: Option<bool>) -> bool {
    match name.as_bytes().first() {
        Some(b'i') => name == "innerHTML",
        Some(b't') => {
            name == "textContent"
                || form == Some(true) && name == "true-value" && may_be_checkbox(el)
        }
        Some(b'v') => form == Some(false) && name == "value",
        Some(b'f') => form == Some(true) && name == "false-value" && may_be_checkbox(el),
        _ => false,
    }
}

/// A `type` of `checkbox` or a bound one, or none beside a spread that may set it.
fn may_be_checkbox(el: &ElementNode<'_>) -> bool {
    let mut spread = false;
    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) if attr.name == "type" => {
                if let Some(value) = &attr.value {
                    return value.content == "checkbox";
                }
            }
            PropNode::Directive(dir) if dir.name == "bind" => match &dir.arg {
                Some(ExpressionNode::Simple(arg)) if arg.is_static => {
                    if arg.content == "type" {
                        return true;
                    }
                }
                _ => spread = true,
            },
            _ => {}
        }
    }
    spread
}
