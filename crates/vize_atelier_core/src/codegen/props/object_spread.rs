//! Object-spread generation for Babel JSX compatibility.

use crate::PropNode;

use super::super::context::CodegenContext;
use super::generate::generate_props_object;
use super::generate_vbind_object_exp;
use super::scan::PropsScan;

/// Emit the `mergeProps: false` path, preserving JavaScript spread semantics.
pub(super) fn try_generate_without_merge_props(
    ctx: &mut CodegenContext,
    props: &[PropNode<'_>],
    scope_id: Option<&str>,
    scan: &PropsScan<'_>,
) -> bool {
    if ctx.merge_props {
        return false;
    }

    // Babel leaves a lone JSX spread as the props expression itself. Once
    // another visible prop participates, it emits one object literal with
    // spread members in authored order instead of calling `mergeProps`.
    if scope_id.is_none()
        && scan.has_vbind_obj
        && !scan.has_von_obj
        && !scan.has_other
        && scan.visible_count(false) == 1
    {
        generate_vbind_object_exp(ctx, props);
    } else {
        generate_props_object(ctx, props, false, scan);
    }
    true
}
