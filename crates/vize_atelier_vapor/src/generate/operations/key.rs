//! Keyed fragments use the runtime-owned replacement and disposal scope.

use crate::ir::KeyIRNode;
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{FxHashMap, cstr};

use super::{
    super::{context::GenerateContext, generate_block},
    insertion::emit_insertion_state,
};

pub(super) fn generate_key(
    ctx: &mut GenerateContext,
    node: &KeyIRNode<'_>,
    templates: &FxHashMap<usize, usize>,
) {
    ctx.use_helper("createKeyedFragment");
    emit_insertion_state(ctx, node.parent, node.anchor);
    let mut head = EmitDocument::plain(&cstr!("const n{} = _createKeyedFragment(() => (", node.id));
    head.push_spanned(&ctx.spanned_expression_node(&node.value));
    head.push_str("), () => {");
    ctx.push_line_spanned(&head);
    let transition = ctx.transition_slot;
    let slot_root = transition && node.parent.is_none();
    ctx.transition_slot = false;
    let fragment = ctx.is_fragment;
    ctx.is_fragment = true;
    ctx.indent();
    ctx.push_component_scope();
    generate_block(ctx, &node.render, templates);
    ctx.pop_component_scope();
    ctx.deindent();
    ctx.push_line(if slot_root { "}, true)" } else { "})" });
    ctx.transition_slot = transition;
    ctx.is_fragment = fragment;
}
