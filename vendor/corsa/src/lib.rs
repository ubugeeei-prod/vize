//! Top-level facade crate for the `corsa` workspace.
//!
//! This crate re-exports the building blocks used to talk to Corsa
//! over stdio:
//!
//! - [`api`] for the Corsa API bindings
//! - [`jsonrpc`] for stdio JSON-RPC framing and transport
//! - [`lint`] for Rust-authored rule primitives behind Oxlint JS plugins
//! - [`lsp`] for LSP clients and virtual-document overlays
//! - [`orchestrator`] for local orchestration plus optional experimental
//!   distributed helpers
//! - [`observability`] for structured runtime events
//! - [`runtime`] for the lightweight in-house executor
//! - [`utils`] for shared type-text and checker-adjacent helpers
//!
//! # Examples
//!
//! ```
//! use corsa::{
//!     jsonrpc::RequestId,
//!     lsp::{VirtualChange, VirtualDocument},
//!     runtime::block_on,
//! };
//!
//! let mut document = VirtualDocument::untitled("/demo.ts", "typescript", "const value = 1;")?;
//! document.apply_changes(&[VirtualChange::replace("const value = 2;")])?;
//! assert_eq!(document.text, "const value = 2;");
//!
//! let request_id = RequestId::integer(7);
//! assert_eq!(request_id.to_string(), "7");
//!
//! let value = block_on(async { 21 * 2 });
//! assert_eq!(value, 42);
//! # Ok::<(), corsa::CorsaError>(())
//! ```

/// Re-exports the Corsa stdio API bindings.
pub mod api {
    pub use corsa_client::*;
}

/// Re-exports shared error types.
pub mod error {
    pub use corsa_core::{CorsaError, Result, RpcResponseError};
}

/// Re-exports performance-oriented building blocks such as `CompactString`.
pub mod fast {
    pub use corsa_core::fast::*;
}

/// Re-exports JSON-RPC transport primitives.
pub mod jsonrpc {
    pub use corsa_jsonrpc::*;
}

/// Re-exports Rust-authored lint rule primitives.
pub mod lint {
    pub use corsa_core::lint::*;
}

/// Re-exports LSP clients, overlays, and virtual document helpers.
pub mod lsp {
    pub use corsa_lsp::*;
}

/// Re-exports structured operational events used across the workspace.
pub mod observability {
    pub use corsa_core::{CorsaEvent, CorsaObserver, SharedObserver};
}

/// Re-exports client orchestration and replicated-state helpers.
pub mod orchestrator {
    pub use corsa_orchestrator::{ApiOrchestrator, ApiOrchestratorConfig, ApiOrchestratorStats};
    #[cfg(feature = "experimental-distributed")]
    pub use corsa_orchestrator::{
        ChannelTransport, DistributedApiOrchestrator, FileStorage, HardState, InMemoryStorage,
        InProcessTransport, PersistedLogEntry, RaftCluster, RaftClusterBuilder, RaftConfig,
        RaftMessage, RaftRole, RaftSnapshot, RaftStorage, RaftTransport, ReplicatedCacheEntry,
        ReplicatedCommand, ReplicatedSnapshot, ReplicatedState,
    };
}

/// Re-exports process spawning primitives.
pub mod process {
    pub use corsa_core::{AsyncChildGuard, CorsaCommand};
}

/// Re-exports the lightweight in-house runtime.
pub mod runtime {
    pub use corsa_runtime::*;
}

/// Re-exports shared pure utility helpers.
pub mod utils {
    pub use corsa_core::utils::*;
}

pub use corsa_core::{CorsaError, CorsaEvent, CorsaObserver, Result, SharedObserver};
