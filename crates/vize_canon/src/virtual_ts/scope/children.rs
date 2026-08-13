//! Recursive traversal of nested template scopes.

use vize_carton::{String, profile};
use vize_croquis::ScopeKind;

use super::closures::generate_scope_node;
use super::context::ScopeGenContext;
use crate::virtual_ts::types::VizeMapping;

pub(super) fn generate_child_scopes(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ScopeGenContext<'_, '_>,
    parent_scope_id: u32,
    indent: &str,
) {
    if let Some(child_ids) = ctx.children_map.get(&parent_scope_id) {
        for &child_id in child_ids {
            if let Some(child_scope) = ctx.summary.scopes.get_scope(child_id)
                && matches!(
                    child_scope.kind,
                    ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::EventHandler
                )
            {
                profile!(
                    "canon.virtual_ts.scope_node",
                    generate_scope_node(ts, mappings, ctx, child_scope, indent)
                );
            }
        }
    }
}
