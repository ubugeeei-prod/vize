//! Single text branches use a keyed Fragment in the client If emitter.
//!
//! Mirror that concrete branch shape so SSR markers hydrate against the real
//! client vnode. A lone element keeps its own root, and a For owns its markers.

use vize_atelier_core::TemplateChildNode;

pub(super) fn single_text_branch(children: &[TemplateChildNode<'_>]) -> bool {
    matches!(
        children,
        [TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)]
    )
}
