//! Frontend-neutral control-flow, data-flow, and effect graphs.
//!
//! `vize_flow` is the single-compilation-unit flow representation shared by
//! compilers, linters, and type checkers. A frontend may produce it from SFC,
//! JSX, a legacy compiler input, or a future syntax without exposing that
//! syntax in this crate.
//!
//! This crate deliberately does not parse source or infer frontend semantics.
//! It records blocks, operations, control transfer, values, symbols, and
//! effects after a producer has identified them. It therefore has no
//! dependency on Relief, Croquis, Atelier, JSX, or SFC crates.
//!
//! # Not cross-file analysis
//!
//! `vize_flow` and `vize_croquis_cf` solve different problems. This crate
//! represents control and data flow inside one compilation unit.
//! `vize_croquis_cf` aggregates semantic facts across modules and component
//! files. Cross-file consumers may use flow-derived facts, but cross-file
//! ownership and module resolution do not belong here.
//!
//! # Model
//!
//! - [`FlowGraph`] owns opaque, type-safe IDs and validates every reference at
//!   its API boundary.
//! - [`ControlEdgeKind`] preserves branch, loop, return, and exceptional flow.
//! - values and [`DataEdge`]s model definitions and uses independently of ASTs;
//! - [`Effect`]s and [`EffectEdge`]s model observable operations and ordering;
//! - [`Reachability`] and [`Dominators`] provide reusable graph analyses.

mod analysis;
mod build;
mod error;
mod graph;
mod ids;
mod model;
mod product;
mod provenance;
mod validate;

pub use analysis::{Dominators, Reachability};
pub use error::{FlowError, FlowResult, InvariantViolation, ValidationErrors};
pub use graph::FlowGraph;
pub use ids::{
    BlockId, ControlEdgeId, DataEdgeId, EffectEdgeId, EffectId, NodeId, SourceId, SymbolId, ValueId,
};
pub use model::{
    Block, ControlEdge, ControlEdgeKind, DataEdge, DataEdgeKind, DataUseKind, Effect, EffectEdge,
    EffectEdgeKind, EffectKind, Node, NodeKind, Source, Symbol, TerminatorKind, Value,
};
pub use product::FlowProduct;
pub use provenance::{Provenance, SourceSpan};
