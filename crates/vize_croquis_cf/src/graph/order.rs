//! Preserve the incoming order previously reached by a second finalization.

use super::{DependencyEdge, DependencyGraph, SmallVec};

#[cfg(test)]
mod tests;

impl DependencyGraph {
    /// Rebuilding component edges moves them behind every retained import.
    /// Stable-partition just this boundary so one finalization has the same
    /// incoming vectors as repeated finalization, without resolving again.
    pub(crate) fn finalize_importer_order(&mut self) {
        let profiler = vize_l0::profiler::global_profiler();
        let _phase = profiler.global_span("croquis_cf.dependencies.order_incoming");
        let mut components = SmallVec::<[_; 8]>::new();
        let mut partitioned = 0;
        for node in self.nodes.values_mut() {
            let mut seen_component = false;
            let interleaved = node.importers.iter().any(|(_, kind)| {
                if *kind == DependencyEdge::ComponentUsage {
                    seen_component = true;
                    false
                } else {
                    seen_component
                }
            });
            if !interleaved {
                continue;
            }
            node.importers.retain(|edge| {
                if edge.1 == DependencyEdge::ComponentUsage {
                    components.push(*edge);
                    false
                } else {
                    true
                }
            });
            node.importers.extend(components.drain(..));
            partitioned += 1;
        }
        profiler.record_counter(
            "croquis_cf.dependencies.order_incoming.vectors_visited",
            self.nodes.len() as u64,
        );
        profiler.record_counter(
            "croquis_cf.dependencies.order_incoming.vectors_partitioned",
            partitioned,
        );
    }
}
