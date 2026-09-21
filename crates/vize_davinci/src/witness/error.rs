//! [`WitnessError`] — why a witness link does not verify.

use crate::diagnostic::{Verdict, WitnessKey};
use crate::fact::FactError;
use crate::pass::AnalysisId;
use vize_s0::Span;

/// Why a witness chain does not verify: the first failing link, by index in
/// proof order, with the values that disagree — every variant is an exact
/// oracle for a forged-witness fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WitnessError {
    /// The link names a group no [`WitnessCheck`](super::WitnessCheck) is
    /// registered for — a group that cannot back a witness.
    UnknownGroup { link: usize, group: AnalysisId },
    /// Reading the group through the producer's view failed: the producer
    /// did not declare it ([`FactError::Undeclared`], debug builds), no run
    /// computed it, or another group type owns its id.
    Fact { link: usize, error: FactError },
    /// The link's key has a shape the group's key type cannot take.
    KeyShape {
        link: usize,
        group: AnalysisId,
        key: WitnessKey,
    },
    /// No fact is stored under the link's key.
    MissingKey {
        link: usize,
        group: AnalysisId,
        key: WitnessKey,
    },
    /// The fact exists but is about another span than the link claims.
    SpanMismatch {
        link: usize,
        group: AnalysisId,
        fact: Span,
        witness: Span,
    },
    /// The fact exists at the claimed span but is not proven — the
    /// error-on-maybe the doctrine forbids, caught at run time.
    NotProven {
        link: usize,
        group: AnalysisId,
        verdict: Verdict,
    },
}
