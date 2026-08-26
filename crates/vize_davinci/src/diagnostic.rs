//! The unified diagnostic every stage emits.
//!
//! `architecture.md`'s one-diagnostics-channel contract: "diagnostics carry a
//! `Span`, a stage of origin, and structured parts; all rendering (CLI, LSP,
//! JSON, corpus reports) consumes the same finished `Vec<Diagnostic>`. This
//! structurally removes the two-independent-assembly-paths failure mode in
//! canon."
//!
//! # What P2-1 lands, and what it does not
//!
//! This module lands the **type**. Replacing `vize_relief::CompilerError` at
//! its call sites is P4's convergence task and is explicitly a non-goal here:
//! a type nothing emits yet cannot change any output byte, which is why
//! P2-1's corpus acceptance is trivially empty.
//!
//! # Coordinates
//!
//! A diagnostic keys on [`vize_s0::Span`] — two byte offsets — and nothing
//! else. `Position` does not exist: P1-4 retired line/column tracking after
//! measuring the parser's to be degenerate by construction, and P1-3 shrank
//! `SourceLocation` from 48 bytes to 8 by making it span-only. Line and column
//! are **derived at rendering time** from the authored text, via
//! `vize_s0::line_index::LineIndex`.
//!
//! # Ownership: why the text is owned
//!
//! Every field is `'static`, enforced by the assertion at the bottom of this
//! file. That is the P1-11 arena/cache contract: a compile's arena is reset
//! and reused for the next file, so anything crossing a compile boundary must
//! own its bytes or it is reading freed memory. Diagnostics cross that
//! boundary by definition — they outlive the compile that produced them, get
//! collected across a batch, and get cached.
//!
//! Message text is therefore **owned**, the deliberate P1-10 exception that
//! `vize_relief::CompilerError::message` already carries. Every other node
//! field in the compiler moved to `&'a str` in that change; diagnostics did
//! not, for exactly this reason.

use alloc::vec::Vec;

pub use crate::stage::Stage;
use vize_s0::{Span, String};

/// How much a diagnostic claims.
///
/// The distinction is load-bearing rather than cosmetic: `assurance.md`'s
/// no-error-on-maybe rule is that a heuristic "never produces an error; it
/// produces silence, or an explicitly-labeled hint/suggestion severity that
/// _says_ it is not a proof". [`Severity::Error`] is the severity P4-6 will
/// require a verifying [`Witness`] for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// A proven violation. P4-6 will require a verifying witness here.
    Error,
    /// A probable problem, stated as such.
    Warning,
    /// Information that is not a problem.
    Info,
    /// A suggestion the author may ignore.
    Hint,
}

/// What a structured part of a diagnostic is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PartKind {
    /// The span the diagnostic is primarily about.
    Primary,
    /// A span that explains the primary one (a declaration, a prior use).
    Secondary,
    /// Guidance that is not a machine-applicable edit.
    Help,
    /// A machine-applicable edit, whose `message` is the replacement text.
    Suggestion,
}

/// One labelled span of a diagnostic.
///
/// Structured parts are what make the rustc/Elm-grade bar (charter #42)
/// reachable by the renderer instead of by string assembly in each producer:
/// a renderer receives spans and roles, never a pre-formatted sentence it has
/// to re-parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticPart {
    /// The role this part plays.
    pub kind: PartKind,
    /// Where it points.
    pub span: Span,
    /// Its label. Owned, for the reason in the module docs.
    pub message: String,
}

impl DiagnosticPart {
    /// A part of `kind` labelling `span`.
    #[must_use]
    pub fn new(kind: PartKind, span: Span, message: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            message: message.into(),
        }
    }
}

/// The proof a diagnostic rests on.
///
/// `assurance.md`: "An error must carry its witness — the concrete fact chain
/// that proves the violation (binding → escape → effect edge, with spans via
/// provenance). The witness is machine-checkable against the fact base, so a
/// false positive is not a matter of opinion: it is a witness that fails
/// verification."
///
/// **This is a slot, not an implementation.** The fact-chain type and its
/// verifier arrive with the unified channel at P4-6; until then the variant
/// set is deliberately just the two states the migration note describes, so
/// that a diagnostic can already record *that* it is unproven rather than
/// leave the question unrepresented. Phase 4 exits on "every error-severity
/// diagnostic carries a verifying witness"; legacy diagnostics are exempt **by
/// inventory** (see [`Witness::LegacyExempt`]), never silently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Witness {
    /// A producer that predates the witness SDK, exempt by inventory.
    ///
    /// The payload names the producer, so the exemption is an entry in a list
    /// rather than an absence — which is the whole point of "never silently".
    LegacyExempt(String),
}

/// A diagnostic from any stage, in the one channel every renderer reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// How much this diagnostic claims.
    pub severity: Severity,
    /// Which stage produced it.
    pub stage: Stage,
    /// The primary source range, in authored-file byte offsets.
    pub span: Span,
    /// The headline message. Owned, for the reason in the module docs.
    pub message: String,
    /// Labelled spans, help and suggestions.
    pub parts: Vec<DiagnosticPart>,
    /// The proof, once P4-6 lands it. `None` today for every producer.
    pub witness: Option<Witness>,
}

/// The classification enums are single-byte tags on every target, so these
/// hold on the wasm32 lane P2-14 makes required as well - see the note on
/// `NodeId`'s asserts for why they are not pointer-width-guarded.
const _: () = assert!(size_of::<Severity>() == 1);
const _: () = assert!(size_of::<Stage>() == 1);
const _: () = assert!(size_of::<PartKind>() == 1);

/// `Diagnostic` is one owned value copied into batch collections and caches, so
/// a fat one is paid for per finding: 88 bytes is `Span` (8) + severity/stage
/// tags + an owned `String` message (24) + the parts `Vec` (24) + the witness
/// slot (24, and the reason a real witness type must be boxed rather than
/// inlined when P4-6 lands it). These are 64-bit footprints of
/// pointer-containing structs, so they carry the `vize_relief` guard.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Diagnostic>() == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<DiagnosticPart>() == 40);

/// Type-level half of the P1-11 arena/cache contract: a diagnostic that
/// borrowed arena bytes could not be `'static`, so this stops compiling if one
/// ever does. Same enforcement the batch cache types carry.
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<Diagnostic>();
    assert_owned::<DiagnosticPart>();
    assert_owned::<Witness>();
};

impl Diagnostic {
    /// A diagnostic with no parts and no witness.
    #[must_use]
    pub fn new(severity: Severity, stage: Stage, span: Span, message: impl Into<String>) -> Self {
        Self {
            severity,
            stage,
            span,
            message: message.into(),
            parts: Vec::new(),
            witness: None,
        }
    }

    /// Attach a structured part.
    #[must_use]
    pub fn with_part(mut self, part: DiagnosticPart) -> Self {
        self.parts.push(part);
        self
    }

    /// Attach a witness.
    #[must_use]
    pub fn with_witness(mut self, witness: Witness) -> Self {
        self.witness = Some(witness);
        self
    }

    /// Whether P4-6's law — every error carries a verifying witness — holds
    /// for this diagnostic.
    ///
    /// Non-error severities satisfy it trivially. The law is not enforced
    /// anywhere yet, by design: it becomes a gate when the witness type is
    /// real, and until then this predicate is what an inventory counts.
    #[must_use]
    pub fn satisfies_witness_law(&self) -> bool {
        self.severity != Severity::Error || self.witness.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticPart, PartKind, Severity, Stage, Witness};
    use vize_s0::Span;

    #[test]
    fn a_new_diagnostic_carries_no_parts_and_no_witness() {
        let diagnostic = Diagnostic::new(
            Severity::Warning,
            Stage::Semantic,
            Span::new(4, 9),
            "v-for without a key",
        );
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert_eq!(diagnostic.stage, Stage::Semantic);
        assert_eq!(diagnostic.span, Span::new(4, 9));
        assert_eq!(diagnostic.message.as_str(), "v-for without a key");
        assert!(diagnostic.parts.is_empty());
        assert_eq!(diagnostic.witness, None);
    }

    #[test]
    fn parts_accumulate_in_the_order_they_are_attached() {
        let diagnostic = Diagnostic::new(
            Severity::Error,
            Stage::Surface,
            Span::new(0, 1),
            "unterminated element",
        )
        .with_part(DiagnosticPart::new(
            PartKind::Primary,
            Span::new(0, 1),
            "opened here",
        ))
        .with_part(DiagnosticPart::new(
            PartKind::Help,
            Span::new(9, 9),
            "add a closing tag",
        ));
        assert_eq!(diagnostic.parts.len(), 2);
        assert_eq!(diagnostic.parts[0].kind, PartKind::Primary);
        assert_eq!(diagnostic.parts[1].kind, PartKind::Help);
        assert_eq!(diagnostic.parts[1].message.as_str(), "add a closing tag");
    }

    #[test]
    fn the_witness_law_holds_trivially_below_error_severity() {
        for severity in [Severity::Warning, Severity::Info, Severity::Hint] {
            let diagnostic =
                Diagnostic::new(severity, Stage::Emit, Span::new(0, 0), "not an error");
            assert!(diagnostic.satisfies_witness_law());
        }
    }

    #[test]
    fn an_error_without_a_witness_fails_the_law_and_with_one_holds_it() {
        let bare = Diagnostic::new(Severity::Error, Stage::Lowered, Span::new(2, 3), "proven");
        assert!(!bare.satisfies_witness_law());

        let exempt = bare
            .clone()
            .with_witness(Witness::LegacyExempt("vize_canon".into()));
        assert!(exempt.satisfies_witness_law());
        assert_eq!(
            exempt.witness,
            Some(Witness::LegacyExempt("vize_canon".into()))
        );
    }

    #[test]
    fn severities_order_from_most_to_least_claimed() {
        assert!(Severity::Error < Severity::Warning);
        assert!(Severity::Warning < Severity::Info);
        assert!(Severity::Info < Severity::Hint);
    }
}
