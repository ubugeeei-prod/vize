//! Atlas identity for a single-compilation-unit flow graph.

use vize_atlas::Product;

use crate::FlowGraph;

/// Demandable control/data/effect graph produced by a frontend adapter.
pub struct FlowProduct;

impl Product for FlowProduct {
    type Value = FlowGraph;

    const NAME: &'static str = "flow.graph";
}
