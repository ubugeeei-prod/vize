use crate::{
    Block, BlockId, ControlEdge, ControlEdgeId, ControlEdgeKind, DataEdge, DataEdgeId,
    DataEdgeKind, DataUseKind, Effect, EffectEdge, EffectEdgeId, EffectEdgeKind, EffectId,
    EffectKind, FlowError, FlowGraph, FlowResult, Node, NodeId, NodeKind, Provenance, Source,
    SourceId, Symbol, SymbolId, Value, ValueId,
};
use vize_carton::String;
use vize_carton::source_anchor::SourceAnchor;

impl FlowGraph {
    /// Register a logical source and return its graph-local ID.
    pub fn add_source(&mut self, name: impl Into<String>) -> FlowResult<SourceId> {
        self.add_source_inner(name.into(), None)
    }

    /// Register a logical source tied to a stable compilation source revision.
    pub fn add_source_with_anchor(
        &mut self,
        name: impl Into<String>,
        anchor: SourceAnchor,
    ) -> FlowResult<SourceId> {
        self.add_source_inner(name.into(), Some(anchor))
    }

    fn add_source_inner(
        &mut self,
        name: String,
        anchor: Option<SourceAnchor>,
    ) -> FlowResult<SourceId> {
        ensure_capacity(self.sources.len(), "source")?;
        let id = SourceId::from_index(self.sources.len());
        self.sources.push(Source { id, name, anchor });
        Ok(id)
    }

    /// Change the source attribution of an existing block.
    pub fn set_block_provenance(
        &mut self,
        block: BlockId,
        provenance: Provenance,
    ) -> FlowResult<()> {
        self.check_provenance(provenance)?;
        let target = self
            .blocks
            .get_mut(block.index())
            .ok_or(FlowError::UnknownBlock(block))?;
        target.provenance = provenance;
        Ok(())
    }

    /// Add a basic block. It is unreachable until connected by control edges.
    pub fn add_block(&mut self, provenance: Provenance) -> FlowResult<BlockId> {
        self.check_provenance(provenance)?;
        ensure_capacity(self.blocks.len(), "block")?;
        let id = BlockId::from_index(self.blocks.len());
        self.blocks.push(Block {
            id,
            provenance,
            nodes: Vec::new(),
            incoming: Vec::new(),
            outgoing: Vec::new(),
        });
        Ok(id)
    }

    /// Append an operation to a block.
    pub fn add_node(
        &mut self,
        block: BlockId,
        kind: NodeKind,
        provenance: Provenance,
    ) -> FlowResult<NodeId> {
        self.check_block(block)?;
        self.check_provenance(provenance)?;
        ensure_capacity(self.nodes.len(), "node")?;
        let id = NodeId::from_index(self.nodes.len());
        self.nodes.push(Node {
            id,
            block,
            kind,
            provenance,
        });
        self.blocks[block.index()].nodes.push(id);
        Ok(id)
    }

    /// Connect two blocks with a possible control transfer.
    pub fn add_control_edge(
        &mut self,
        from: BlockId,
        to: BlockId,
        kind: ControlEdgeKind,
        provenance: Provenance,
    ) -> FlowResult<ControlEdgeId> {
        self.check_block(from)?;
        self.check_block(to)?;
        self.check_provenance(provenance)?;
        ensure_capacity(self.control_edges.len(), "control edge")?;
        let id = ControlEdgeId::from_index(self.control_edges.len());
        self.control_edges.push(ControlEdge {
            id,
            from,
            to,
            kind,
            provenance,
        });
        self.blocks[from.index()].outgoing.push(id);
        self.blocks[to.index()].incoming.push(id);
        Ok(id)
    }

    /// Register a frontend-independent symbol identity.
    pub fn add_symbol(&mut self, provenance: Provenance) -> FlowResult<SymbolId> {
        self.check_provenance(provenance)?;
        ensure_capacity(self.symbols.len(), "symbol")?;
        let id = SymbolId::from_index(self.symbols.len());
        self.symbols.push(Symbol { id, provenance });
        Ok(id)
    }

    /// Create a value defined by a node.
    ///
    /// A matching [`DataEdgeKind::Definition`] edge is inserted atomically.
    pub fn define_value(
        &mut self,
        definition: NodeId,
        symbol: Option<SymbolId>,
        provenance: Provenance,
    ) -> FlowResult<ValueId> {
        self.check_node(definition)?;
        self.check_symbol(symbol)?;
        self.check_provenance(provenance)?;
        ensure_capacity(self.values.len(), "value")?;
        ensure_capacity(self.data_edges.len(), "data edge")?;
        let id = ValueId::from_index(self.values.len());
        self.values.push(Value {
            id,
            definition: Some(definition),
            symbol,
            provenance,
        });
        self.push_data_edge(id, definition, DataEdgeKind::Definition, provenance);
        Ok(id)
    }

    /// Create a parameter, import, or other value without a local definition.
    pub fn add_external_value(
        &mut self,
        symbol: Option<SymbolId>,
        provenance: Provenance,
    ) -> FlowResult<ValueId> {
        self.check_symbol(symbol)?;
        self.check_provenance(provenance)?;
        ensure_capacity(self.values.len(), "value")?;
        let id = ValueId::from_index(self.values.len());
        self.values.push(Value {
            id,
            definition: None,
            symbol,
            provenance,
        });
        Ok(id)
    }

    /// Record a use, phi input, mutation, or capture of a value.
    pub fn add_data_use(
        &mut self,
        value: ValueId,
        node: NodeId,
        kind: DataUseKind,
        provenance: Provenance,
    ) -> FlowResult<DataEdgeId> {
        self.check_value(value)?;
        self.check_node(node)?;
        self.check_provenance(provenance)?;
        ensure_capacity(self.data_edges.len(), "data edge")?;
        Ok(self.push_data_edge(value, node, kind.into(), provenance))
    }

    /// Record an observable or ordering-relevant effect at a node.
    pub fn add_effect(
        &mut self,
        node: NodeId,
        kind: EffectKind,
        symbol: Option<SymbolId>,
        provenance: Provenance,
    ) -> FlowResult<EffectId> {
        self.check_node(node)?;
        self.check_symbol(symbol)?;
        self.check_provenance(provenance)?;
        ensure_capacity(self.effects.len(), "effect")?;
        let id = EffectId::from_index(self.effects.len());
        self.effects.push(Effect {
            id,
            node,
            kind,
            symbol,
            provenance,
        });
        Ok(id)
    }

    /// Relate two effects by ordering, dependency, or conflict.
    pub fn add_effect_edge(
        &mut self,
        from: EffectId,
        to: EffectId,
        kind: EffectEdgeKind,
        provenance: Provenance,
    ) -> FlowResult<EffectEdgeId> {
        self.check_effect(from)?;
        self.check_effect(to)?;
        self.check_provenance(provenance)?;
        ensure_capacity(self.effect_edges.len(), "effect edge")?;
        let id = EffectEdgeId::from_index(self.effect_edges.len());
        self.effect_edges.push(EffectEdge {
            id,
            from,
            to,
            kind,
            provenance,
        });
        Ok(id)
    }

    fn push_data_edge(
        &mut self,
        value: ValueId,
        node: NodeId,
        kind: DataEdgeKind,
        provenance: Provenance,
    ) -> DataEdgeId {
        let id = DataEdgeId::from_index(self.data_edges.len());
        self.data_edges.push(DataEdge {
            id,
            value,
            node,
            kind,
            provenance,
        });
        id
    }

    fn check_provenance(&self, provenance: Provenance) -> FlowResult<()> {
        if let Some(span) = provenance.span()
            && self.source(span.source()).is_none()
        {
            return Err(FlowError::UnknownSource(span.source()));
        }
        Ok(())
    }

    fn check_block(&self, id: BlockId) -> FlowResult<()> {
        self.block(id)
            .map(|_| ())
            .ok_or(FlowError::UnknownBlock(id))
    }

    fn check_node(&self, id: NodeId) -> FlowResult<()> {
        self.node(id).map(|_| ()).ok_or(FlowError::UnknownNode(id))
    }

    fn check_value(&self, id: ValueId) -> FlowResult<()> {
        self.value(id)
            .map(|_| ())
            .ok_or(FlowError::UnknownValue(id))
    }

    fn check_symbol(&self, id: Option<SymbolId>) -> FlowResult<()> {
        match id {
            Some(id) if self.symbol(id).is_none() => Err(FlowError::UnknownSymbol(id)),
            _ => Ok(()),
        }
    }

    fn check_effect(&self, id: EffectId) -> FlowResult<()> {
        self.effect(id)
            .map(|_| ())
            .ok_or(FlowError::UnknownEffect(id))
    }
}

fn ensure_capacity(length: usize, kind: &'static str) -> FlowResult<()> {
    if u32::try_from(length).is_err() {
        Err(FlowError::CapacityExceeded(kind))
    } else {
        Ok(())
    }
}
