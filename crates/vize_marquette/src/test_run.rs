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

mod canonical;
mod model;
mod validate;

#[cfg(test)]
pub(crate) mod model_tests;

pub use canonical::{CanonicalTestRunError, canonical_test_run_json, test_run_fingerprint};
pub use model::{
    TEST_RUN_EVIDENCE_FORMAT, TEST_RUN_EVIDENCE_FORMAT_VERSION, TestRunArtifact, TestRunEvidence,
    TestRunIsolation, TestRunRetainedEvidence, TestRunRunner, TestRunSelection,
    TestRunSuiteExecution, TestRunSuiteKind, TestRunSuiteOutcome, TestRunTargetExecution,
    TestRunTargetKind, TestRunVerification, TestRunVerificationOutcome,
};
pub use validate::{
    TEST_RUN_MAX_SHARDS, TEST_RUN_MAX_SUITES, TEST_RUN_MAX_TARGETS, validate_test_run,
};
