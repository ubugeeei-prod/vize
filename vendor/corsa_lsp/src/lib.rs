//! LSP-focused client helpers for Corsa.
//!
//! This crate complements `corsa_client` with Language Server Protocol
//! utilities:
//!
//! - [`LspClient`] talks to Corsa over LSP stdio
//! - [`LspOverlay`] keeps a mirrored set of virtual documents
//! - [`VirtualDocument`] and [`VirtualChange`] model in-memory edits in LSP
//!   coordinates
//! - custom request types expose the extra protocol extensions that Corsa
//!   layers on top of standard LSP
//!
//! These types are most useful for editor integrations, benchmarks that mimic
//! editor workflows, and orchestration layers that need to replicate
//! virtual-document state.

/// Re-exports JSON-RPC transport primitives used by the LSP client.
pub mod jsonrpc {
    pub use corsa_jsonrpc::*;
}

/// Re-exports child-process helpers used to launch Corsa over LSP stdio.
pub mod process {
    pub use corsa_core::{
        AsyncChildGuard, CorsaCommand, ProcessCloseObservation, ProcessCloseReceipt,
        ProcessLifetimeObserver, ProcessLifetimeSnapshot, ProcessObservation, ProcessObserver,
        ProcessOwnership, ProcessReceipt, ProcessTerminalObserver,
    };
}

/// Re-exports structured operational events used by the LSP configs.
pub mod observability {
    pub use corsa_core::{CorsaEvent, CorsaObserver, SharedObserver};
}

pub use corsa_core::{CorsaError, CorsaEvent, CorsaObserver, Result, SharedObserver};

#[path = "lsp/mod.rs"]
/// LSP client, overlay, and custom-request types.
pub mod lsp;

pub use lsp::*;
