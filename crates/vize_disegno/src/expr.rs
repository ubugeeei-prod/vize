//! The S2 expression reference (P2-5b): [`ExprRef`] and its payload family.
//!
//! An expression position in the op family holds an [`ExprRef`] - a shared
//! arena reference to one of the payloads below, and the enum is **total
//! over everything the parser actually produces**:
//!
//! - [`ExprRef::Js`] - the retained oxc AST (the P1-5 parse-once contract):
//!   text that parses as one complete TS-dialect expression covering the
//!   whole content, carried beside that exact text and its authored span.
//! - [`ExprRef::Foreign`] - a non-JS expression dialect (charter #28).
//!   **Type-only until phase 6**: the type and its folio spelling exist so
//!   the contract is closed, but no dialect implementation ships and no
//!   lowering constructs one.
//! - [`ExprRef::Filter`] - Vue 2 pipe filters (`{{ msg | capitalize }}`).
//!   A dialect expression, not an escape: `|` here is not bitwise-OR.
//!   Constructed only when a Vue 2 dialect lowering admits a chain
//!   (P2-9 installment 7); never produced by [`ExprRef::parse_js_in`].
//! - [`ExprRef::Opaque`] - the escape variant for the retained-`None`
//!   classes P1-5/P1-9 measured, with **pessimal documented semantics from
//!   day one** (the imported LLVM `undef`/`poison` rule - see
//!   [`opaque`]'s module docs for the laws, and
//!   `davinci-road/plan/phase-2-records/p2-5b.md` for the decision record
//!   and the measurement that picked this resolution).
//!
//! # No equality, on purpose
//!
//! `ExprRef` implements neither `PartialEq` nor `Eq`. For [`OpaqueExpr`]
//! textual equality must never imply semantic equality (pessimal law 4:
//! an opaque expression is not even equal to itself), and offering
//! structural equality on the other variants would make the unsound
//! comparison one `match` arm away. Consumers that need identity compare
//! the owned folio mirror, which is explicit about what it carries.
//!
//! # Capability resolution
//!
//! How a consumer asks questions about an expression (enumerate
//! referenced bindings, classify const-ness, map spans, emit for a
//! target) is the [`capability`] contract, resolved **once per file and
//! never dyn-dispatched per node** (performance guardrail 1).

pub mod capability;
pub mod filter;
pub mod foreign;
pub mod js;
pub mod opaque;

pub use capability::ExprDialect;
pub use filter::{VueFilterApp, VueFilterExpr};
pub use foreign::{ForeignExpr, ForeignFact};
pub use js::JsExpr;
pub use opaque::{OpaqueExpr, OpaqueReason};

use vize_s0::{Allocator, Span};

/// One expression position's reference: retained JS AST, foreign dialect,
/// or the classified escape.
///
/// A *reference*, deliberately: payloads are shared `&'a` borrows of the
/// compile arena, so two positions may name the same payload (`ui.model`'s
/// `read`/`write` typically do), and the enum stays two words. Nothing
/// here survives `Allocator::reset` - the folio carries the owned form
/// (P1-11's contract; see `crate::folio`).
#[derive(Debug, Clone, Copy)]
pub enum ExprRef<'a> {
    /// The retained oxc AST with its exact source text and authored span.
    Js(&'a JsExpr<'a>),
    /// A foreign-dialect expression (type-only until phase 6).
    Foreign(&'a ForeignExpr<'a>),
    /// Vue 2 pipe-filter chain (the dialect reading of `|`).
    Filter(&'a VueFilterExpr<'a>),
    /// The escape variant: no AST exists, pessimal semantics apply.
    Opaque(&'a OpaqueExpr<'a>),
}

impl<'a> ExprRef<'a> {
    /// The payload's stage-wide mnemonic - the keyword its folio token
    /// starts with.
    #[must_use]
    pub const fn mnemonic(&self) -> &'static str {
        match self {
            Self::Js(_) => "js",
            Self::Foreign(_) => "foreign",
            Self::Filter(_) => "vue.filter",
            Self::Opaque(_) => "opaque",
        }
    }

    /// The exact authored (or synthesized) text of the expression.
    ///
    /// For [`ExprRef::Js`] this is the text the AST was parsed from and
    /// covers (the P1-5 `raw` contract); for the other variants it is all
    /// the compiler knows.
    #[must_use]
    pub const fn source(&self) -> &'a str {
        match self {
            Self::Js(js) => js.source,
            Self::Foreign(foreign) => foreign.source,
            Self::Filter(filter) => filter.source,
            Self::Opaque(opaque) => opaque.source,
        }
    }

    /// The authored range of [`ExprRef::source`] in the compiled file.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Js(js) => js.span,
            Self::Foreign(foreign) => foreign.span,
            Self::Filter(filter) => filter.span,
            Self::Opaque(opaque) => opaque.span,
        }
    }

    /// The **total** load-path constructor: parse `source` as one complete
    /// TS-dialect expression into the arena, falling back to the escape
    /// variant when the text is not admitted.
    ///
    /// This is how a folio's `js(...)` payload re-enters an arena (the
    /// owned page stores slice + span only, because AST references cannot
    /// persist across a compile - P1-11). It never panics and never
    /// half-succeeds (the MLIR one-shot import: lowerings are total
    /// functions): text [`JsExpr::parse_in`] refuses comes back as
    /// [`ExprRef::Opaque`] with the classified reason and pessimal
    /// semantics, which is exactly what a hand-edited or stale folio has
    /// earned.
    #[must_use]
    pub fn parse_js_in(allocator: &'a Allocator, source: &'a str, span: Span) -> Self {
        match JsExpr::parse_in(allocator, source, span) {
            Ok(js) => Self::Js(js),
            Err(reason) => Self::Opaque(allocator.alloc(OpaqueExpr {
                reason,
                source,
                span,
            })),
        }
    }
}

/// The reference family is `Drop`-free like every S2 type (see
/// [`crate::op`]).
const _: () = assert!(!core::mem::needs_drop::<ExprRef<'static>>());

/// Two words: a tag plus a shared payload pointer (see [`crate::op`] for
/// the guard rationale).
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<ExprRef<'_>>() == 16);
