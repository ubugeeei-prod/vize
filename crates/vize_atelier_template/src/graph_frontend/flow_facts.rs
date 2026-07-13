use vize_carton::{FxHashMap, String};
use vize_flow::{
    BlockId, DataUseKind, EffectKind, FlowGraph, FlowResult, NodeId, NodeKind, SourceId, SymbolId,
    ValueId,
};
use vize_relief::{SnapshotExpression, SourceLocation};

use super::{expression::expression_code, provenance::flow_provenance};

#[derive(Default)]
pub(super) struct FlowFacts {
    values: FxHashMap<String, (SymbolId, ValueId)>,
}

impl FlowFacts {
    pub(super) fn add_code_use(
        &mut self,
        graph: &mut FlowGraph,
        source: SourceId,
        block: BlockId,
        code: String,
        location: &SourceLocation,
    ) -> FlowResult<NodeId> {
        let provenance = flow_provenance(location, source);
        let node = graph.add_node(block, NodeKind::Operation, provenance)?;
        let (symbol, value) = if let Some(pair) = self.values.get(&code).copied() {
            pair
        } else {
            let symbol = graph.add_symbol(provenance)?;
            let value = graph.add_external_value(Some(symbol), provenance)?;
            self.values.insert(code, (symbol, value));
            (symbol, value)
        };
        graph.add_data_use(value, node, DataUseKind::Use, provenance)?;
        graph.add_effect(node, EffectKind::Read, Some(symbol), provenance)?;
        Ok(node)
    }

    pub(super) fn add_binding_definition(
        &mut self,
        graph: &mut FlowGraph,
        source: SourceId,
        block: BlockId,
        binding: &SnapshotExpression,
    ) -> FlowResult<()> {
        let code = expression_code(binding);
        let provenance = flow_provenance(binding.location(), source);
        let node = graph.add_node(block, NodeKind::Operation, provenance)?;
        let symbol = graph.add_symbol(provenance)?;
        let value = graph.define_value(node, Some(symbol), provenance)?;
        self.values.insert(code, (symbol, value));
        Ok(())
    }
}
