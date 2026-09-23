//! The SDK for `vize:contracts` guests (Davinci phase 6, P6-2).
//!
//! **Experimental:** the contracts are `0.x` until GA; they move under the
//! written policy in `docs/davinci/contracts-compat-policy.md`, and this
//! crate's version moves with them.
//!
//! An input-dialect guest is a `wasm32-wasip2` component that depends on this
//! crate alone — never on a vize implementation crate — and implements two
//! traits:
//!
//! ```text
//! use vize_extension_sdk::handshake::{Capability, Guest as Handshake};
//! use vize_extension_sdk::input_lowering::{Guest as Lowering, LoweredBlock, SourceBlock};
//!
//! struct Dialect;
//! impl Handshake for Dialect {
//!     fn get_capability() -> Capability {
//!         vize_extension_sdk::capability(&["hello"])
//!     }
//! }
//! impl Lowering for Dialect {
//!     fn lower_block(block: SourceBlock) -> LoweredBlock {
//!         /* write the S1 and S2 pages with `vize_extension_sdk::pages` */
//!     }
//! }
//! vize_extension_sdk::export_input_dialect!(Dialect);
//! ```
//!
//! The crate owns the canonical WIT package (`wit/`) and released surfaces
//! (`versions/`), its bindings, the handshake constants the host negotiates,
//! writers for the pages the host accepts ([`pages`]), and — with
//! the default `runtime` feature, on `wasm32` — what an import-free `no_std`
//! guest must provide itself: a global allocator, `cabi_realloc`,
//! `memcmp`/`bcmp` and a panic handler. The world imports no host function,
//! so a guest that links WASI is refused at instantiation.

#![no_std]
// The workspace's `vize_carton` string policy cannot apply here: the SDK
// depends on no vize crate (P6-2's pinned dependency set), and the WIT
// bindings speak `alloc::string::String`.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

pub mod pages;
#[cfg(all(feature = "runtime", target_arch = "wasm32"))]
mod runtime;

/// The generated bindings of the `input-dialect` world.
#[allow(missing_docs, clippy::all, clippy::pedantic)]
pub mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "input-dialect",
        pub_export_macro: true,
        export_macro_name: "export_input_dialect",
        default_bindings_module: "vize_extension_sdk::bindings",
        additional_derives: [PartialEq, Eq],
    });
}

/// Export a type implementing [`handshake::Guest`] and
/// [`input_lowering::Guest`] as the guest component's world.
pub use bindings::export_input_dialect;
pub use bindings::exports::vize::contracts::{handshake, input_lowering};
pub use bindings::vize::contracts::types;

/// The WIT package this SDK binds.
pub const PACKAGE: &str = "vize:contracts@0.1.2";
/// The handshake protocol version of this contract version.
pub const PROTOCOL_VERSION: u32 = 1;
/// The S1 page schema version the [`pages::s1`] writer emits.
pub const S1_PAGE_SCHEMA: u32 = 1;
/// The S2 page schema version the [`pages::s2`] writer emits.
pub const S2_PAGE_SCHEMA: u32 = 1;
/// The features the input-dialect world requires, sorted.
pub const REQUIRED_FEATURES: &[&str] = &["s1-page@1", "s2-page@1"];
/// The facts page (`expression-facts` α document) schema version of the
/// expression-dialect world.
pub const FACTS_PAGE_SCHEMA: u32 = 1;
/// The projection page schema version of the expression-dialect world.
pub const PROJECTION_PAGE_SCHEMA: u32 = 1;
/// The features the expression-dialect world requires, sorted.
pub const EXPRESSION_REQUIRED_FEATURES: &[&str] = &["facts-page@1", "projection-page@1"];
/// The S3 page schema version the output-target world carries.
pub const S3_PAGE_SCHEMA: u32 = 1;
/// The emit-document page schema version the output-target world returns.
pub const EMIT_DOCUMENT_PAGE_SCHEMA: u32 = 1;
/// The features the output-target world requires, sorted.
pub const OUTPUT_REQUIRED_FEATURES: &[&str] = &["emit-document-page@1", "s2-page@1", "s3-page@1"];

/// The capability offer for a guest lowering the given `lang` values:
/// protocol version and features, sorted and unique as the host requires.
#[must_use]
pub fn capability(langs: &[&str]) -> handshake::Capability {
    let mut features: Vec<String> = langs
        .iter()
        .map(|lang| ["lang:", lang].concat())
        .chain(
            REQUIRED_FEATURES
                .iter()
                .map(|&feature| String::from(feature)),
        )
        .collect();
    features.sort_unstable();
    features.dedup();
    handshake::Capability {
        protocol_version: PROTOCOL_VERSION,
        features,
    }
}
