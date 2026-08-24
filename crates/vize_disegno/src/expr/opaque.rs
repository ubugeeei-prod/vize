//! [`OpaqueExpr`] - the escape variant, with pessimal semantics written
//! down before the first consumer exists.
//!
//! # Why an escape variant exists at all
//!
//! The retained-AST contract (P1-5) is deliberately narrow: an AST is
//! retained iff the text parses as one complete TS expression covering
//! the whole content. P1-9 measured the text outside that contract at
//! 11.73% of corpus expression rewrites (re-measured at 12.73% for this
//! task - the record has both runs), and two of its classes can **never**
//! carry an AST: text the nesting guard refuses is never handed to a
//! parser at all, and text oxc rejects has no AST by definition. A total
//! `ExprRef` therefore needs this variant no matter how far the retained
//! contract is later widened - widening only moves inputs from `Opaque`
//! to `Js`, it cannot empty the class.
//!
//! # The pessimal laws (the imported LLVM `undef`/`poison` rule)
//!
//! LLVM's expensive regret was an escape value with **underspecified**
//! semantics: passes assumed whatever was locally convenient, and the
//! contradictions took a decade to dig out. The imported rule is that the
//! escape variant answers every capability query with the worst case,
//! from day one, in writing:
//!
//! 1. **May read anything.** An opaque expression is assumed to
//!    reference every binding reachable in its scope. No pass may prove
//!    a binding unused, unshadowed, or dead across one.
//! 2. **May do anything.** It is assumed to have arbitrary side effects.
//!    No pass may drop, duplicate, reorder, or speculate one.
//! 3. **Never constant.** Const-ness classification answers "not
//!    constant" ([`OpaqueExpr::is_constant`]). No folding, hoisting, or
//!    caching.
//! 4. **Equal to nothing, itself included.** Textual equality never
//!    implies semantic equality, so no CSE, deduplication, or
//!    memoization keys on an opaque expression. Encoded in the type:
//!    neither `OpaqueExpr` nor `ExprRef` implements `PartialEq`.
//! 5. **Emission is byte-verbatim or refusal.** The only sound emission
//!    is [`OpaqueExpr::source`] unchanged, into a position whose grammar
//!    accepts it verbatim; a target that cannot do that must fail with a
//!    diagnostic, never guess.
//!
//! The consequence is deliberate: **every transformation of an opaque
//! expression is unsound by definition**, so the only valid handlings
//! are pass-through and diagnostic. A pass that wants to do better must
//! first move the input out of this class - by widening the retained
//! contract (`crates/vize_armature/src/parser/expression.rs`), which the
//! laws make a monotone, always-safe change.

use vize_s0::Span;

/// Why an expression position carries no AST.
///
/// The classes are exactly the measured retained-`None` shapes (P1-5's
/// contract, P1-9's counters). Two are **text-classifiable** - decidable
/// from the text alone, and the only refusals [`super::JsExpr::parse_in`]
/// produces. The other three are **position-classified**: grammatical
/// facts about where the text came from, assignable only by the lowering
/// that saw the position (P2-8), never recoverable from the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpaqueReason {
    /// Position-classified: a `v-for` value (`item of items`). The
    /// splitter synthesizes sub-expressions that never existed as
    /// template text, and the value itself is not a JS expression - JS
    /// `in` associates left while Vue's v-for grammar splits at the
    /// first viable `in`/`of`, so the two genuinely disagree on
    /// `a in b in c`. A retained AST of the whole value would parse the
    /// wrong grammar; consuming one naively is the P1-6-recorded bug
    /// P2-8's lowering must not reintroduce.
    ForValue,
    /// Position-classified: a `v-on` handler body of more than one
    /// statement (`a++; b++`). A lone `Expression` cannot represent it.
    MultiStatement,
    /// Text-classifiable: the shared nesting guard
    /// (`vize_s0::expression_guard::expression_is_safe_to_parse`)
    /// refused the text before parsing - oxc's recursive parser cannot
    /// be depth-limited, so the text was never handed to it at all.
    NestingRefused,
    /// Text-classifiable: oxc rejected the text, or the first complete
    /// expression does not cover it.
    ParseRejected,
    /// Position-classified: a compound expression rebuilt from source
    /// slices (merged text + interpolation), whose content never existed
    /// as one authored expression.
    Compound,
}

impl OpaqueReason {
    /// The reason's folio spelling.
    #[must_use]
    pub const fn mnemonic(self) -> &'static str {
        match self {
            Self::ForValue => "for-value",
            Self::MultiStatement => "multi-statement",
            Self::NestingRefused => "nesting-refused",
            Self::ParseRejected => "parse-rejected",
            Self::Compound => "compound",
        }
    }

    /// Parse a folio spelling back into the reason.
    #[must_use]
    pub fn from_mnemonic(text: &str) -> Option<Self> {
        match text {
            "for-value" => Some(Self::ForValue),
            "multi-statement" => Some(Self::MultiStatement),
            "nesting-refused" => Some(Self::NestingRefused),
            "parse-rejected" => Some(Self::ParseRejected),
            "compound" => Some(Self::Compound),
            _ => None,
        }
    }
}

/// The escape payload behind [`super::ExprRef::Opaque`]: the classified
/// reason, the exact text, and where it was authored.
///
/// Semantics are the module-level pessimal laws; this type deliberately
/// offers nothing that would make violating them convenient (no
/// `PartialEq`, no accessor that "interprets" the text).
#[derive(Debug, Clone, Copy)]
pub struct OpaqueExpr<'a> {
    /// Why no AST exists.
    pub reason: OpaqueReason,
    /// The exact authored (or rebuilt) text.
    pub source: &'a str,
    /// The authored range of `source` in the compiled file.
    pub span: Span,
}

impl OpaqueExpr<'_> {
    /// Pessimal law 3: an opaque expression is never constant.
    #[must_use]
    pub const fn is_constant(&self) -> bool {
        false
    }
}

/// See [`crate::op`] for both guard rationales.
const _: () = assert!(!core::mem::needs_drop::<OpaqueExpr<'static>>());
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<OpaqueReason>() == 1);
    assert!(core::mem::size_of::<OpaqueExpr<'_>>() == 32);
};
