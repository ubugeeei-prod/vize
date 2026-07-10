//! Reactive effect graph: dependencies between `computed`, `watch`, and refs.
//!
//! Foundation for issue #695. Cyclic `computed` chains (`a → b → a`) lock up
//! Vue's reactive update loop. This module ships the graph model so the
//! analyzer can detect cycles and report them as warnings.
//!
//! The actual cycle-detection pass and the Patina rule that surfaces it are
//! follow-ups. The intent of landing the model now is so the analyzer and
//! the lint rule can be developed against a stable shape.

mod builder;
mod cycles;

pub use builder::{
    EffectGraphScript, build_effect_graph_from_script, build_effect_graph_from_script_setup,
    build_effect_graph_from_sfc_scripts,
};

use serde::Serialize;
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

/// Stable counters for one reactive effect graph.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectGraphSummary {
    pub node_count: usize,
    pub edge_count: usize,
    pub cycle_count: usize,
    pub cycle_node_count: usize,
}

impl EffectGraphSummary {
    /// Combine summaries for effect graphs known to be independent.
    ///
    /// Counts use saturating addition so aggregation remains conservative for
    /// arbitrarily large generated inputs. The operation is deterministic and
    /// order-independent.
    pub fn merged(self, other: Self) -> Self {
        Self {
            node_count: self.node_count.saturating_add(other.node_count),
            edge_count: self.edge_count.saturating_add(other.edge_count),
            cycle_count: self.cycle_count.saturating_add(other.cycle_count),
            cycle_node_count: self.cycle_node_count.saturating_add(other.cycle_node_count),
        }
    }
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

    /// Count unique nodes that appear in dependency edges.
    pub fn node_count(&self) -> usize {
        let mut nodes = std::collections::BTreeSet::new();
        for edge in &self.edges {
            nodes.insert(&edge.from);
            nodes.insert(&edge.to);
        }
        nodes.len()
    }

    /// Summarize edge and cycle counts for reports and diagnostics.
    pub fn summary(&self) -> EffectGraphSummary {
        let (cycle_count, cycle_node_count) = cycles::cycle_summary(&self.edges);

        EffectGraphSummary {
            node_count: self.node_count(),
            edge_count: self.edges.len(),
            cycle_count,
            cycle_node_count,
        }
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
    use super::{EffectGraph, EffectGraphSummary};

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

        let summary = g.summary();
        assert_eq!(summary.node_count, 2);
        assert_eq!(summary.edge_count, 2);
        assert_eq!(summary.cycle_count, 1);
        assert_eq!(summary.cycle_node_count, 2);

        let json = serde_json::to_string(&summary).unwrap();
        assert_eq!(
            json,
            r#"{"nodeCount":2,"edgeCount":2,"cycleCount":1,"cycleNodeCount":2}"#
        );
    }

    #[test]
    fn no_cycle_in_dag() {
        let mut g = EffectGraph::default();
        g.add_edge("a", "b");
        g.add_edge("b", "c");
        g.add_edge("a", "c");
        assert!(g.find_cycle().is_none());
        assert_eq!(g.summary().node_count, 3);
        assert_eq!(g.summary().cycle_count, 0);
    }

    #[test]
    fn add_edge_is_idempotent() {
        let mut g = EffectGraph::default();
        g.add_edge("a", "b");
        g.add_edge("a", "b");
        assert_eq!(g.edges().count(), 1);
    }

    #[test]
    fn summary_counts_each_cyclic_component_and_self_loop() {
        let mut g = EffectGraph::default();
        for (from, to) in [("a", "b"), ("b", "a"), ("b", "c"), ("c", "d"), ("d", "c")] {
            g.add_edge(from, to);
        }
        assert_eq!(g.summary().cycle_count, 2);
        assert_eq!(g.summary().cycle_node_count, 4);

        let mut self_loop = EffectGraph::default();
        self_loop.add_edge("self", "self");
        assert_eq!(self_loop.summary().cycle_count, 1);
        assert_eq!(self_loop.summary().cycle_node_count, 1);
    }

    #[test]
    fn summary_merge_is_order_independent_and_saturating() {
        let script = EffectGraphSummary {
            node_count: 2,
            edge_count: 3,
            cycle_count: 1,
            cycle_node_count: 2,
        };
        let setup = EffectGraphSummary {
            node_count: 4,
            edge_count: 5,
            cycle_count: 1,
            cycle_node_count: 3,
        };
        let expected = EffectGraphSummary {
            node_count: 6,
            edge_count: 8,
            cycle_count: 2,
            cycle_node_count: 5,
        };

        assert_eq!(script.merged(setup), expected);
        assert_eq!(setup.merged(script), expected);
        assert_eq!(script.merged(EffectGraphSummary::default()), script);
        assert_eq!(EffectGraphSummary::default().merged(script), script);
        assert_eq!(
            script.merged(setup).merged(EffectGraphSummary {
                node_count: 1,
                ..EffectGraphSummary::default()
            }),
            script.merged(setup.merged(EffectGraphSummary {
                node_count: 1,
                ..EffectGraphSummary::default()
            }))
        );
        assert_eq!(
            expected.merged(EffectGraphSummary {
                node_count: usize::MAX,
                edge_count: usize::MAX,
                cycle_count: usize::MAX,
                cycle_node_count: usize::MAX,
            }),
            EffectGraphSummary {
                node_count: usize::MAX,
                edge_count: usize::MAX,
                cycle_count: usize::MAX,
                cycle_node_count: usize::MAX,
            }
        );
    }
}
