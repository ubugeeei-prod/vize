//! The S2 verifier: local invariant checks between passes, debug/CI only.
//!
//! The GHC `-dcore-lint` discipline, imported per
//! `davinci-road/prior-art-toolchains.md`: every check is **local** — a
//! node plus the facts it and its neighbours already declare (spans, side
//! tables, the arena stamp) — with no global inference and no fixpoint;
//! whole-program checks are barrier analyses and belong to P3-1's phase
//! validator, not here. Verifiers run between passes, after each fused
//! group minimum, so a failure names the offending pass instead of the
//! end of the pipeline ("checks the compiler's sanity, not yours").
//!
//! # The invariant catalogue
//!
//! Documented in `davinci-road/plan/folio-format.md` ("S2 verifier
//! invariants") — the format doc is where "canonical" is written down.
//!
//! | code   | rigor      | invariant                                        |
//! | ------ | ---------- | ------------------------------------------------ |
//! | S2V001 | structural | a span never runs backwards                      |
//! | S2V002 | structural | a nested span stays inside its owner's           |
//! | S2V003 | structural | every side-table `NodeId` resolves               |
//! | S2V004 | canonical  | `ui.if` owns at least one branch                 |
//! | S2V005 | canonical  | the leading branch carries a condition           |
//! | S2V006 | canonical  | an unconditional branch is the trailing branch   |
//!
//! [`Rigor`] follows `PassKind`: the structural set holds after every
//! pass; the canonical set additionally holds from the first
//! `MandatoryLowering` pass on — the kind that canonicalizes
//! (`crates/vize_davinci/src/pass/kind.rs`) — and diagnostic/optional
//! passes never change the rigor. Expr-ref liveness is the one check with
//! no code of its own: it reuses the P1-11 arena-generation stamp
//! ([`observer::VerifyObserver::check_live`]) rather than inventing a
//! second mechanism.
//!
//! # One walk, over the owned model
//!
//! The checks run over [`S2Folio`], the owned document model, in one
//! traversal — never a second walk per invariant. That is also why there
//! is no parallel arena-tree walk: the owned model is the single surface
//! the TS-18 fixture set and a live tree (via `S2Folio::of`, which
//! the folio observer's dump already needs) share, and a duplicated walk
//! is exactly the drift the P1-8 scanner split taught. Violations are
//! reported in **page order** — the order `print` emits lines — so
//! aggregation is deterministic and fixture-comparable.
//!
//! # Verification never ships (guardrail 5)
//!
//! These functions are pure and always compile, but the between-pass
//! integration ([`observer::VerifyObserver`]) is empty outside
//! `debug_assertions`: the release struct is a ZST with empty check
//! bodies, const-asserted in [`observer`]. A release pipeline makes zero
//! verifier calls.

use alloc::vec::Vec;
use core::fmt;

use vize_davinci::side_table::SideTable;
use vize_s0::{Span, String, cstr};

use crate::folio::S2Folio;

pub mod observer;
mod walk;

pub use observer::VerifyObserver;

/// Which invariant set applies, tracking the artifact's raw → canonical
/// state (`crates/vize_davinci/src/pass/canonical.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rigor {
    /// No mandatory-lowering pass has run: structural invariants only.
    Raw,
    /// A mandatory-lowering pass has canonicalized: the canonical-form
    /// invariants hold as well, and every later pass must preserve them.
    Canonical,
}

/// Which invariant a [`Violation`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationCode {
    /// S2V001 — a span runs backwards (`start > end`).
    SpanOrder,
    /// S2V002 — a nested line's span escapes its owner's.
    RegionNesting,
    /// S2V003 — a side table references a node the artifact does not
    /// number.
    IdResolution,
    /// S2V004 — `ui.if` owns no branch.
    EmptyIf,
    /// S2V005 — the leading branch of `ui.if` is unconditional.
    LeadingElse,
    /// S2V006 — an unconditional branch is not the trailing branch.
    BranchOrder,
}

impl ViolationCode {
    /// The stable code every rendering starts with.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ViolationCode::SpanOrder => "S2V001",
            ViolationCode::RegionNesting => "S2V002",
            ViolationCode::IdResolution => "S2V003",
            ViolationCode::EmptyIf => "S2V004",
            ViolationCode::LeadingElse => "S2V005",
            ViolationCode::BranchOrder => "S2V006",
        }
    }
}

/// One rejected invariant.
///
/// Not a `vize_davinci::diagnostic::Diagnostic` on purpose: the unified
/// channel carries what the **user** must see about their source, while a
/// verifier finding is a compiler-bug report about an artifact no user
/// wrote ("checks the compiler's sanity, not yours"). It renders as one
/// line — `{code} @{start}:{end} {message}`, canonical `en` locale — and
/// the TS-18 fixture set pins those renderings whole, no partial matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The invariant rejected.
    pub code: ViolationCode,
    /// The offending line's span; `@0:0` for a reference with no source
    /// anchor of its own (S2V003).
    pub span: Span,
    /// The full message. Owned, like every diagnostic that outlives its
    /// compile (the P1-11 contract).
    pub message: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} @{}:{} {}",
            self.code.as_str(),
            self.span.start,
            self.span.end,
            self.message
        )
    }
}

/// The code is a single-byte tag on every target; the violation is
/// message (24) + span (8) + tag, padded to 40 — a 64-bit figure of a
/// pointer-containing struct, so it carries the `vize_relief` guard.
const _: () = assert!(size_of::<ViolationCode>() == 1);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Violation>() == 40);

/// Check the tree-shaped invariants of `folio` at `rigor`, in one walk.
///
/// Returns every violation in page order (the order `print` emits lines);
/// an empty vector means the artifact holds the invariants. The side-table
/// check is [`verify_table`] — id resolution needs the table, which the
/// tree does not carry.
#[must_use]
pub fn verify(folio: &S2Folio, rigor: Rigor) -> Vec<Violation> {
    let mut out = Vec::new();
    walk::walk(folio, rigor, &mut out);
    out
}

/// Check that every [`NodeId`](vize_davinci::id::NodeId) `table`
/// references resolves in `folio`.
///
/// S2 ids are dense and page-ordered — every op line top to bottom, `attr`
/// and `branch` lines carry no id — so a reference resolves iff its index
/// is below the artifact's total op count, the same count the printed
/// `ops=` header states (`folio-format.md`, "Node numbering"). Dangling
/// references are reported in ascending id order, one violation each.
#[must_use]
pub fn verify_table<T>(folio: &S2Folio, table: &SideTable<T>) -> Vec<Violation> {
    let count = folio.op_count();
    let mut out = Vec::new();
    for (id, _) in table.sorted_entries() {
        if u64::from(id.index()) >= count {
            out.push(Violation {
                code: ViolationCode::IdResolution,
                span: Span::new(0, 0),
                message: cstr!("side table references {id}, but the artifact numbers {count} ops"),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Rigor, Violation, ViolationCode, verify, verify_table};
    use crate::folio::S2Folio;
    use alloc::vec;
    use vize_davinci::id::NodeId;
    use vize_davinci::side_table::SideTable;
    use vize_s0::{Span, cstr};

    #[test]
    fn codes_render_as_their_stable_strings() {
        assert_eq!(ViolationCode::SpanOrder.as_str(), "S2V001");
        assert_eq!(ViolationCode::RegionNesting.as_str(), "S2V002");
        assert_eq!(ViolationCode::IdResolution.as_str(), "S2V003");
        assert_eq!(ViolationCode::EmptyIf.as_str(), "S2V004");
        assert_eq!(ViolationCode::LeadingElse.as_str(), "S2V005");
        assert_eq!(ViolationCode::BranchOrder.as_str(), "S2V006");
    }

    #[test]
    fn a_violation_renders_as_one_line() {
        let violation = Violation {
            code: ViolationCode::EmptyIf,
            span: Span::new(3, 9),
            message: cstr!("`ui.if` owns no branch"),
        };
        assert_eq!(
            cstr!("{violation}").as_str(),
            "S2V004 @3:9 `ui.if` owns no branch"
        );
    }

    #[test]
    fn an_empty_page_holds_every_invariant_at_both_rigors() {
        let folio = S2Folio::default();
        assert_eq!(verify(&folio, Rigor::Raw), vec![]);
        assert_eq!(verify(&folio, Rigor::Canonical), vec![]);
    }

    #[test]
    fn an_empty_side_table_dangles_nothing() {
        let folio = S2Folio::default();
        let table: SideTable<u8> = SideTable::new();
        assert_eq!(verify_table(&folio, &table), vec![]);
    }

    #[test]
    fn a_dangling_reference_names_the_id_and_the_count_exactly() {
        let folio = S2Folio::default();
        let mut table = SideTable::new();
        table.insert(
            NodeId::from_index(4).expect("index 4 has an id"),
            "stale fact",
        );
        let violations = verify_table(&folio, &table);
        assert_eq!(
            violations,
            vec![Violation {
                code: ViolationCode::IdResolution,
                span: Span::new(0, 0),
                message: cstr!("side table references %4, but the artifact numbers 0 ops"),
            }]
        );
    }
}
