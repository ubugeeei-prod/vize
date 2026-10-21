//! Vapor: static attributes the template string cannot carry (upstream's
//! `isRuntimeOnlyProp`) become bindings of their literal value, so the element
//! is referenced and a runtime setter applies them. `innerHTML` and
//! `textContent` write content, `<textarea>` and `<select>` ignore a `value`
//! attribute, and the server drops a checkbox's `true-value` / `false-value`.

use vize_l0::Box;

use crate::{
    AttributeNode, DirectiveNode, ElementNode, ExpressionNode, PropNode, SimpleExpressionNode,
};

use super::TransformContext;

#[derive(Clone, Copy)]
pub(super) struct RuntimeOnlyContext {
    form: Option<bool>,
    checkbox: bool,
}

impl RuntimeOnlyContext {
    pub(super) fn new(el: &ElementNode<'_>) -> Self {
        let form = match el.tag {
            "input" => Some(true),
            "textarea" | "select" => Some(false),
            _ => None,
        };
        Self {
            form,
            checkbox: form == Some(true) && may_be_checkbox(el),
        }
    }

    #[inline(always)]
    pub(super) fn matches(self, name: &str) -> bool {
        runtime_only(name, self.form, self.checkbox)
    }

    pub(super) fn directive<'a>(
        ctx: &TransformContext<'a>,
        attr: &AttributeNode<'a>,
    ) -> PropNode<'a> {
        let mut dir = DirectiveNode::new(ctx.allocator, "bind", attr.loc.clone());
        let arg = SimpleExpressionNode::new(attr.name, true, attr.name_loc.clone());
        let value = attr.value.as_ref().map_or_else(
            || SimpleExpressionNode::new("", true, attr.loc.clone()),
            |value| SimpleExpressionNode::new(value.content, true, value.loc.clone()),
        );
        dir.arg = Some(ExpressionNode::Simple(Box::new_in(arg, &ctx.allocator)));
        dir.exp = Some(ExpressionNode::Simple(Box::new_in(value, &ctx.allocator)));
        PropNode::Directive(Box::new_in(dir, &ctx.allocator))
    }
}

/// Dispatches on the first byte: most attributes fail there, before any
/// comparison (wide static elements pay one branch per attribute).
/// `form` is `Some(true)` on an `<input>`, `Some(false)` on a `<textarea>` or
/// `<select>`.
#[inline(always)]
fn runtime_only(name: &str, form: Option<bool>, checkbox: bool) -> bool {
    match name.as_bytes().first() {
        Some(b'i') => name == "innerHTML",
        Some(b't') => {
            name == "textContent" || form == Some(true) && checkbox && name == "true-value"
        }
        Some(b'v') => form == Some(false) && name == "value",
        Some(b'f') => form == Some(true) && checkbox && name == "false-value",
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
