//! Emission targets. Each writes text directly into a [`crate::write::Writer`].
//!
//! - [`dom`] — the virtual-DOM render function, from L2 plus L3 decisions.
//! - [`ssr`] — the server render function, from L2 plus L3 decisions; no
//!   legacy codegen and no Croquis.
//! - [`vapor`] — the Vapor render function, generated directly from the L3
//!   program with the current emission order; no legacy IR.
//! - [`ts`] — the type-check projection (virtual TS) mapped back through
//!   span links.

pub mod dom;
pub mod ssr;
pub mod ts;
pub mod vapor;
