//! P3-2 reactivity lattice facts for S3.
//!
//! The production S2-to-S3 lowering will feed this module from retained AST
//! summaries. The first slice keeps that boundary explicit: callers provide a
//! compact effect/escape summary per binding, and the naive evaluator publishes
//! a typed fact group plus a folio page.

mod class;
mod effect;
mod evaluate;
mod folio;
mod id;
mod input;

pub use class::ReactivityClass;
pub use effect::{EffectKind, EffectSet};
pub use evaluate::{LatticeFacts, evaluate, evaluate_binding};
pub use folio::{FolioBinding, ReactivityFolio};
pub use id::BindingId;
pub use input::{BindingFact, BindingInput, BindingOrigin, EscapeKind, Verdict};

const _: () = assert!(!core::mem::needs_drop::<BindingFact>());
const _: () = assert!(!core::mem::needs_drop::<BindingInput>());

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<BindingFact>() <= 32);
    assert!(core::mem::size_of::<BindingInput>() <= 32);
};
