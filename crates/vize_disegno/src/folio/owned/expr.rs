//! Owned mirrors of the expression payloads (P2-5b).
//!
//! The folio stores expressions as **owned text plus span**, never as AST
//! references, because arena references cannot persist across a compile
//! (P1-11's contract, enforced by the debug arena-generation stamp in
//! `crates/vize_s0/src/allocator/generation.rs`): a `js(...)` payload
//! re-parses into the arena on load through
//! [`JsExpr::parse_in`](crate::expr::JsExpr::parse_in) (or totally via
//! [`ExprRef::parse_js_in`](crate::expr::ExprRef::parse_js_in)). The
//! owned mirrors *do* implement `PartialEq` - equality here compares what
//! the folio carries, which is explicit text, not expression semantics;
//! the pessimal no-equality law lives on the arena types.

use vize_s0::{Span, String};

use crate::expr::{ExprRef, OpaqueReason};

/// Mirror of [`ExprRef`]: one expression payload as the folio carries it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolioExpr {
    /// `js(...)` - re-parses into the arena on load.
    Js {
        /// The exact text the retained AST was parsed from.
        source: String,
        /// The authored range of `source`.
        span: Span,
    },
    /// `foreign(...)` - dialect id + source + span (the phase-6 dialect
    /// contract owns the side tables' folio spelling; until then they do
    /// not serialize - see [`crate::expr::ForeignExpr::facts`]).
    Foreign {
        /// The dialect id.
        dialect: String,
        /// The exact authored text.
        source: String,
        /// The authored range of `source`.
        span: Span,
    },
    /// `opaque(...)` - reason + slice + span; pessimal semantics travel
    /// with the reason.
    Opaque {
        /// Why no AST exists.
        reason: OpaqueReason,
        /// The exact authored (or rebuilt) text.
        source: String,
        /// The authored range of `source`.
        span: Span,
    },
    /// `vue.filter(...)` - Vue 2 pipe-filter chain as authored text.
    Filter {
        /// The exact authored text, pipes included.
        source: String,
        /// The authored range of `source`.
        span: Span,
    },
}

/// Mirror of [`crate::op::ForBinding`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioForBinding {
    /// The iterated collection, object, or range.
    pub source: FolioExpr,
    /// The per-iteration value binding.
    pub value: FolioExpr,
    /// The second binding position (object key), when authored.
    pub key: Option<FolioExpr>,
    /// The third binding position (index), when authored.
    pub index: Option<FolioExpr>,
}

/// Mirror of [`crate::op::BindingContract`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioContract {
    /// What the view reads.
    pub read: FolioExpr,
    /// What updates write into.
    pub write: FolioExpr,
}

/// Bridge one live expression reference into the owned model.
///
/// Exhaustive with no `_` arm on purpose (the staleness discipline of
/// [`super`]): a new `ExprRef` variant must break this file loudly.
pub(in crate::folio) fn own_expr(expr: &ExprRef<'_>) -> FolioExpr {
    match expr {
        ExprRef::Js(js) => FolioExpr::Js {
            source: String::from(js.source),
            span: js.span,
        },
        ExprRef::Foreign(foreign) => FolioExpr::Foreign {
            dialect: String::from(foreign.dialect),
            source: String::from(foreign.source),
            span: foreign.span,
        },
        ExprRef::Opaque(opaque) => FolioExpr::Opaque {
            reason: opaque.reason,
            source: String::from(opaque.source),
            span: opaque.span,
        },
        ExprRef::Filter(filter) => FolioExpr::Filter {
            source: String::from(filter.source),
            span: filter.span,
        },
    }
}
