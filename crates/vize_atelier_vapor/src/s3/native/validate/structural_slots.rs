//! A conditional or loop selects one direct authored slot template.
//! Nested carriers and implicit content beside named slots stay refused.

use super::super::{BindingKind, Content, Node};
use super::{
    Result,
    slots::{has_slot, slot_name},
};
use crate::{ir::ComponentKind, s3::LegacyReason};

fn carrier(nodes: &[Node<'_>], index: usize) -> bool {
    nodes.get(index).is_some_and(|node| {
        matches!(
            node.content,
            Content::Element { tag: "template", ref attributes, .. }
                if attributes.is_empty()
        ) && has_slot(node)
            && node
                .bindings
                .iter()
                .all(|binding| binding.kind == BindingKind::Slot)
    })
}

/// A direct carrier retains its authored element; unwrapped template controls
/// have a condition before the nested slot or a different loop-carrier start.
fn span(nodes: &[Node<'_>], index: usize) -> Option<(u32, u32)> {
    match nodes.get(index).map(|node| &node.content) {
        Some(Content::Element { tag_span, .. }) => Some(*tag_span),
        _ => None,
    }
}

fn structural(nodes: &[Node<'_>], index: usize) -> bool {
    match nodes.get(index).map(|node| &node.content) {
        Some(Content::If { branches }) => {
            branches.len() <= 64
                && branches.iter().all(|branch| {
                    matches!(branch.roots.as_slice(), [child] if carrier(nodes, *child)
                    && span(nodes, *child).is_some_and(|(start, end)| {
                        if branch.condition.is_some() {
                            start <= branch.span.0 && branch.span.1 <= end
                        } else { start == branch.span.0 }
                    }))
                })
        }
        Some(Content::For(body)) if body.key_prop.is_none() => matches!(
            nodes.get(index).map(|node| node.children.as_slice()),
            Some([child]) if carrier(nodes, *child)
                && span(nodes, *child).is_some_and(|(start, _)| start == body.carrier_start)
        ),
        _ => false,
    }
}

pub(super) fn check(nodes: &[Node<'_>], parents: &[Option<usize>]) -> Result<()> {
    for (index, node) in nodes.iter().enumerate() {
        if matches!(
            node.content,
            Content::Element {
                tag: "template",
                ..
            }
        ) {
            let parent = parents.get(index).copied().flatten();
            let direct = parent
                .and_then(|parent| nodes.get(parent))
                .is_some_and(|node| matches!(node.content, Content::Component { .. }));
            let controlled = parent.is_some_and(|parent| {
                structural(nodes, parent)
                    && parents
                        .get(parent)
                        .copied()
                        .flatten()
                        .and_then(|owner| nodes.get(owner))
                        .is_some_and(|node| {
                            matches!(
                                node.content,
                                Content::Component {
                                    kind: ComponentKind::Regular | ComponentKind::Dynamic,
                                    ..
                                }
                            )
                        })
            });
            if !carrier(nodes, index) || !(direct || controlled) {
                return Err(LegacyReason::Element.into());
            }
        }
        let Content::Component { kind, .. } = node.content else {
            continue;
        };
        let explicit = node
            .children
            .iter()
            .any(|child| carrier(nodes, *child) || structural(nodes, *child));
        if !explicit {
            continue;
        }
        if has_slot(node)
            || node
                .children
                .iter()
                .any(|child| !(carrier(nodes, *child) || structural(nodes, *child)))
        {
            return Err(LegacyReason::Component.into());
        }
        if !matches!(kind, ComponentKind::Regular | ComponentKind::Dynamic)
            && node.children.iter().any(|child| structural(nodes, *child))
        {
            return Err(LegacyReason::Component.into());
        }
        for (position, child) in node.children.iter().enumerate() {
            if let Some(name) = nodes.get(*child).and_then(slot_name)
                && node
                    .children
                    .iter()
                    .take(position)
                    .any(|previous| nodes.get(*previous).and_then(slot_name) == Some(name))
            {
                return Err(LegacyReason::Component.into());
            }
        }
    }
    Ok(())
}
