use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use vize_s0::String;

/// Serialized `format` marker for test-run evidence records.
///
/// Readers must reject any other value before trusting the record.
pub const TEST_RUN_EVIDENCE_FORMAT: &str = "vize.test-run.evidence";

/// Current serialized test-run evidence format.
///
/// Readers must reject a higher value until they explicitly support it.
pub const TEST_RUN_EVIDENCE_FORMAT_VERSION: u32 = 1;

/// Immutable, content-addressed reference to retained evidence.
///
/// The reference names the retained content by SHA-256 so a green label can
/// never drift away from the bytes that were verified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunRetainedEvidence {
    /// Content-addressed retrieval reference in `sha256:<64 hex>` form.
    pub reference: String,
    /// Lowercase SHA-256 fingerprint of the retained content.
    pub fingerprint: String,
}

impl TestRunRetainedEvidence {
    /// Creates a retained-evidence binding.
    pub fn new(reference: impl Into<String>, fingerprint: impl Into<String>) -> Self {
        Self {
            reference: reference.into(),
            fingerprint: fingerprint.into(),
        }
    }
}

/// Exact release artifact that the recorded test executions exercised.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunArtifact {
    /// Stable artifact identifier.
    pub id: String,
    /// Lowercase SHA-256 fingerprint of the artifact bytes.
    pub fingerprint: String,
    /// Exact artifact size in bytes; must be at least one byte.
    pub size_bytes: u64,
}

/// Isolation level of the runner that executed the tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestRunIsolation {
    /// Runner reserved for one verified workload at a time.
    Dedicated,
    /// Runner created for this invocation and destroyed afterwards.
    Ephemeral,
}

/// Authenticated runner that executed the recorded test run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunRunner {
    /// Stable runner identity.
    pub identity: String,
    /// Retained evidence for the runner's authentication.
    pub authentication_evidence: TestRunRetainedEvidence,
    /// Isolation level the runner guaranteed for this invocation.
    pub isolation: TestRunIsolation,
    /// Lowercase SHA-256 fingerprint of the exact invocation.
    pub invocation_fingerprint: String,
    /// Retained evidence describing the execution environment.
    pub environment_evidence: TestRunRetainedEvidence,
    /// Lowercase SHA-256 fingerprint of the execution environment.
    pub environment_fingerprint: String,
}

/// Complete candidate-selected target and suite coverage for the run.
///
/// Recorded executions must cover exactly these identifiers; anything less is
/// an undeclared omission and anything more is an undeclared execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunSelection {
    /// Selected target identifiers, sorted and unique.
    pub target_ids: BTreeSet<String>,
    /// Selected suite identifiers, sorted and unique.
    pub suite_ids: BTreeSet<String>,
}

/// Kind of user-visible target a test execution ran against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestRunTargetKind {
    /// Standards-based web output.
    Web,
    /// Native mobile or device application output.
    Native,
    /// Desktop application output.
    Desktop,
    /// Terminal application output.
    Terminal,
    /// Server or backend output.
    Server,
}

/// One executed target of the release candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunTargetExecution {
    /// Stable target identifier from the candidate selection.
    pub id: String,
    /// Kind of target that was exercised.
    pub kind: TestRunTargetKind,
    /// Environment identifier the target executed in.
    pub environment: String,
}

/// Semantic kind of one executed suite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestRunSuiteKind {
    /// Isolated unit tests.
    Unit,
    /// Cross-component integration tests.
    Integration,
    /// Contract and conformance tests.
    Contract,
    /// Full end-to-end tests.
    EndToEnd,
    /// Accessibility tests.
    Accessibility,
    /// Visual regression tests.
    Visual,
    /// Performance and budget tests.
    Performance,
    /// Fault-injection and resilience tests.
    Resilience,
    /// Installation tests.
    Installation,
    /// Upgrade tests.
    Upgrade,
    /// Data or schema migration tests.
    Migration,
}

/// Final result of one executed suite shard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestRunSuiteOutcome {
    /// Every selected test passed.
    Passed,
    /// At least one test failed.
    Failed,
    /// Execution stopped before completion.
    Cancelled,
}

/// One executed suite shard with its exact recorded results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunSuiteExecution {
    /// Stable suite identifier from the candidate selection.
    pub id: String,
    /// Target this shard executed against.
    pub target_id: String,
    /// Semantic suite kind.
    pub kind: TestRunSuiteKind,
    /// One-based shard index within `shard_count`.
    pub shard_index: u32,
    /// Total number of shards the suite was split into.
    pub shard_count: u32,
    /// Final shard outcome.
    pub outcome: TestRunSuiteOutcome,
    /// Number of passed tests.
    pub passed: u32,
    /// Number of failed tests.
    pub failed: u32,
    /// Number of skipped tests.
    pub skipped: u32,
    /// Number of retried tests; retries must always be declared.
    pub retries: u32,
    /// Wall-clock shard duration in milliseconds.
    pub duration_ms: u64,
    /// Lowercase SHA-256 fingerprint of the exact shard invocation.
    pub invocation_fingerprint: String,
    /// Retained machine-readable report evidence.
    pub report: TestRunRetainedEvidence,
    /// Retained execution log evidence.
    pub log: TestRunRetainedEvidence,
}

/// Final result of the independent verification pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestRunVerificationOutcome {
    /// The verifier accepted the complete run.
    Accepted,
    /// The verifier rejected the run.
    Rejected,
}

/// Independent verification summary over every recorded execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunVerification {
    /// Stable identity of the independent verifier.
    pub verifier: String,
    /// Millisecond-precision UTC time the verification completed.
    pub completed_at: String,
    /// Final verification outcome.
    pub outcome: TestRunVerificationOutcome,
    /// Exact number of recorded target executions.
    pub target_count: u32,
    /// Exact number of recorded suite executions.
    pub suite_count: u32,
    /// Total passed tests across every recorded suite execution.
    pub passed: u32,
    /// Total failed tests across every recorded suite execution.
    pub failed: u32,
    /// Total skipped tests across every recorded suite execution.
    pub skipped: u32,
    /// Total retried tests across every recorded suite execution.
    pub retries: u32,
    /// Retained evidence produced by the verification pass.
    pub evidence: TestRunRetainedEvidence,
}

/// Complete, versioned test-run evidence for one release candidate.
///
/// The record binds application, environment, application contract, source
/// revision, release, and exact artifact fingerprint to bounded target and
/// suite executions, so a `tests` check can only be satisfied by retained,
/// immutable facts instead of a mutable label or path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunEvidence {
    /// Serialized format marker; always [`TEST_RUN_EVIDENCE_FORMAT`].
    pub format: String,
    /// Serialized format version.
    ///
    /// Defaults to [`TEST_RUN_EVIDENCE_FORMAT_VERSION`].
    #[serde(default = "default_test_run_format_version")]
    pub format_version: u32,
    /// Stable record identifier.
    pub id: String,
    /// Application the run verified.
    pub application: String,
    /// Deployment environment the run verified the candidate for.
    pub environment: String,
    /// Lowercase SHA-256 fingerprint of the application contract.
    pub contract_fingerprint: String,
    /// Exact source revision the candidate was built from.
    pub source_revision: String,
    /// Release the candidate belongs to.
    pub release: String,
    /// Exact artifact the recorded executions exercised.
    pub artifact: TestRunArtifact,
    /// Millisecond-precision UTC time the run started.
    pub started_at: String,
    /// Millisecond-precision UTC time the run completed.
    pub completed_at: String,
    /// Millisecond-precision UTC time the record expires.
    pub valid_until: String,
    /// Authenticated runner that executed the run.
    pub runner: TestRunRunner,
    /// Complete candidate-selected target and suite coverage.
    pub selection: TestRunSelection,
    /// Recorded target executions.
    pub targets: Vec<TestRunTargetExecution>,
    /// Recorded suite executions.
    pub suites: Vec<TestRunSuiteExecution>,
    /// Independent verification summary.
    pub verification: TestRunVerification,
}

const fn default_test_run_format_version() -> u32 {
    TEST_RUN_EVIDENCE_FORMAT_VERSION
}
