//! NAPI bindings for SFC parsing and compilation.
//!
//! FFI boundary code: uses std types for JavaScript interop.
//!
//! Native batch APIs in this module are deliberately split into stats-only and
//! code-returning paths. The stats path can aggregate repeated SFC bodies inside
//! Rust and avoid sending generated JavaScript over the JS/native boundary, while
//! the code path preserves per-file output for Vite/plugin callers.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

mod batch;
mod batch_results;
mod bundler;
mod compile;
mod css;
mod parse;
mod types;

pub use batch::*;
pub use batch_results::*;
pub use bundler::*;
pub use compile::*;
pub use css::*;
pub use parse::*;
pub use types::*;
