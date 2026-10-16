//! Shared primitives used across the `corsa` workspace.
//!
//! This crate intentionally stays small and foundational. It contains the
//! common error type, process lifecycle helpers, and a few performance-oriented
//! collection/string aliases used throughout the higher-level crates.
//!
//! Most applications will consume this crate indirectly via `corsa_client`
//! or the top-level `corsa` facade, but it is also useful on its own when
//! embedding Corsa process management in another integration.

// Some rule unit tests build deeply nested `json!` lint-node fixtures that
// exceed the default macro recursion limit during expansion.
#![recursion_limit = "256"]

mod error;
/// Compact string/collection aliases used to keep hot paths allocation-light.
pub mod fast;
/// Rust-authored lint rule primitives and built-in type-aware rules.
pub mod lint;
mod observability;
mod process;
mod process_receipt;
mod rpc;
/// Shared pure helpers for type-text classification and unsafe-flow checks.
pub mod utils;

pub use error::{CorsaError, Result};
pub use observability::{CorsaEvent, CorsaObserver, SharedObserver, observe};
/// Child-process guard and reusable command template for Corsa.
pub use process::{
    AsyncChildGuard, CorsaCommand, OwnedProcess, ProcessCloseObservation, ProcessCloseReceipt,
    ProcessLifetimeObserver, ProcessLifetimeSnapshot, ProcessOwnership, ProcessTerminalObserver,
    terminate_child_process, wait_for_child_exit,
};
pub use process_receipt::{ProcessObservation, ProcessObserver, ProcessReceipt};
pub use rpc::RpcResponseError;
