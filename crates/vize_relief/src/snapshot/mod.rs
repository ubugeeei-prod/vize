//! Owned, source-faithful snapshots of Relief syntax.
//!
//! The parser-facing Relief tree is arena allocated. [`ReliefSnapshot`] copies
//! that tree into an owned `Send + Sync + 'static` product suitable for Atlas
//! caches and concurrent tool consumers. The snapshot retains Relief syntax:
//! tags, properties, directives, expressions, control-flow syntax, comments,
//! source locations, nesting, and source order.
//!
//! This is deliberately not a render HIR. It does not normalize elements into
//! backend operations, infer semantic identity, or link files. Rendu owns
//! render lowering, Croquis owns derived semantics, and `vize_croquis_cf` owns
//! cross-file aggregation.

mod convert;
mod copy;
mod expression;
mod ids;
mod materialize;
mod node;
mod property;
mod root;
mod walk;

#[cfg(test)]
mod tests;

pub use expression::{
    SnapshotCompoundChild, SnapshotCompoundExpression, SnapshotExpression, SnapshotForParseResult,
    SnapshotSimpleExpression,
};
pub use ids::ReliefSnapshotNodeId;
pub use node::{
    ReliefSnapshotNode, ReliefSnapshotNodeKind, SnapshotComment, SnapshotElement, SnapshotFor,
    SnapshotHoisted, SnapshotIf, SnapshotIfBranch, SnapshotInterpolation, SnapshotText,
    SnapshotTextCall, SnapshotTextCallContent,
};
pub use property::{SnapshotAttribute, SnapshotDirective, SnapshotProp};
pub use root::{ReliefSnapshot, SnapshotImport};
pub use walk::{ReliefSnapshotVisit, ReliefSnapshotWalker};
