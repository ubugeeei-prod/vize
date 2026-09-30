//! P3-2 reactivity lattice facts for L3.
//!
//! The production L2-to-L3 lowering will feed this module from retained AST
//! summaries. The first slice keeps that boundary explicit: callers provide a
//! compact effect/escape summary per binding, and the naive evaluator publishes
//! a typed fact group plus a folio page.

mod class;
pub mod dump;
mod effect;
mod evaluate;
mod from_kind;
mod id;
mod input;

pub use crate::lattice::dump::{Binding as DumpBinding, Page as ReactivityPage};
pub use class::ReactivityClass;
pub use effect::{EffectKind, EffectSet};
pub use evaluate::{LatticeFacts, evaluate, evaluate_binding};
pub use from_kind::SourceKind;
pub use id::BindingId;
pub use input::{BindingFact, BindingInput, BindingOrigin, EscapeKind, Verdict};

const _: () = assert!(!core::mem::needs_drop::<BindingFact>());
const _: () = assert!(!core::mem::needs_drop::<BindingInput>());

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<BindingFact>() <= 32);
    assert!(core::mem::size_of::<BindingInput>() <= 32);
};
