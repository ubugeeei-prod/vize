use crate::{BlockId, ControlEdgeKind, FlowGraph};

const fn traversable_from_entry(kind: ControlEdgeKind) -> bool {
    !matches!(
        kind,
        ControlEdgeKind::FunctionEntry | ControlEdgeKind::Unreachable
    )
}

/// Blocks reachable from a graph's entry along possible control edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reachability {
    reachable: Vec<bool>,
}

impl Reachability {
    /// Whether a block is reachable. Unknown IDs return `false`.
    pub fn contains(&self, block: BlockId) -> bool {
        self.reachable.get(block.index()).copied().unwrap_or(false)
    }

    /// Reachable blocks in stable ID order.
    pub fn blocks(&self) -> impl Iterator<Item = BlockId> + '_ {
        self.reachable
            .iter()
            .enumerate()
            .filter(|(_, reachable)| **reachable)
            .map(|(index, _)| BlockId::from_index(index))
    }

    /// Number of reachable blocks.
    pub fn len(&self) -> usize {
        self.reachable
            .iter()
            .filter(|reachable| **reachable)
            .count()
    }

    /// Whether no blocks are reachable. A valid [`FlowGraph`] is never empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Dominance relation for the reachable portion of a flow graph.
///
/// Dominance is intentionally undefined for unreachable blocks: queries for
/// those blocks return `false` or `None` rather than inventing a relation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dominators {
    entry: BlockId,
    reachable: Vec<bool>,
    sets: Vec<Vec<bool>>,
    immediate: Vec<Option<BlockId>>,
}

impl Dominators {
    /// Entry block used for this analysis.
    pub const fn entry(&self) -> BlockId {
        self.entry
    }

    /// Whether `dominator` dominates `block`.
    pub fn dominates(&self, dominator: BlockId, block: BlockId) -> bool {
        if !self.is_reachable(block) || !self.is_reachable(dominator) {
            return false;
        }
        self.sets
            .get(block.index())
            .and_then(|set| set.get(dominator.index()))
            .copied()
            .unwrap_or(false)
    }

    /// Immediate dominator, or `None` for the entry and unreachable blocks.
    pub fn immediate_dominator(&self, block: BlockId) -> Option<BlockId> {
        self.immediate.get(block.index()).copied().flatten()
    }

    /// Dominators of a reachable block in stable ID order.
    pub fn dominators_of(&self, block: BlockId) -> impl Iterator<Item = BlockId> + '_ {
        let reachable = self.is_reachable(block);
        self.sets
            .get(block.index())
            .into_iter()
            .flatten()
            .enumerate()
            .filter(move |(_, dominates)| **dominates && reachable)
            .map(|(index, _)| BlockId::from_index(index))
    }

    /// Whether dominance was computed for a block.
    pub fn is_reachable(&self, block: BlockId) -> bool {
        self.reachable.get(block.index()).copied().unwrap_or(false)
    }
}

impl FlowGraph {
    /// Compute entry reachability over all possible control transfers.
    pub fn reachability(&self) -> Reachability {
        let mut reachable = vec![false; self.blocks.len()];
        let mut pending = vec![self.entry];
        while let Some(block) = pending.pop() {
            if reachable[block.index()] {
                continue;
            }
            reachable[block.index()] = true;
            for edge in self.blocks[block.index()].outgoing.iter().rev() {
                let edge = &self.control_edges[edge.index()];
                if !traversable_from_entry(edge.kind) {
                    continue;
                }
                let successor = edge.to;
                if !reachable[successor.index()] {
                    pending.push(successor);
                }
            }
        }
        Reachability { reachable }
    }

    /// Reachable blocks in deterministic reverse-postorder.
    ///
    /// This is a useful consumer order for forward data-flow and type
    /// environments: a block appears after its reachable predecessors except
    /// for loop back-edges. Successors retain producer insertion order.
    pub fn reverse_postorder(&self) -> Vec<BlockId> {
        let mut visited = vec![false; self.blocks.len()];
        let mut postorder = Vec::with_capacity(self.blocks.len());
        let mut stack = vec![(self.entry, false)];
        while let Some((block, expanded)) = stack.pop() {
            if expanded {
                postorder.push(block);
                continue;
            }
            if visited[block.index()] {
                continue;
            }
            visited[block.index()] = true;
            stack.push((block, true));
            for edge in self.blocks[block.index()].outgoing.iter().rev() {
                let edge = &self.control_edges[edge.index()];
                if !traversable_from_entry(edge.kind) {
                    continue;
                }
                let successor = edge.to;
                if !visited[successor.index()] {
                    stack.push((successor, false));
                }
            }
        }
        postorder.reverse();
        postorder
    }

    /// Compute classic iterative block dominators for reachable blocks.
    pub fn dominators(&self) -> Dominators {
        let reachability = self.reachability();
        let reachable = reachability.reachable;
        let count = self.blocks.len();
        let mut sets = vec![vec![false; count]; count];

        for block in 0..count {
            if !reachable[block] {
                continue;
            }
            if block == self.entry.index() {
                sets[block][block] = true;
            } else {
                sets[block].clone_from_slice(&reachable);
            }
        }

        loop {
            let mut changed = false;
            for block_index in 0..count {
                if block_index == self.entry.index() || !reachable[block_index] {
                    continue;
                }
                let block = &self.blocks[block_index];
                let mut predecessors = block.incoming.iter().filter_map(|edge_id| {
                    let edge = &self.control_edges[edge_id.index()];
                    if !traversable_from_entry(edge.kind) {
                        return None;
                    }
                    let predecessor = edge.from.index();
                    reachable[predecessor].then_some(predecessor)
                });
                let mut next = predecessors
                    .next()
                    .map(|predecessor| sets[predecessor].clone())
                    .unwrap_or_else(|| vec![false; count]);
                for predecessor in predecessors {
                    for (candidate, dominates) in next.iter_mut().enumerate() {
                        *dominates &= sets[predecessor][candidate];
                    }
                }
                next[block_index] = true;
                if next != sets[block_index] {
                    sets[block_index] = next;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let immediate = immediate_dominators(self.entry, &reachable, &sets);
        Dominators {
            entry: self.entry,
            reachable,
            sets,
            immediate,
        }
    }
}

fn immediate_dominators(
    entry: BlockId,
    reachable: &[bool],
    sets: &[Vec<bool>],
) -> Vec<Option<BlockId>> {
    let mut immediate = vec![None; sets.len()];
    for block in 0..sets.len() {
        if block == entry.index() || !reachable[block] {
            continue;
        }
        let strict: Vec<_> = sets[block]
            .iter()
            .enumerate()
            .filter(|(candidate, dominates)| **dominates && *candidate != block)
            .map(|(candidate, _)| candidate)
            .collect();
        let candidate = strict.iter().copied().find(|candidate| {
            strict
                .iter()
                .all(|other| other == candidate || sets[*candidate][*other])
        });
        immediate[block] = candidate.map(BlockId::from_index);
    }
    immediate
}
