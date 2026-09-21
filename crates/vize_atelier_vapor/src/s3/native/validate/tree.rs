//! Region ownership becomes the native tree: element children, branch roots,
//! and loop bodies. Every owner is accounted for exactly once.

use vize_carton::{FxHashMap, FxHashSet};
use vize_s3::op::{OpId, Program, RegionId};

use super::super::{Content, Node};
use super::Result;
use crate::s3::{AdmissionFailure, LegacyReason};

/// Returns each node's owning node. Branch roots and loop bodies are owned by
/// their control node, whose body starts a separate template.
pub(super) fn assemble(
    program: &Program<'_>,
    nodes: &mut [Node<'_>],
    indexes: &FxHashMap<OpId, (usize, RegionId)>,
    regions: &FxHashMap<RegionId, std::vec::Vec<OpId>>,
) -> Result<std::vec::Vec<Option<usize>>> {
    let mut owners = FxHashSet::default();
    let mut parents = std::vec![None; nodes.len()];
    for region in &program.regions {
        let Some(owner) = region.owner else { continue };
        let Some(&(index, _)) = indexes.get(&owner) else {
            return Err(LegacyReason::Structure.into());
        };
        let children: std::vec::Vec<_> = regions
            .get(&region.id)
            .into_iter()
            .flatten()
            .map(|id| indexes[id].0)
            .collect();
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
                if !owners.insert(owner) || vize_carton::is_void_tag(tag) && !children.is_empty() {
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
                if !owners.insert(owner) {
                    return Err(AdmissionFailure::Invalid("loop owns several bodies"));
                }
                body.ok_or(LegacyReason::ControlFlow)?;
            }
            // Slot content and fallbacks are fragments rendered by their own block.
            Content::Component { .. } | Content::Outlet { .. } => {
                if !owners.insert(owner) {
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
    for (id, (index, _)) in indexes {
        match &nodes[*index].content {
            Content::Element { .. } | Content::Component { .. } | Content::Outlet { .. }
                if !owners.contains(id) =>
            {
                return Err(LegacyReason::Structure.into());
            }
            Content::For(_) if !owners.contains(id) => {
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

/// HTML parsing closes an earlier button or list item instead of nesting a
/// new one. The guard applies within one template string; a control body is
/// instantiated from its own template and inserted through DOM operations.
pub(super) fn check_nesting(nodes: &[Node<'_>], parents: &[Option<usize>]) -> Result<()> {
    let mut open = std::vec![(false, false); nodes.len()];
    for (index, node) in nodes.iter().enumerate() {
        let inherited = parents[index]
            .filter(|parent| matches!(nodes[*parent].content, Content::Element { .. }))
            .map_or((false, false), |parent| open[parent]);
        let (button, item) = match node.content {
            Content::Element { tag: "button", .. } => (true, false),
            Content::Element { tag: "li", .. } => (false, true),
            _ => (false, false),
        };
        if inherited.0 && button || inherited.1 && item {
            return Err(LegacyReason::Structure.into());
        }
        open[index] = (inherited.0 || button, inherited.1 || item);
    }
    Ok(())
}
