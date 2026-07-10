//! Traversal over the owned Rendu graph.

use crate::{RenduNode, RenduNodeId, RenduRoot};

/// Enter/exit event for a node occurrence in render order.
#[derive(Debug, Clone, Copy)]
pub enum RenduWalkEvent<'a> {
    Enter {
        id: RenduNodeId,
        node: &'a RenduNode,
        parent: Option<RenduNodeId>,
        depth: usize,
    },
    Exit {
        id: RenduNodeId,
        node: &'a RenduNode,
        parent: Option<RenduNodeId>,
        depth: usize,
    },
}

impl RenduWalkEvent<'_> {
    pub const fn id(self) -> RenduNodeId {
        match self {
            Self::Enter { id, .. } | Self::Exit { id, .. } => id,
        }
    }

    pub const fn depth(self) -> usize {
        match self {
            Self::Enter { depth, .. } | Self::Exit { depth, .. } => depth,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Frame {
    id: RenduNodeId,
    parent: Option<RenduNodeId>,
    depth: usize,
    exiting: bool,
}

/// Iterative depth-first walker. The only allocation is its traversal stack;
/// nodes and expressions remain in the root's arenas.
pub struct RenduWalker<'a> {
    root: &'a RenduRoot,
    stack: Vec<Frame>,
}

impl<'a> RenduWalker<'a> {
    pub fn new(root: &'a RenduRoot) -> Self {
        let stack = root
            .entry()
            .iter()
            .rev()
            .copied()
            .map(|id| Frame {
                id,
                parent: None,
                depth: 0,
                exiting: false,
            })
            .collect();
        Self { root, stack }
    }

    fn push_children(&mut self, id: RenduNodeId, node: &RenduNode, depth: usize) {
        let child_depth = depth + 1;
        let mut push_group = |children: &[RenduNodeId]| {
            self.stack
                .extend(children.iter().rev().copied().map(|child| Frame {
                    id: child,
                    parent: Some(id),
                    depth: child_depth,
                    exiting: false,
                }));
        };
        match node {
            RenduNode::Fragment { children, .. }
            | RenduNode::Element { children, .. }
            | RenduNode::Component { children, .. }
            | RenduNode::SlotContent { children, .. } => push_group(children),
            RenduNode::SlotOutlet { fallback, .. } => push_group(fallback),
            RenduNode::If { branches, .. } => {
                for branch in branches.iter().rev() {
                    push_group(&branch.body);
                }
            }
            RenduNode::For { body, .. } => push_group(body),
            RenduNode::Text { .. }
            | RenduNode::Expression { .. }
            | RenduNode::Comment { .. }
            | RenduNode::HoistRef { .. } => {}
        }
    }
}

impl<'a> Iterator for RenduWalker<'a> {
    type Item = RenduWalkEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.stack.pop()?;
        let node = self
            .root
            .node(frame.id)
            .expect("RenduRoot validation guarantees walker node references");
        if frame.exiting {
            return Some(RenduWalkEvent::Exit {
                id: frame.id,
                node,
                parent: frame.parent,
                depth: frame.depth,
            });
        }
        self.stack.push(Frame {
            exiting: true,
            ..frame
        });
        self.push_children(frame.id, node, frame.depth);
        Some(RenduWalkEvent::Enter {
            id: frame.id,
            node,
            parent: frame.parent,
            depth: frame.depth,
        })
    }
}

/// Walk all enter/exit events in render order.
pub fn walk_rendu(root: &RenduRoot, mut visit: impl FnMut(RenduWalkEvent<'_>)) {
    RenduWalker::new(root).for_each(&mut visit);
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct RenduWalkSummary {
    pub node_occurrences: usize,
    pub max_depth: usize,
}

pub fn summarize_rendu(root: &RenduRoot) -> RenduWalkSummary {
    let mut summary = RenduWalkSummary::default();
    for event in RenduWalker::new(root) {
        if let RenduWalkEvent::Enter { depth, .. } = event {
            summary.node_occurrences += 1;
            summary.max_depth = summary.max_depth.max(depth);
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenduBuilder, RenduProvenance};

    #[test]
    fn walker_emits_balanced_events_and_depth() {
        let mut builder = RenduBuilder::new();
        let leaf = builder.add_node(RenduNode::Text {
            value: "leaf".into(),
            provenance: RenduProvenance::generated(),
        });
        let parent = builder.add_node(RenduNode::Fragment {
            children: vec![leaf],
            provenance: RenduProvenance::generated(),
        });
        builder.push_entry(parent);
        let root = builder.finish().expect("valid HIR");

        let events = RenduWalker::new(&root).collect::<Vec<_>>();
        assert_eq!(events.len(), 4);
        assert!(matches!(events[0], RenduWalkEvent::Enter { id, depth: 0, .. } if id == parent));
        assert!(matches!(events[1], RenduWalkEvent::Enter { id, depth: 1, .. } if id == leaf));
        assert!(matches!(events[2], RenduWalkEvent::Exit { id, depth: 1, .. } if id == leaf));
        assert!(matches!(events[3], RenduWalkEvent::Exit { id, depth: 0, .. } if id == parent));
        assert_eq!(
            summarize_rendu(&root),
            RenduWalkSummary {
                node_occurrences: 2,
                max_depth: 1,
            }
        );
    }
}
