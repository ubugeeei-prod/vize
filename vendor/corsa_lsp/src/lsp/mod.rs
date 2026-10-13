//! LSP client and virtual-document helpers for Corsa.
//!
//! The public API intentionally mirrors the way editor integrations think about
//! the world:
//!
//! - [`LspClient`] owns the transport connection
//! - [`LspOverlay`] tracks which documents are open
//! - [`VirtualDocument`] and [`VirtualChange`] model the in-memory text state
//! - custom request types expose Corsa-specific extensions

mod client;
mod custom;
mod overlay;
mod virtual_document;
#[cfg(test)]
mod virtual_document_tests;

/// Stdio LSP client and spawn configuration.
pub use client::{LspClient, LspSpawnConfig};
/// Custom Corsa LSP request definitions.
pub use custom::{
    InitializeApiSessionParams, InitializeApiSessionRequest, InitializeApiSessionResult,
};
/// In-memory overlay that emits LSP document notifications.
pub use overlay::LspOverlay;
/// Virtual documents and incremental edits expressed in LSP coordinates.
pub use virtual_document::{VirtualChange, VirtualDocument};
