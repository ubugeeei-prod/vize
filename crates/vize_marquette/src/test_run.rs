//! Release-bound test-run evidence.
//!
//! A `tests` deployment check is only trustworthy when its retained evidence
//! is immutable and bound to the exact candidate it verified. This module
//! defines the strict record, its canonical serialization, and the SHA-256
//! fingerprint admitted as `test-run:<sha256>` by deployment gates.
//!
//! The contract validates and binds retained facts. It does not operate a
//! runner, authenticate an external worker, guarantee production topology
//! equivalence, or prove that unreported tests ran; promotion infrastructure
//! must collect these facts from protected runner and report stores and
//! retain them immutably.

mod admission;
mod canonical;
mod check;
mod decision;
mod model;
mod transition;
mod validate;

#[cfg(test)]
pub(crate) mod model_tests;

pub use admission::{
    TEST_RUN_ADMISSION_PREFIX, TestRunCandidate, admit_test_run, parse_test_run_admission_id,
    test_run_admission_id,
};
pub use canonical::{CanonicalTestRunError, canonical_test_run_json, test_run_fingerprint};
pub use check::{
    TEST_RUN_CHECK_FORMAT, TEST_RUN_CHECK_FORMAT_VERSION, TestRunCheck, validate_test_run_check,
    verify_test_run_check,
};
pub use decision::{
    TEST_RUN_DENIAL_CODES, TestRunAdmissionDecision, TestRunDenialCode, decide_test_run_admission,
    test_run_denial_code,
};
pub use model::{
    TEST_RUN_EVIDENCE_FORMAT, TEST_RUN_EVIDENCE_FORMAT_VERSION, TestRunArtifact, TestRunEvidence,
    TestRunIsolation, TestRunRetainedEvidence, TestRunRunner, TestRunSelection,
    TestRunSuiteExecution, TestRunSuiteKind, TestRunSuiteOutcome, TestRunTargetExecution,
    TestRunTargetKind, TestRunVerification, TestRunVerificationOutcome,
};
pub use transition::{
    CanonicalTransitionError, TEST_RUN_TRANSITION_FORMAT, TEST_RUN_TRANSITION_FORMAT_VERSION,
    TEST_RUN_TRANSITION_MAX_ACCEPTED, TestRunRetainedDecision, TestRunRetainedDiagnostic,
    TestRunTransition, canonical_test_run_transition_json, test_run_transition_fingerprint,
    validate_test_run_transition, verify_test_run_transition,
};
pub use validate::{
    TEST_RUN_MAX_SHARDS, TEST_RUN_MAX_SUITES, TEST_RUN_MAX_TARGETS, validate_test_run,
};
