use super::{ReliefSnapshot, ReliefSnapshotNode, ReliefSnapshotNodeId};

/// One node yielded by a source-order snapshot traversal.
#[derive(Debug, Clone, Copy)]
pub struct ReliefSnapshotVisit<'a> {
    pub id: ReliefSnapshotNodeId,
    pub depth: usize,
    pub node: &'a ReliefSnapshotNode,
}

/// Non-recursive depth-first traversal over an owned Relief snapshot.
pub struct ReliefSnapshotWalker<'a> {
    snapshot: &'a ReliefSnapshot,
    pending: Vec<(ReliefSnapshotNodeId, usize)>,
}

impl<'a> ReliefSnapshotWalker<'a> {
    pub(crate) fn from_roots(snapshot: &'a ReliefSnapshot, roots: &[ReliefSnapshotNodeId]) -> Self {
        let pending = roots.iter().rev().map(|id| (*id, 0)).collect();
        Self { snapshot, pending }
    }

    pub(crate) fn from_one(snapshot: &'a ReliefSnapshot, root: ReliefSnapshotNodeId) -> Self {
        Self {
            snapshot,
            pending: vec![(root, 0)],
        }
    }

    pub(crate) fn empty(snapshot: &'a ReliefSnapshot) -> Self {
        Self {
            snapshot,
            pending: Vec::new(),
        }
    }
}

impl<'a> Iterator for ReliefSnapshotWalker<'a> {
    type Item = ReliefSnapshotVisit<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (id, depth) = self.pending.pop()?;
        let node = self.snapshot.node(id)?;
        self.pending.extend(
            node.children()
                .iter()
                .rev()
                .map(|child| (*child, depth + 1)),
        );
        Some(ReliefSnapshotVisit { id, depth, node })
    }
}
