//! Single text branches use a keyed Fragment in the client If emitter.
//!
//! Mirror that concrete branch shape so SSR markers hydrate against the real
//! client vnode. A lone element keeps its own root, and a For owns its markers.

use vize_atelier_core::{ElementType, TemplateChildNode};

pub(super) fn needs_fragment(children: &[TemplateChildNode<'_>]) -> bool {
    match children {
        [TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)] => true,
        [TemplateChildNode::Element(el)] if el.tag_type == ElementType::Template => {
            super::rendered_child_count(&el.children) > 1
        }
        [_] | [] => false,
        _ => super::rendered_child_count(children) > 1,
    }
}
