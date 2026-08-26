//! One durable atomic transition per release decision.
//!
//! A release gate must persist two facts together: the decision it made and
//! the anti-replay state that decision accepted. This module defines the
//! single canonical record carrying both, chained by canonical SHA-256
//! fingerprints, so the durable write is one atomic unit and no crash point
//! can separate a decision from its accepted state.
//!
//! # Host durability contract
//!
//! The contract defines validation, verification, and canonical bytes; the
//! host implements the durability primitive. A conforming host must write
//! the complete canonical bytes of the next transition to a temporary
//! location, flush them to durable storage, and then atomically rename or
//! commit them so that exactly one complete chain tip exists at every
//! instant. On recovery the host must load the tip, verify it against its
//! retained predecessor with [`verify_test_run_transition`], and discard —
//! never repair — a torn or partial record before deciding anything new.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vize_s0::String;

use crate::validate::DiagnosticSeverity;

use super::admission::TestRunCandidate;
use super::decision::TestRunDenialCode;

mod checks;
#[cfg(test)]
mod tests;

pub use checks::{validate_test_run_transition, verify_test_run_transition};

/// Serialized `format` marker for release-transition records.
///
/// Readers must reject any other value before trusting the record.
pub const TEST_RUN_TRANSITION_FORMAT: &str = "vize.test-run.transition";

/// Current serialized release-transition format.
///
/// Readers must reject a higher value until they explicitly support it.
pub const TEST_RUN_TRANSITION_FORMAT_VERSION: u32 = 1;

/// Maximum admission ids one transition may carry as accepted state.
pub const TEST_RUN_TRANSITION_MAX_ACCEPTED: usize = 4096;

/// One retained diagnostic inside a durable transition record.
///
/// The shape and serialization match live [`crate::ContractDiagnostic`]
/// values exactly; the retained form owns its code so persisted records can
/// be read back by any host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunRetainedDiagnostic {
    /// Stable machine-readable diagnostic code.
    pub code: String,
    /// Severity recorded for the diagnostic; any diagnostic denies.
    pub severity: DiagnosticSeverity,
    /// JSON-style path into the decided input.
    pub path: String,
    /// Human-readable explanation recorded with the decision.
    pub message: String,
}

/// One retained allow-or-deny decision inside a durable transition record.
///
/// The shape and serialization match live
/// [`crate::TestRunAdmissionDecision`] values exactly. Validation rejects a
/// retained decision whose `allowed` flag, denial codes, or diagnostic
/// ordering disagree with the published mapping, so a record cannot claim an
/// outcome its own diagnostics contradict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunRetainedDecision {
    /// Whether the release decision admitted the candidate.
    pub allowed: bool,
    /// Deduplicated denial causes sorted lexicographically; empty if allowed.
    pub denial_codes: Vec<TestRunDenialCode>,
    /// Complete diagnostics in the stable path, code, message order.
    pub diagnostics: Vec<TestRunRetainedDiagnostic>,
}

/// One durable atomic release transition.
///
/// The record binds the decision, the exact candidate and evidence it
/// decided, and the complete accepted anti-replay state after the decision
/// into one canonical document. `sequence` grows by exactly one per
/// transition and `previous` names the predecessor's canonical SHA-256
/// fingerprint, so a chain tip proves the entire decision history and the
/// accepted set can never drift from the decision that produced it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunTransition {
    /// Serialized format marker; always [`TEST_RUN_TRANSITION_FORMAT`].
    pub format: String,
    /// Serialized format version.
    ///
    /// Defaults to [`TEST_RUN_TRANSITION_FORMAT_VERSION`].
    #[serde(default = "default_test_run_transition_format_version")]
    pub format_version: u32,
    /// One-based position of this transition in its chain.
    pub sequence: u64,
    /// Canonical fingerprint of the predecessor; `None` only at genesis.
    pub previous: Option<String>,
    /// Millisecond-precision UTC instant the decision was made.
    pub decided_at: String,
    /// Exact candidate the decision was made for.
    pub candidate: TestRunCandidate,
    /// Exact `test-run:<sha256>` admission id the decision evaluated.
    pub evidence: String,
    /// Retained decision exactly as it was produced.
    pub decision: TestRunRetainedDecision,
    /// Complete anti-replay state after this transition: every admission id
    /// ever accepted in this chain, sorted and unique.
    pub accepted: Vec<String>,
}

const fn default_test_run_transition_format_version() -> u32 {
    TEST_RUN_TRANSITION_FORMAT_VERSION
}

/// Failure to serialize a release transition canonically.
#[derive(Debug)]
pub struct CanonicalTransitionError(serde_json::Error);

impl std::fmt::Display for CanonicalTransitionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("failed to serialize the release transition")
    }
}

impl std::error::Error for CanonicalTransitionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Serializes a release transition canonically.
///
/// Property order matches the record schema and the accepted state sorts
/// lexicographically after deduplication, so equivalent transitions produce
/// byte-identical JSON in every language. These are the exact bytes a host
/// must write atomically and the exact bytes the chain fingerprint covers.
/// Call validation before trusting the record; canonicalization does not
/// make an invalid record valid.
pub fn canonical_test_run_transition_json(
    transition: &TestRunTransition,
) -> Result<Vec<u8>, CanonicalTransitionError> {
    let mut canonical = transition.clone();
    canonical.accepted.sort_unstable();
    canonical.accepted.dedup();
    canonical.decision.denial_codes.sort_unstable();
    canonical.decision.denial_codes.dedup();
    serde_json::to_vec(&canonical).map_err(CanonicalTransitionError)
}

/// Returns a lowercase SHA-256 fingerprint of the canonical transition.
///
/// The fingerprint is the exact value the successor transition must name as
/// `previous`, forming the durable chain.
pub fn test_run_transition_fingerprint(
    transition: &TestRunTransition,
) -> Result<String, CanonicalTransitionError> {
    let bytes = canonical_test_run_transition_json(transition)?;
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(output)
}
