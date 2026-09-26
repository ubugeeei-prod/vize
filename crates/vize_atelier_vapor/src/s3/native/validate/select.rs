//! A bounded select parsing contract: model-bound selects with flat options.

use super::super::{BindingKind, Content, Node};
use super::{Result, at};
use crate::s3::LegacyReason;

pub(super) fn check(nodes: &[Node<'_>], parents: &[Option<usize>]) -> Result<()> {
    for (index, node) in nodes.iter().enumerate() {
        match &node.content {
            Content::Element { tag: "select", .. } => {
                if !node
                    .bindings
                    .iter()
                    .any(|binding| binding.kind == BindingKind::Model)
                    || node.bindings.iter().any(|binding| {
                        matches!(binding.kind, BindingKind::Html | BindingKind::Text)
                    })
                {
                    return Err(LegacyReason::Binding.into());
                }
                for child in &node.children {
                    if !matches!(
                        at(nodes, *child)?.content,
                        Content::Element { tag: "option", .. }
                    ) {
                        return Err(LegacyReason::Element.into());
                    }
                }
            }
            Content::Element { tag: "option", .. } => {
                let parent = at(parents, index)?.ok_or(LegacyReason::Element)?;
                if !matches!(
                    at(nodes, parent)?.content,
                    Content::Element { tag: "select", .. }
                ) || node
                    .bindings
                    .iter()
                    .any(|binding| binding.kind == BindingKind::Html)
                {
                    return Err(LegacyReason::Element.into());
                }
                for child in &node.children {
                    if !matches!(at(nodes, *child)?.content, Content::Text { .. }) {
                        return Err(LegacyReason::Element.into());
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
