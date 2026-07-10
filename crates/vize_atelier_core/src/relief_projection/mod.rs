//! Compatibility classification helpers for the legacy Relief-driven emitter.
//!
//! These views are deliberately local to Atelier. They must not be confused
//! with the frontend-neutral `vize_rendu` HIR used by the artifact graph.

mod children;
mod model;
mod op;

pub(crate) use children::ReliefChildren;
pub(crate) use model::ReliefElementKind;
pub(crate) use op::ReliefRenderOp;
