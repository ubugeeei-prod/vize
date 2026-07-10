use crate::{
    Block, BlockId, ControlEdge, ControlEdgeId, DataEdge, DataEdgeId, Effect, EffectEdge,
    EffectEdgeId, EffectId, FlowError, FlowResult, Node, NodeId, Source, SourceId, Symbol,
    SymbolId, Value, ValueId,
};

/// Owned flow representation for one compilation unit.
///
/// Entities are stored densely and referred to by type-safe IDs. Mutation is
/// only available through checked methods, so producer bugs are reported at
/// the boundary instead of surfacing later during analysis.
#[derive(Debug, Clone)]
pub struct FlowGraph {
    pub(crate) entry: BlockId,
    pub(crate) sources: Vec<Source>,
    pub(crate) blocks: Vec<Block>,
    pub(crate) nodes: Vec<Node>,
    pub(crate) symbols: Vec<Symbol>,
    pub(crate) values: Vec<Value>,
    pub(crate) control_edges: Vec<ControlEdge>,
    pub(crate) data_edges: Vec<DataEdge>,
    pub(crate) effects: Vec<Effect>,
    pub(crate) effect_edges: Vec<EffectEdge>,
}

impl Default for FlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowGraph {
    /// Create a graph with one synthetic entry block.
    pub fn new() -> Self {
        let entry = BlockId::from_index(0);
        Self {
            entry,
            sources: Vec::new(),
            blocks: vec![Block {
                id: entry,
                provenance: crate::Provenance::Synthetic,
                nodes: Vec::new(),
                incoming: Vec::new(),
                outgoing: Vec::new(),
            }],
            nodes: Vec::new(),
            symbols: Vec::new(),
            values: Vec::new(),
            control_edges: Vec::new(),
            data_edges: Vec::new(),
            effects: Vec::new(),
            effect_edges: Vec::new(),
        }
    }

    /// Entry block used by reachability and dominance analysis.
    pub const fn entry_block(&self) -> BlockId {
        self.entry
    }

    pub fn sources(&self) -> impl ExactSizeIterator<Item = &Source> {
        self.sources.iter()
    }

    pub fn blocks(&self) -> impl ExactSizeIterator<Item = &Block> {
        self.blocks.iter()
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = &Node> {
        self.nodes.iter()
    }

    pub fn symbols(&self) -> impl ExactSizeIterator<Item = &Symbol> {
        self.symbols.iter()
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = &Value> {
        self.values.iter()
    }

    pub fn control_edges(&self) -> impl ExactSizeIterator<Item = &ControlEdge> {
        self.control_edges.iter()
    }

    pub fn data_edges(&self) -> impl ExactSizeIterator<Item = &DataEdge> {
        self.data_edges.iter()
    }

    pub fn effects(&self) -> impl ExactSizeIterator<Item = &Effect> {
        self.effects.iter()
    }

    pub fn effect_edges(&self) -> impl ExactSizeIterator<Item = &EffectEdge> {
        self.effect_edges.iter()
    }

    pub fn source(&self, id: SourceId) -> Option<&Source> {
        self.sources.get(id.index())
    }

    pub fn block(&self, id: BlockId) -> Option<&Block> {
        self.blocks.get(id.index())
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.index())
    }

    pub fn symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.index())
    }

    pub fn value(&self, id: ValueId) -> Option<&Value> {
        self.values.get(id.index())
    }

    pub fn effect(&self, id: EffectId) -> Option<&Effect> {
        self.effects.get(id.index())
    }

    pub fn control_edge(&self, id: ControlEdgeId) -> Option<&ControlEdge> {
        self.control_edges.get(id.index())
    }

    /// Look up a data edge by its graph-local ID.
    pub fn data_edge(&self, id: DataEdgeId) -> Option<&DataEdge> {
        self.data_edges.get(id.index())
    }

    /// Look up an effect edge by its graph-local ID.
    pub fn effect_edge(&self, id: EffectEdgeId) -> Option<&EffectEdge> {
        self.effect_edges.get(id.index())
    }

    /// Outgoing control edges in insertion order.
    pub fn successor_edges(
        &self,
        block: BlockId,
    ) -> FlowResult<impl ExactSizeIterator<Item = &ControlEdge>> {
        let block = self
            .blocks
            .get(block.index())
            .ok_or(FlowError::UnknownBlock(block))?;
        Ok(block
            .outgoing
            .iter()
            .map(|id| &self.control_edges[id.index()]))
    }

    /// Incoming control edges in insertion order.
    pub fn predecessor_edges(
        &self,
        block: BlockId,
    ) -> FlowResult<impl ExactSizeIterator<Item = &ControlEdge>> {
        let block = self
            .blocks
            .get(block.index())
            .ok_or(FlowError::UnknownBlock(block))?;
        Ok(block
            .incoming
            .iter()
            .map(|id| &self.control_edges[id.index()]))
    }

    /// Data edges incident to a value.
    pub fn data_edges_for_value(
        &self,
        value: ValueId,
    ) -> FlowResult<impl Iterator<Item = &DataEdge>> {
        if self.value(value).is_none() {
            return Err(FlowError::UnknownValue(value));
        }
        Ok(self
            .data_edges
            .iter()
            .filter(move |edge| edge.value == value))
    }

    /// Data edges incident to a node.
    pub fn data_edges_for_node(&self, node: NodeId) -> FlowResult<impl Iterator<Item = &DataEdge>> {
        if self.node(node).is_none() {
            return Err(FlowError::UnknownNode(node));
        }
        Ok(self.data_edges.iter().filter(move |edge| edge.node == node))
    }

    /// Effects performed by a node.
    pub fn effects_for_node(&self, node: NodeId) -> FlowResult<impl Iterator<Item = &Effect>> {
        if self.node(node).is_none() {
            return Err(FlowError::UnknownNode(node));
        }
        Ok(self
            .effects
            .iter()
            .filter(move |effect| effect.node == node))
    }
}
