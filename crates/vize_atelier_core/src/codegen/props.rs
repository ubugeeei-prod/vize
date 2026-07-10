//! Props generation functions.

mod directives;
mod events;
mod generate;
mod object;
mod scan;
mod vbind;
mod von_vmodel;

use crate::relief_projection::ReliefRenderOp;
use crate::{PropNode, RuntimeHelper};

use super::{context::CodegenContext, expression::generate_expression};

pub use directives::{
    StaticMerge, generate_directive_prop_with_static,
    generate_slot_outlet_directive_prop_with_static, is_supported_directive,
};
pub use events::von_event_key_for;
pub use generate::generate_props;

/// Check if there's a v-bind without argument (object spread)
pub(crate) fn has_vbind_object(props: &[PropNode<'_>]) -> bool {
    props.iter().any(|prop| {
        matches!(
            ReliefRenderOp::from_prop(prop),
            ReliefRenderOp::Directive {
                name: "bind",
                arg: None,
                ..
            }
        )
    })
}

/// Check if there's a v-on without argument (event object spread)
pub(crate) fn has_von_object(props: &[PropNode<'_>]) -> bool {
    props.iter().any(|prop| {
        matches!(
            ReliefRenderOp::from_prop(prop),
            ReliefRenderOp::Directive {
                name: "on",
                arg: None,
                ..
            }
        )
    })
}

/// Generate the v-bind object expression
pub(crate) fn generate_vbind_object_exp(ctx: &mut CodegenContext, props: &[PropNode<'_>]) {
    for prop in props {
        if let ReliefRenderOp::Directive {
            name: "bind",
            arg: None,
            exp: Some(exp),
            ..
        } = ReliefRenderOp::from_prop(prop)
            && let Some(exp) = exp.node()
        {
            generate_expression(ctx, exp);
            return;
        }
    }
}

/// Generate the v-on object expression wrapped with toHandlers
pub(crate) fn generate_von_object_exp(ctx: &mut CodegenContext, props: &[PropNode<'_>]) {
    ctx.use_helper(RuntimeHelper::ToHandlers);
    ctx.push(ctx.helper(RuntimeHelper::ToHandlers));
    ctx.push("(");
    for prop in props {
        if let ReliefRenderOp::Directive {
            name: "on",
            arg: None,
            exp: Some(exp),
            ..
        } = ReliefRenderOp::from_prop(prop)
            && let Some(exp) = exp.node()
        {
            generate_expression(ctx, exp);
            ctx.push(", true"); // true for handlerOnly
            break;
        }
    }
    ctx.push(")");
}
