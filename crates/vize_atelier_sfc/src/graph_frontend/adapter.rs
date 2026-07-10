use vize_flow::FlowGraph;
use vize_relief::ReliefSnapshot;
use vize_rendu::RenduRoot;

use super::{
    SfcGraphAdapterError, lower_relief_snapshot_to_rendu, project_relief_snapshot_to_flow,
};

/// Zero-allocation view exposing independent products for one cached snapshot.
#[derive(Debug, Clone, Copy)]
pub struct SfcTemplateGraphAdapter<'a> {
    snapshot: &'a ReliefSnapshot,
}

impl<'a> SfcTemplateGraphAdapter<'a> {
    pub const fn new(snapshot: &'a ReliefSnapshot) -> Self {
        Self { snapshot }
    }

    /// Cached syntax product without constructing any downstream graph.
    pub const fn snapshot(self) -> &'a ReliefSnapshot {
        self.snapshot
    }

    /// Explicitly lower this syntax snapshot into render HIR.
    pub fn lower_rendu(self) -> Result<RenduRoot, SfcGraphAdapterError> {
        lower_relief_snapshot_to_rendu(self.snapshot)
    }

    /// Explicitly project this syntax snapshot into control/data/effect flow.
    pub fn project_flow(self) -> Result<FlowGraph, SfcGraphAdapterError> {
        project_relief_snapshot_to_flow(self.snapshot)
    }
}
