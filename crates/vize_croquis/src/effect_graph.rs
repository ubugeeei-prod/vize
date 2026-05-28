//! Reactive effect graph: dependencies between `computed`, `watch`, and refs.
//!
//! Foundation for issue #695. Cyclic `computed` chains (`a → b → a`) lock up
//! Vue's reactive update loop. This module ships the graph model so the
//! analyzer can detect cycles and report them as warnings.
//!
//! The actual cycle-detection pass and the Patina rule that surfaces it are
//! follow-ups. The intent of landing the model now is so the analyzer and
//! the lint rule can be developed against a stable shape.

use vize_carton::CompactString;

/// A node in the effect graph — usually a reactive binding name.
pub type EffectNodeId = CompactString;

/// One reactive dependency: `from` reads `to` during evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectEdge {
    pub from: EffectNodeId,
    pub to: EffectNodeId,
}

/// Effect graph built from one SFC's `computed` getters and `watch` source
/// expressions. Workspace-wide cycles need merging across files; that
/// follow-up reuses this shape.
#[derive(Debug, Default, Clone)]
pub struct EffectGraph {
    edges: Vec<EffectEdge>,
}

impl EffectGraph {
    /// Add a `from → to` dependency edge.
    pub fn add_edge(&mut self, from: impl Into<EffectNodeId>, to: impl Into<EffectNodeId>) {
        let edge = EffectEdge {
            from: from.into(),
            to: to.into(),
        };
        if !self.edges.contains(&edge) {
            self.edges.push(edge);
        }
    }

    /// Iterate over all dependency edges.
    pub fn edges(&self) -> impl Iterator<Item = &EffectEdge> {
        self.edges.iter()
    }

    /// Detect the first cycle reachable from any node, returned as the chain
    /// of node ids in traversal order. `None` when no cycle exists.
    ///
    /// Tarjan-style strongly-connected-components would scale better for
    /// dense graphs, but reactive graphs in typical SFCs are tiny (<100
    /// nodes), and a DFS with recursion-stack tracking is simpler and
    /// produces an actionable chain.
    pub fn find_cycle(&self) -> Option<Vec<EffectNodeId>> {
        let nodes: Vec<&EffectNodeId> = {
            let mut seen = std::collections::BTreeSet::new();
            for edge in &self.edges {
                seen.insert(&edge.from);
                seen.insert(&edge.to);
            }
            seen.into_iter().collect()
        };

        let mut on_stack = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        for start in nodes {
            if visited.contains(start) {
                continue;
            }
            let mut stack: Vec<(EffectNodeId, usize)> = vec![(start.clone(), 0)];
            on_stack.clear();
            on_stack.insert(start.clone());
            while let Some((node, idx)) = stack.last().cloned() {
                let next_edge = self.edges.iter().filter(|e| e.from == node).nth(idx);
                let Some(edge) = next_edge else {
                    stack.pop();
                    on_stack.remove(&node);
                    visited.insert(node);
                    continue;
                };
                // Advance the iterator pointer for the current node.
                if let Some(last) = stack.last_mut() {
                    last.1 += 1;
                }
                if on_stack.contains(&edge.to) {
                    // Cycle found — assemble the chain from the recursion
                    // stack starting at the first occurrence of `edge.to`.
                    let cycle_start = stack
                        .iter()
                        .position(|(name, _)| *name == edge.to)
                        .unwrap_or(0);
                    let mut chain: Vec<EffectNodeId> = stack[cycle_start..]
                        .iter()
                        .map(|(name, _)| name.clone())
                        .collect();
                    chain.push(edge.to.clone());
                    return Some(chain);
                }
                if !visited.contains(&edge.to) {
                    stack.push((edge.to.clone(), 0));
                    on_stack.insert(edge.to.clone());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::EffectGraph;

    #[test]
    fn detects_two_node_cycle() {
        let mut g = EffectGraph::default();
        g.add_edge("a", "b");
        g.add_edge("b", "a");
        let cycle = g.find_cycle().expect("expected cycle");
        // Cycle starts and ends at the same node.
        assert_eq!(cycle.first(), cycle.last());
        assert!(cycle.contains(&"a".into()));
        assert!(cycle.contains(&"b".into()));
    }

    #[test]
    fn no_cycle_in_dag() {
        let mut g = EffectGraph::default();
        g.add_edge("a", "b");
        g.add_edge("b", "c");
        g.add_edge("a", "c");
        assert!(g.find_cycle().is_none());
    }

    #[test]
    fn add_edge_is_idempotent() {
        let mut g = EffectGraph::default();
        g.add_edge("a", "b");
        g.add_edge("a", "b");
        assert_eq!(g.edges().count(), 1);
    }
}
