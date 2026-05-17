//! NAPI bindings for SFC parsing and compilation.
//!
//! FFI boundary code: uses std types for JavaScript interop.
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
