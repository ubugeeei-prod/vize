//! `ui.model` - the two-way binding contract, never its realization.
//!
//! `v-model` is not sugar for `:value` + `@input`: the runtime realization
//! guards IME composition, handles checkbox arrays, `.lazy`'s
//! change-vs-input switch, and select-multiple. So the neutral core
//! carries **the contract only** - what is read, what is written, the
//! value-type flow - which is all lint, the reactivity lattice, and type
//! projection need, and which Svelte's `bind:` lowers to identically
//! (charter #40). Each S4 target picks the realization at lowering: VDOM
//! emits runtime directive references, Vapor calls upstream vapor helpers,
//! SSR renders attributes. IME/composition handling is **runtime-owned by
//! declaration** (charter #23 tiering); the compiler's obligation ends at
//! selecting the realization and preserving this contract.

use vize_s0::{Span, Vec};

use super::{Attribute, DynamicName};
use crate::expr::ExprRef;

/// The two-way binding contract of a [`ModelOp`].
///
/// The value-type flow is the pair's law: **the type of what is read is
/// the type of what is written** - one declared value type flowing
/// view-ward through `read` and model-ward through `write`. The law
/// stays on the pair rather than in a third field: now that the
/// positions are real [`ExprRef`]s (usually two references to the *same*
/// payload - the authored expression; custom accessors split them), the
/// flow is checkable on them directly, and checking it is the verifier's
/// (P2-6) and the projection's, never this type's.
#[derive(Debug, Clone, Copy)]
pub struct BindingContract<'a> {
    /// What the view reads.
    pub read: ExprRef<'a>,
    /// What updates write into. Usually the same payload as `read`;
    /// custom accessors split them.
    pub write: ExprRef<'a>,
}

/// `ui.model` - a two-way binding, attached to one element or component.
///
/// Realization is never expanded in S2. The authored argument uses the
/// same static-or-computed name position as `ui.bind` / `ui.on`; element
/// kind and dialect modifiers (`.lazy`, `.number`, `.trim`) ride as
/// [`Attribute`]s so the contract stays dialect-neutral while lowering
/// still sees everything it needs to select a realization.
#[derive(Debug)]
pub struct ModelOp<'a> {
    /// The binding contract (see [`BindingContract`] for the flow law).
    pub contract: BindingContract<'a>,
    /// Authored model prop name, when present.
    pub argument: Option<DynamicName<'a>>,
    /// Element kind and dialect modifiers, in lowering-declared order.
    pub attributes: Vec<'a, Attribute<'a>>,
    /// The authored binding's source range.
    pub span: Span,
}

/// See [`crate::op`] for the guard rationale.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<BindingContract<'_>>() == 32);
    assert!(core::mem::size_of::<ModelOp<'_>>() == 88);
};
