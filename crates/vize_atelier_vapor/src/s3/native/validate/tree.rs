//! Region ownership becomes the native tree: element children, branch roots,
//! and loop bodies. Every owner is accounted for exactly once.

use vize_s3::op::Program;

use super::super::{Content, Node};
use super::{Result, Slots};
use crate::s3::{AdmissionFailure, LegacyReason};

/// Returns each node's owning node. Branch roots and loop bodies are owned by
/// their control node, whose body starts a separate template. Owned regions
/// move their members into the owner; the unowned root region stays.
pub(super) fn assemble(
    program: &Program<'_>,
    nodes: &mut [Node<'_>],
    slots: &Slots,
    regions: &mut [std::vec::Vec<usize>],
) -> Result<std::vec::Vec<Option<usize>>> {
    let mut owned = std::vec![false; nodes.len()];
    let mut parents = std::vec![None; nodes.len()];
    for region in &program.regions {
        let Some(owner) = region.owner else { continue };
        let Some(&Some((index, _))) = slots.get(owner.index() as usize) else {
            return Err(LegacyReason::Structure.into());
        };
        let children = regions
            .get_mut(region.id.index() as usize)
            .map(std::mem::take)
            .unwrap_or_default();
        if children.iter().any(|child| *child <= index) {
            return Err(LegacyReason::Structure.into());
        }
        // Branches and loop bodies render one element in this slice. Template
        // fragments, text roots and directly nested control flow stay legacy.
        let body = match children.as_slice() {
            [child] if matches!(nodes[*child].content, Content::Element { .. }) => Some(*child),
            _ => None,
        };
        match &mut nodes[index].content {
            Content::Element { tag, .. } => {
                if std::mem::replace(&mut owned[index], true)
                    || vize_carton::is_void_tag(tag) && !children.is_empty()
                {
                    return Err(LegacyReason::Structure.into());
                }
            }
            Content::If { branches } => {
                let branch = branches
                    .iter_mut()
                    .find(|branch| branch.region == region.id)
                    .ok_or(AdmissionFailure::Invalid(
                        "branch region lacks its condition",
                    ))?;
                branch.root = Some(body.ok_or(LegacyReason::ControlFlow)?);
            }
            Content::For(_) => {
                if std::mem::replace(&mut owned[index], true) {
                    return Err(AdmissionFailure::Invalid("loop owns several bodies"));
                }
                body.ok_or(LegacyReason::ControlFlow)?;
            }
            // Slot content and fallbacks are fragments rendered by their own block.
            Content::Component { .. } | Content::Outlet { .. } => {
                if std::mem::replace(&mut owned[index], true) {
                    return Err(AdmissionFailure::Invalid("slot owner has several regions"));
                }
            }
            Content::Text { .. } => return Err(LegacyReason::Structure.into()),
        }
        for child in &children {
            parents[*child] = Some(index);
        }
        if !matches!(nodes[index].content, Content::If { .. }) {
            nodes[index].children = children;
        }
    }
    for (node, owned) in nodes.iter().zip(owned) {
        match &node.content {
            Content::Element { .. } | Content::Component { .. } | Content::Outlet { .. }
                if !owned =>
            {
                return Err(LegacyReason::Structure.into());
            }
            Content::For(_) if !owned => {
                return Err(AdmissionFailure::Invalid("loop lacks its body"));
            }
            Content::If { branches } if branches.iter().any(|branch| branch.root.is_none()) => {
                return Err(AdmissionFailure::Invalid("branch lacks its region"));
            }
            _ => {}
        }
    }
    Ok(parents)
}

/// Authored element depth the native lane admits, far below the legacy
/// parser's 4096-element flattening limit (which it reports).
const MAX_DEPTH: u32 = 1024;

/// HTML tree construction closes an earlier button or list item instead of
/// nesting a new one. The legacy parser reads authored markup, so the guard
/// follows authored nesting, not template strings: any open button counts,
/// and an open list item counts until a list or a component bounds its scope.
/// Branches and loops are transparent. Parents precede their children.
pub(super) fn check_nesting(nodes: &[Node<'_>], parents: &[Option<usize>]) -> Result<()> {
    let mut open = std::vec![(false, false, 0_u32); nodes.len()];
    for (index, node) in nodes.iter().enumerate() {
        let inherited = parents[index].map_or((false, false, 0), |parent| {
            let (button, item, depth) = open[parent];
            let scoped = !matches!(
                nodes[parent].content,
                Content::Element {
                    tag: "ul" | "ol",
                    ..
                } | Content::Component { .. }
                    | Content::Outlet { .. }
            );
            (button, item && scoped, depth)
        });
        let (button, item) = match node.content {
            Content::Element { tag: "button", .. } => (true, false),
            Content::Element { tag: "li", .. } => (false, true),
            _ => (false, false),
        };
        // Branches and loops ride on their body element's tag.
        let element = matches!(
            node.content,
            Content::Element { .. } | Content::Component { .. } | Content::Outlet { .. }
        );
        let depth = inherited.2 + u32::from(element);
        if inherited.0 && button || inherited.1 && item || depth > MAX_DEPTH {
            return Err(LegacyReason::Structure.into());
        }
        open[index] = (inherited.0 || button, inherited.1 || item, depth);
    }
    Ok(())
}
