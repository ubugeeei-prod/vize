use vize_l0::id::NodeId;

/// A canonical walk failed to yield complete, correctly nested decisions.
/// Partial scratch tables are never returned as completed analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionBuildError {
    /// Script/template failure or interrupted admission is retained by the file.
    IncompleteFile,
    /// Source custody is complete, but no explicit prepared-text output profile
    /// has been established for this original File.
    PreparedTextProfile,
    /// The actual native attribute event did not consume its own whole value row.
    OriginalAttributeValue { node: Option<NodeId>, slot: usize },
    /// The shared L2 walk exhausted its stage-local id space.
    NodeLimit,
    /// A node's enter/leave or attached owner violated the shared walk.
    InvalidTraversal { node: NodeId },
    /// The result does not account for the sealed owner's node count.
    NodeCountMismatch { expected: u32, actual: usize },
    /// A supplied key is outside the sealed owner's dense node range.
    InvalidNode { node: NodeId },
    /// A supplied node would replace a decision instead of adding one row.
    DuplicateNode { node: NodeId },
    /// A control owner appeared more than once in the shared walk.
    DuplicateControl { node: NodeId },
}
