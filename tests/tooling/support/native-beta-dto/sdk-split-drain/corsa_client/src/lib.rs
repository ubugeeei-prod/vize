//! High-level client bindings for the Corsa stdio API.
//!
//! This crate wraps the raw transports and endpoint naming used by Corsa
//! behind typed request/response helpers. In practice it is the main entry
//! point when you want to:
//!
//! - spawn a Corsa worker process
//! - initialize it once and reuse the session
//! - create and reuse snapshots
//! - ask type, symbol, and syntax questions through strongly typed helpers
//! - attach filesystem callbacks for overlay-like workflows
//!
//! # Main Building Blocks
//!
//! - [`ApiClient`] manages a single worker process or pipe connection.
//! - [`ApiSpawnConfig`] describes how that worker should be started.
//! - [`ManagedSnapshot`] keeps snapshot handles alive and releases them on drop.
//! - [`ApiProfile`] gives orchestrators a stable name for a spawn configuration.
//! - [`SemanticQuery`], reached via [`ProjectSession::semantics`], is the stable
//!   fact vocabulary this crate owns.
//!
//! # Two API Surfaces
//!
//! The endpoint helpers on [`ApiClient`] and [`ProjectSession`] mirror upstream
//! Corsa naming on purpose, so new upstream capability is cheap to expose and
//! easy to audit — and so they move when upstream moves.
//!
//! [`SemanticQuery`] is the other half: a small, `corsa-bind`-owned vocabulary
//! that answers with opaque handles and keeps its signatures across upstream
//! renames, versioned by [`SEMANTIC_QUERY_VERSION`]. Build foreign hosts
//! against it, and drop to the mirror when you need the full upstream payload.
//! See `docs/architecture_charter.md`.
//!
//! # Performance Model
//!
//! `corsa` does not try to out-compile Corsa itself. The win comes from
//! session reuse, snapshot reuse, and cheaper transports such as sync msgpack.
//! For docs and benchmarks around that trade-off, see the workspace guides.

/// Re-exports shared error types used by the client APIs.
pub mod error {
    pub use corsa_core::{CorsaError, Result, RpcResponseError};
}

/// Re-exports low-level JSON-RPC helpers used by the stdio client transport.
pub mod jsonrpc {
    pub use corsa_jsonrpc::*;
}

/// Re-exports process-spawning primitives used to launch Corsa.
pub mod process {
    pub use corsa_core::{AsyncChildGuard, CorsaCommand};
}

/// Re-exports structured operational events used by the client configs.
pub mod observability {
    pub use corsa_core::{CorsaEvent, CorsaObserver, SharedObserver};
}

/// Re-exports shared LSP model types used by editor-style API responses.
pub mod lsp_types {
    pub use ::lsp_types::*;
}

pub use corsa_core::{CorsaError, CorsaEvent, CorsaObserver, Result, SharedObserver};

#[path = "api/mod.rs"]
/// Typed bindings for the Corsa stdio API surface.
pub mod api;

pub use api::*;
