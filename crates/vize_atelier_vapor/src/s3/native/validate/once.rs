//! A small `v-once` surface: a plain element/text subtree with bound props.
//! Control flow, components and other directives retain the existing lane
//! until their one-time contracts are checked independently.

use vize_carton::ensure_sufficient_stack;

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
        check_subtree(nodes, index, true)?;
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

fn check_subtree(nodes: &[Node<'_>], index: usize, root: bool) -> Result<()> {
    ensure_sufficient_stack(|| {
        let node = nodes.get(index).ok_or(LegacyReason::Operation)?;
        match node.content {
            Content::Text { .. } if !root && node.bindings.is_empty() => return Ok(()),
            Content::Element { tag, .. } if tag != "template" => {}
            _ => return Err(LegacyReason::Operation.into()),
        }
        if node.bindings.iter().any(|binding| {
            binding.kind != BindingKind::Prop && !(root && binding.kind == BindingKind::Once)
        }) {
            return Err(LegacyReason::Operation.into());
        }
        for &child in &node.children {
            check_subtree(nodes, child, false)?;
        }
        Ok(())
    })
}
