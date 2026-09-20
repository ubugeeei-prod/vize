//! The lexical scope a patterned-template `v-match` lowers to (RFC 823).
//!
//! The scope's block runs once against a computed of its source, so the
//! selector and each arm's bindings stay reactive without `createFor`: no list
//! fragment is created, a single-root match keeps attribute fallthrough, and
//! the rendered nodes line up with the VDOM and server output.

use crate::ir::ForIRNode;
use vize_carton::{FxHashMap, String, ToCompactString, cstr};

use super::{
    super::{
        context::{ForScope, GenerateContext},
        generate_block,
    },
    insertion::{block_requires_parent_insertion_state, emit_insertion_state},
};

pub(super) fn generate_match_scope(
    ctx: &mut GenerateContext,
    for_node: &ForIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    ctx.use_helper("computed");
    emit_insertion_state(ctx, for_node.parent, for_node.anchor);

    let depth = ctx.for_scopes.len();
    let source = ctx.resolve_expression_node(&for_node.source);
    // Aliases resolve through the scope exactly like loop aliases do: the
    // selector result as `_for_itemN.value`, a binding as a path into it.
    ctx.for_scopes.push(ForScope {
        value_alias: for_node
            .value
            .as_ref()
            .map(|value| String::new(value.content)),
        key_alias: None,
        index_alias: None,
        depth,
    });

    let scope_item = cstr!("_for_item{depth}");
    let id = for_node.id.to_compact_string();
    ctx.push_line(&["const n", &id, " = ((", &scope_item, ") => {"].concat());
    ctx.indent();
    if block_requires_parent_insertion_state(&for_node.render) {
        emit_insertion_state(ctx, for_node.parent, for_node.anchor);
    }
    ctx.push_component_scope();
    generate_block(ctx, &for_node.render, element_template_map);
    ctx.pop_component_scope();
    ctx.deindent();
    ctx.push_line(&["})(_computed(() => (", &source, ")))"].concat());

    ctx.for_scopes.pop();
}
