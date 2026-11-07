//! Original Vue 2 callback owners lowered into the shared canonical op family.
//!
//! This first, explicit family preserves ordinary single-line HTML text, static
//! attributes and non-filter interpolations. Unsupported semantics remain typed
//! holes beside the complete original owner. There is no modern Component/File
//! conversion, compatibility parser, reparse, or product fallback.
//!
//! Physical ownership lives under `dialect/vue2`; ordinary child declarations
//! remain here so existing API, type and test identities do not change.

mod children;
mod element;
#[cfg(test)]
mod tests;

include!("dialect/vue2/native.rs");
