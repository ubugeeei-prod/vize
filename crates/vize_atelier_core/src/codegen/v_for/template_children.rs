//! Children of a multi-child template loop.

use crate::codegen::RenderMapMode;
use crate::TemplateChildNode;

use super::super::{
    children::generate_children_force_array, context::CodegenContext, node::generate_node,
};

/// Keep the direct array path for element-only fragments. Text runs need the
/// shared child generator so interpolations become VNodes rather than strings.
pub(super) fn generate_template_for_children<M: RenderMapMode>(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
) {
    if children.iter().any(|child| {
        matches!(
            child,
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        )
    }) {
        generate_children_force_array::<M>(ctx, children);
        return;
    }

    ctx.push("[");
    ctx.indent();
    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            ctx.push(",");
        }
        ctx.newline();
        generate_node::<M>(ctx, child);
    }
    ctx.deindent();
    ctx.newline();
    ctx.push("]");
}
