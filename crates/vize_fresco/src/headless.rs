//! Deterministic, terminal-free rendering and semantic snapshots.
//!
//! Headless snapshots exercise Fresco's production layout, painter, and cell
//! buffer without enabling terminal modes or writing escape sequences. Callers
//! supply semantic metadata separately from visual nodes so rendering remains
//! generic while accessibility and focus contracts stay directly assertable.

mod model;
mod renderer;
mod snapshot;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod validation_tests;

pub use model::{
    AnnouncementPoliteness, HeadlessAnnouncement, HeadlessPresentation, HeadlessSemanticNode,
    SemanticRole, SemanticState,
};
pub use renderer::{DEFAULT_HEADLESS_CELL_BUDGET, HeadlessRenderError, HeadlessRenderer};
pub use snapshot::{HeadlessCell, HeadlessSnapshot, SemanticSnapshotNode};
