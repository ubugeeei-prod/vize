use thiserror::Error;
use vize_relief::ReliefSnapshotNodeId;

/// Failure while lowering an owned Relief snapshot into an independent graph.
#[derive(Debug, Error)]
pub enum TemplateGraphAdapterError {
    #[error("invalid Rendu product: {0}")]
    InvalidRendu(#[from] vize_rendu::RenduValidationErrors),
    #[error("invalid Flow projection input: {0}")]
    InvalidFlow(#[from] vize_flow::FlowError),
    #[error("invalid Flow projection: {0}")]
    InvalidFlowGraph(#[from] vize_flow::ValidationErrors),
    #[error("if node references non-branch snapshot node {0:?}")]
    ExpectedIfBranch(ReliefSnapshotNodeId),
    #[error("snapshot node {0:?} does not exist")]
    MissingSnapshotNode(ReliefSnapshotNodeId),
    #[error("Relief hoist index {0} exceeds Rendu's u32 index space")]
    HoistIndexOverflow(usize),
}
