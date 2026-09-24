//! A small `v-once` surface: an element with direct text and bound props.
//! More complex descendants retain the existing lane until their one-time
//! control-flow and component contracts are checked independently.

use super::super::{BindingKind, Content, Node};
use super::Result;
use crate::s3::LegacyReason;

pub(super) fn check(nodes: &[Node<'_>], parents: &[Option<usize>]) -> Result<()> {
    for (index, node) in nodes.iter().enumerate() {
        if !node
            .bindings
            .iter()
            .any(|binding| binding.kind == BindingKind::Once)
        {
            continue;
        }
        if !matches!(node.content, Content::Element { tag, .. } if tag != "template")
            || node
                .bindings
                .iter()
                .any(|binding| !matches!(binding.kind, BindingKind::Once | BindingKind::Prop))
            || node.children.iter().any(|child| {
                !nodes
                    .get(*child)
                    .is_some_and(|child| matches!(child.content, Content::Text { .. }))
            })
        {
            return Err(LegacyReason::Operation.into());
        }
        let mut ancestor = parents.get(index).copied().flatten();
        for _ in 0..nodes.len() {
            let Some(parent) = ancestor else { break };
            let Some(owner) = nodes.get(parent) else {
                return Err(LegacyReason::Operation.into());
            };
            if !matches!(owner.content, Content::Element { .. }) {
                return Err(LegacyReason::Operation.into());
            }
            ancestor = parents.get(parent).copied().flatten();
        }
        if ancestor.is_some() {
            return Err(LegacyReason::Operation.into());
        }
    }
    Ok(())
}
