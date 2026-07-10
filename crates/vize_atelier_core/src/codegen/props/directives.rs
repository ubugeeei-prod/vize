//! Directive-to-prop generation (v-bind, v-on, v-model, v-html, v-text).

use crate::relief_projection::ReliefRenderOp;
use crate::{DirectiveNode, ExpressionNode, RuntimeHelper};

use super::super::{
    context::CodegenContext, expression::generate_expression,
    helpers::is_constant_simple_expression,
};
use super::vbind::generate_vbind_prop;
use super::von_vmodel::{generate_vmodel_prop, generate_von_prop};

/// Check if an expression is a static literal (no runtime identifiers).
/// Returns true for: object literals, array literals, string literals, numbers
/// that don't reference any runtime variables (no `_ctx.` after processing).
pub(super) fn is_static_expression(exp: &ExpressionNode<'_>, ctx: &CodegenContext) -> bool {
    match exp {
        ExpressionNode::Simple(simple) => {
            is_constant_simple_expression(simple, ctx.options.binding_metadata.as_ref())
        }
        ExpressionNode::Compound(_) => false,
    }
}

/// Check if a directive will produce valid output
pub fn is_supported_directive(dir: &DirectiveNode<'_>) -> bool {
    let ReliefRenderOp::Directive { name, arg, .. } = ReliefRenderOp::from_directive(dir) else {
        unreachable!("directive classification requires ReliefRenderOp::Directive");
    };
    // v-model with dynamic arg on components needs special props handling
    // Static v-model is handled via withDirectives for native elements or transformed for components
    if name == "model" {
        return arg.and_then(|arg| arg.node()).is_some_and(|arg| match arg {
            ExpressionNode::Simple(exp) => !exp.is_static,
            ExpressionNode::Compound(_) => true,
        });
    }
    matches!(name, "bind" | "on" | "html" | "text")
}

/// A static class/style attribute that will be merged with a dynamic
/// `:class`/`:style` binding, plus whether the static value appears before
/// the dynamic one in source order (Vue preserves source order in the merged
/// array).
#[derive(Clone, Copy, Default)]
pub struct StaticMerge<'a> {
    pub class: Option<&'a str>,
    pub class_before: bool,
    pub style: Option<&'a str>,
    pub style_before: bool,
}

impl<'a> StaticMerge<'a> {
    /// Build the merge metadata from an element's props in source order.
    pub fn from_props(props: &'a [crate::PropNode<'a>]) -> Self {
        let mut merge = StaticMerge::default();
        let mut class_index = None;
        let mut style_index = None;
        for (index, prop) in props.iter().enumerate() {
            match ReliefRenderOp::from_prop(prop) {
                ReliefRenderOp::Attribute { name, value, .. } => {
                    if name == "class" && merge.class.is_none() {
                        merge.class = value;
                        class_index = Some(index);
                    } else if name == "style" && merge.style.is_none() {
                        merge.style = value;
                        style_index = Some(index);
                    }
                }
                ReliefRenderOp::Directive { name, arg, .. } => {
                    if name == "bind"
                        && let Some(ExpressionNode::Simple(exp)) = arg.and_then(|arg| arg.node())
                        && exp.is_static
                    {
                        if exp.content == "class" && class_index.is_some_and(|i| i < index) {
                            merge.class_before = true;
                        } else if exp.content == "style" && style_index.is_some_and(|i| i < index) {
                            merge.style_before = true;
                        }
                    }
                }
                _ => unreachable!(
                    "element props lower to attribute or directive Relief projection operations"
                ),
            }
        }
        merge
    }
}

/// Generate directive as prop with optional static class/style merging
pub fn generate_directive_prop_with_static(
    ctx: &mut CodegenContext,
    dir: &DirectiveNode<'_>,
    static_merge: StaticMerge<'_>,
) {
    generate_directive_prop_with_static_key_casing(
        ctx,
        dir,
        static_merge,
        StaticBindKeyCasing::Preserve,
    );
}

/// Generate a directive prop for a `<slot>` outlet.
///
/// Vue camelizes static slot prop keys before passing them to renderSlot.
pub fn generate_slot_outlet_directive_prop_with_static(
    ctx: &mut CodegenContext,
    dir: &DirectiveNode<'_>,
    static_merge: StaticMerge<'_>,
) {
    generate_directive_prop_with_static_key_casing(
        ctx,
        dir,
        static_merge,
        StaticBindKeyCasing::Camelize,
    );
}

#[derive(Clone, Copy)]
pub(super) enum StaticBindKeyCasing {
    Preserve,
    Camelize,
}

fn generate_directive_prop_with_static_key_casing(
    ctx: &mut CodegenContext,
    dir: &DirectiveNode<'_>,
    static_merge: StaticMerge<'_>,
    static_key_casing: StaticBindKeyCasing,
) {
    let ReliefRenderOp::Directive { name, exp, .. } = ReliefRenderOp::from_directive(dir) else {
        unreachable!("directive emission requires ReliefRenderOp::Directive");
    };
    match name {
        "bind" => {
            generate_vbind_prop(ctx, dir, static_merge, static_key_casing);
        }
        "on" => {
            generate_von_prop(ctx, dir);
        }
        "model" => {
            generate_vmodel_prop(ctx, dir);
        }
        "html" => {
            // v-html="rawHtml" -> innerHTML: _ctx.rawHtml
            ctx.push("innerHTML: ");
            if let Some(exp) = exp.and_then(|exp| exp.node()) {
                generate_expression(ctx, exp);
            } else {
                ctx.push("undefined");
            }
        }
        "text" => {
            // v-text="message" -> textContent: _toDisplayString(_ctx.message)
            ctx.use_helper(RuntimeHelper::ToDisplayString);
            ctx.push("textContent: ");
            ctx.push(ctx.helper(RuntimeHelper::ToDisplayString));
            ctx.push("(");
            if let Some(exp) = exp.and_then(|exp| exp.node()) {
                generate_expression(ctx, exp);
            } else {
                ctx.push("undefined");
            }
            ctx.push(")");
        }
        _ => {
            // Other directives are skipped by is_supported_directive()
            // This case should not be reached in normal operation
        }
    }
}
