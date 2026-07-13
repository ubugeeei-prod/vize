//! # vize_atlas
//!
//! Source Atlas is the representation-independent artifact graph for Vize's
//! multi-input, multi-output toolchain. A [`Compilation`] owns stable sources,
//! open typed [`Product`] and [`Provider`] contracts, dependency planning,
//! memoized execution, invalidation, provider observations, counters, and
//! execution traces.
//!
//! Domain crates add marker types implementing [`Product`] and providers
//! implementing [`Provider`] without editing Atlas. Atlas deliberately has no
//! enum of syntax trees, semantic models, render HIRs, tools, or targets.
//! Independently registered providers may target the same product; planning
//! selects exactly one by applicability and captures its [`ProviderId`]. Typed
//! input revisions invalidate only the products and plans that declare them.

mod cache;
mod compilation;
mod error;
mod input;
mod invalidation;
mod observation;
mod outcome;
mod plan;
mod planner;
mod product;
mod provider;
mod shared;
mod source;
mod source_error;
mod trace;

pub use cache::{ArtifactCache, CachedProduct};
pub use compilation::{Compilation, CompilationSnapshot, QuerySession};
pub use error::{PlanError, ProviderError, QueryError, RegisterProviderError};
pub use input::{
    CompilationInput, CompilationInputError, CompilationInputs, InputId, SourceInput,
    SourceInputId, SourceKind, SourceKindInput,
};
pub use invalidation::{
    InputInvalidationReport, InvalidatedProduct, InvalidationPolicy, InvalidationReport,
    SourceInputInvalidationReport, SourceRemovalReport,
};
pub use observation::{ObservationKind, ProviderObservation};
pub use outcome::{ExecutionOutcome, ProductStatus, QueryOutcome};
pub use plan::Plan;
pub use product::{CachePolicy, Product, ProductId, ProductRequest, ProductView};
pub use provider::{PlanningContext, Provider, ProviderContext, ProviderId};
pub use shared::Shared;
pub use source::{
    SourceId, SourceProvenance, SourceRange, SourceRevision, SourceRevisionChange, SourceSnapshot,
    SourceStore,
};
pub use source_error::SourceError;
pub use trace::{ExecutionCounters, ExecutionTrace, ProductCounters, TraceEvent};
