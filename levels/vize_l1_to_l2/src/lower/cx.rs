//! The lowering context: id minting, span recovery, and the three fact
//! channels (diagnostics, provenance, scopes).
//!
//! L2 node ids are dense and page-ordered. Exhaustion is a diagnostic, not a
//! panic. The Vue policy owns the implementation while this ordinary child
//! module preserves the existing internal identity.

mod custom_element;
mod for_parts;
mod span;
mod text_parent;

include!("../dialect/vue/lower/cx.rs");
