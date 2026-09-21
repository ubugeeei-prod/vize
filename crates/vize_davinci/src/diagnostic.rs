//! The unified diagnostic every stage emits.
//!
//! `architecture.md`'s one-diagnostics-channel contract: "diagnostics carry a
//! `Span`, a stage of origin, and structured parts; all rendering (CLI, LSP,
//! JSON, corpus reports) consumes the same finished `Vec<Diagnostic>`. This
//! structurally removes the two-independent-assembly-paths failure mode in
//! canon."
//!
//! # The verdict rules, as types (P4-6a)
//!
//! `assurance.md`'s "verdicts are proofs" rules are enforced by construction
//! rather than by review:
//!
//! - [`Severity::Error`] is reachable only through [`Diagnostic::proven`],
//!   which takes a non-empty [`WitnessChain`] naming fact groups, or through
//!   [`Diagnostic::legacy_error`], which takes a declared [`Exemption`]
//!   counted by `davinci-road/plan/witness-exemptions.tsv` — an inventory
//!   that only shrinks.
//! - [`Diagnostic::new`] takes an [`Advisory`] severity, so an error without
//!   a witness is a type error — the provisional "canary that tries
//!   error-on-unknown fails to compile":
//!
//! ```compile_fail,E0308
//! use vize_davinci::diagnostic::{Diagnostic, Severity, Stage};
//! use vize_s0::Span;
//!
//! let unproven = Diagnostic::new(Severity::Error, Stage::Semantic, Span::new(0, 1), "maybe");
//! ```
//!
//!   Its twin, identical but for the severity, builds — so the canary fails
//!   for exactly the severity (stable rustdoc does not check the error code;
//!   the twin is what pins the reason):
//!
//! ```
//! use vize_davinci::diagnostic::{Advisory, Diagnostic, Stage};
//! use vize_s0::Span;
//!
//! let unproven = Diagnostic::new(Advisory::Warning, Stage::Semantic, Span::new(0, 1), "maybe");
//! ```
//!
//! - The fields that carry the claim — severity and witness — are private,
//!   so a struct literal cannot forge an unwitnessed error either:
//!
//! ```compile_fail,E0451
//! use vize_davinci::diagnostic::{Diagnostic, Severity, Stage};
//! use vize_s0::Span;
//!
//! let forged = Diagnostic {
//!     severity: Severity::Error,
//!     stage: Stage::Semantic,
//!     span: Span::new(0, 1),
//!     message: "maybe".into(),
//!     parts: Vec::new(),
//!     witness: None,
//! };
//! ```
//!
//! - Rules declare a [`RuleContract`] — a [`Tier`] over a declared
//!   [`Domain`] — whose const constructor rejects a heuristic rule declaring
//!   error severity at compile time (see [`tier`]).
//!
//! Re-checking a witness against the fact base is P4-6b's verifier
//! (`vize_davinci::witness`); this module only makes the unproven error
//! unrepresentable.
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

pub mod severity;
pub mod tier;
pub mod verdict;
pub mod witness;

use alloc::vec::Vec;

pub use crate::stage::Stage;
pub use severity::{Advisory, Severity};
pub use tier::{Domain, RuleContract, Tier};
pub use verdict::Verdict;
use vize_s0::{Span, String};
pub use witness::{Exemption, Witness, WitnessChain, WitnessKey, WitnessKeyed, WitnessLink};

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

/// A diagnostic from any stage, in the one channel every renderer reads.
///
/// `severity` and `witness` are private and paired by the constructors: an
/// error always carries a [`Witness`] — a proof or a counted exemption — and
/// nothing can lower that pairing after construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// How much this diagnostic claims. Read through [`Diagnostic::severity`].
    severity: Severity,
    /// Which stage produced it.
    pub stage: Stage,
    /// The primary source range, in authored-file byte offsets.
    pub span: Span,
    /// The headline message. Owned, for the reason in the module docs.
    pub message: String,
    /// Labelled spans, help and suggestions.
    pub parts: Vec<DiagnosticPart>,
    /// The proof or the exemption. `Some` on every error by construction.
    witness: Option<Witness>,
}

/// The classification enums are single-byte tags on every target, so these
/// hold on the wasm32 lane P2-14 makes required as well - see the note on
/// `NodeId`'s asserts for why they are not pointer-width-guarded.
const _: () = assert!(size_of::<Severity>() == 1);
const _: () = assert!(size_of::<Advisory>() == 1);
const _: () = assert!(size_of::<Stage>() == 1);
const _: () = assert!(size_of::<PartKind>() == 1);

/// `Diagnostic` is one owned value copied into batch collections and caches, so
/// a fat one is paid for per finding: 88 bytes is `Span` (8) + severity/stage
/// tags + an owned `String` message (24) + the parts `Vec` (24) + the witness
/// slot (24). The witness keeps that slot size because its chain is a boxed
/// slice (16) and an exemption is a `&'static` reference (8) — the P2-1 note's
/// "a real witness type must be boxed rather than inlined", honoured. These are
/// 64-bit footprints of pointer-containing structs, so they carry the
/// `vize_relief` guard.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Diagnostic>() == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Option<Witness>>() == 24);
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
    /// A diagnostic that claims no proof: a warning, a note or a hint.
    ///
    /// This is the constructor every unproven finding uses, and it cannot
    /// build an error — see the module docs for the `compile_fail` canary.
    #[must_use]
    pub fn new(severity: Advisory, stage: Stage, span: Span, message: impl Into<String>) -> Self {
        Self::build(severity.severity(), stage, span, message.into(), None)
    }

    /// A proven error: `witness` is the non-empty fact chain that proves the
    /// violation, re-checked against the fact base by P4-6b's verifier.
    #[must_use]
    pub fn proven(
        stage: Stage,
        span: Span,
        message: impl Into<String>,
        witness: WitnessChain,
    ) -> Self {
        let witness = Some(Witness::Proven(witness));
        Self::build(Severity::Error, stage, span, message.into(), witness)
    }

    /// An error from a producer that predates the witness SDK, exempt from
    /// the witness law **by inventory**: `exemption` is a declared `static`
    /// counted by `davinci-road/plan/witness-exemptions.tsv`, never an
    /// ambient absence.
    #[must_use]
    pub fn legacy_error(
        exemption: &'static Exemption,
        stage: Stage,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        let witness = Some(Witness::LegacyExempt(exemption));
        Self::build(Severity::Error, stage, span, message.into(), witness)
    }

    fn build(
        severity: Severity,
        stage: Stage,
        span: Span,
        message: String,
        witness: Option<Witness>,
    ) -> Self {
        Self {
            severity,
            stage,
            span,
            message,
            parts: Vec::new(),
            witness,
        }
    }

    /// Attach a structured part.
    #[must_use]
    pub fn with_part(mut self, part: DiagnosticPart) -> Self {
        self.parts.push(part);
        self
    }

    /// Attach the fact chain behind this diagnostic.
    ///
    /// On an error it replaces the previous proof, or retires the exemption
    /// — a proof always supersedes one. On an advisory diagnostic it is the
    /// "why" a renderer may expand. The severity never changes.
    #[must_use]
    pub fn with_witness(mut self, witness: WitnessChain) -> Self {
        self.witness = Some(Witness::Proven(witness));
        self
    }

    /// How much this diagnostic claims.
    #[must_use]
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// The proof or the exemption this diagnostic carries. Always `Some` for
    /// an error; `Some` for an advisory only when a chain was attached.
    #[must_use]
    pub fn witness(&self) -> Option<&Witness> {
        self.witness.as_ref()
    }

    /// The fact chain, when this diagnostic carries one.
    #[must_use]
    pub fn witness_chain(&self) -> Option<&WitnessChain> {
        match &self.witness {
            Some(Witness::Proven(chain)) => Some(chain),
            Some(Witness::LegacyExempt(_)) | None => None,
        }
    }

    /// The exemption this diagnostic reports under, when it is a legacy
    /// error — what a runtime inventory counts.
    #[must_use]
    pub fn exemption(&self) -> Option<&'static Exemption> {
        match self.witness {
            Some(Witness::LegacyExempt(exemption)) => Some(exemption),
            Some(Witness::Proven(_)) | None => None,
        }
    }
}
