//! Individual operation code generators.
//!
//! Each function emits JavaScript code for a specific IR operation node.

mod component;
mod component_props;
mod component_slots;
mod directives;
mod dom;
mod events;
mod for_loop;
mod if_block;
mod insertion;
mod refs;
mod slots;

use crate::ir::OperationNode;
use vize_carton::FxHashMap;

use super::context::GenerateContext;

/// Generate operation
pub(crate) fn generate_operation(
    ctx: &mut GenerateContext,
    op: &OperationNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    match op {
        OperationNode::SetProp(set_prop) => {
            dom::generate_set_prop(ctx, set_prop);
        }
        OperationNode::SetDynamicProps(set_props) => {
            dom::generate_set_dynamic_props(ctx, set_props);
        }
        OperationNode::SetText(set_text) => {
            dom::generate_set_text(ctx, set_text);
        }
        OperationNode::SetEvent(set_event) => {
            events::generate_set_event(ctx, set_event);
        }
        OperationNode::SetHtml(set_html) => {
            dom::generate_set_html(ctx, set_html);
        }
        OperationNode::SetTemplateRef(set_ref) => {
            refs::generate_set_template_ref(ctx, set_ref);
        }
        OperationNode::InsertNode(insert) => {
            refs::generate_insert_node(ctx, insert);
        }
        OperationNode::PrependNode(prepend) => {
            refs::generate_prepend_node(ctx, prepend);
        }
        OperationNode::Directive(directive) => {
            directives::generate_directive(ctx, directive);
        }
        OperationNode::If(if_node) => {
            if_block::generate_if(ctx, if_node, element_template_map);
        }
        OperationNode::For(for_node) => {
            for_loop::generate_for(ctx, for_node, element_template_map);
        }
        OperationNode::CreateComponent(component) => {
            component::generate_create_component(ctx, component, element_template_map);
        }
        OperationNode::SlotOutlet(slot) => {
            slots::generate_slot_outlet(ctx, slot);
        }
        OperationNode::GetTextChild(get_text) => {
            refs::generate_get_text_child(ctx, get_text);
        }
        OperationNode::ChildRef(child_ref) => {
            refs::generate_child_ref(ctx, child_ref);
        }
        OperationNode::NextRef(next_ref) => {
            refs::generate_next_ref(ctx, next_ref);
        }
    }
}
