//! Placeholder planning for blocks that need insertion.
//!
//! Vue 3.6 inserts components, slot outlets, `v-if` and `v-for` blocks with
//! `setInsertionState`. A block keeps a `<!---->` placeholder in its parent's
//! template only when a template-rendered sibling (text, interpolation or
//! element) follows it: the placeholder is the insertion point on the client
//! and the block's hydration unit on the server. Trailing blocks have no
//! placeholder; they append, carrying their logical unit index instead.

use vize_carton::ensure_sufficient_stack;

use super::template::is_template_backed_element;
use super::{ElementType, TemplateChildNode};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Unit {
    Rendered,
    Block,
}

fn collect_units(children: &[TemplateChildNode<'_>], units: &mut std::vec::Vec<Unit>) {
    for child in children {
        match child {
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_) => {
                units.push(Unit::Rendered);
            }
            TemplateChildNode::Element(el) if el.tag_type == ElementType::Template => {
                ensure_sufficient_stack(|| collect_units(&el.children, units));
            }
            TemplateChildNode::Element(el) if is_template_backed_element(el) => {
                units.push(Unit::Rendered);
            }
            TemplateChildNode::Element(_)
            | TemplateChildNode::If(_)
            | TemplateChildNode::For(_) => units.push(Unit::Block),
            _ => {}
        }
    }
}

/// For each block among `children` (template wrappers flattened, document
/// order), whether it keeps a template placeholder because a
/// template-rendered sibling follows it.
pub(crate) fn block_placeholders(children: &[TemplateChildNode<'_>]) -> std::vec::Vec<bool> {
    let mut units = std::vec::Vec::new();
    collect_units(children, &mut units);
    let mut rendered_after = false;
    let mut flags = std::vec::Vec::new();
    for unit in units.iter().rev() {
        match unit {
            Unit::Rendered => rendered_after = true,
            Unit::Block => flags.push(rendered_after),
        }
    }
    flags.reverse();
    flags
}
