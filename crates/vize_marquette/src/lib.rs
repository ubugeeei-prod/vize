//! Versioned application marquettes for Vize.
//!
//! **Experimental:** serialized contracts are versioned, but Rust API
//! compatibility is not guaranteed until the marquette model is validated by
//! multiple target adapters.
//!
//! `vize_marquette` defines the language-neutral graph shared by the web,
//! native, desktop, terminal, backend, transport, testing, and gallery
//! surfaces. The crate intentionally owns contracts rather than runtime
//! implementations. Target-specific crates consume this model and report
//! their capabilities through it.
//!
//! # Design guarantees
//!
//! - Serialized names are stable and independent from Rust type names.
//! - Validation diagnostics have stable codes and authored graph paths.
//! - Canonical serialization is deterministic for equivalent contracts.
//! - Compatibility analysis distinguishes additive and breaking changes.
//! - Every reference is validated before an adapter may execute a plan.
//!
//! # Example
//!
//! ```
//! use vize_marquette::{
//!     ApplicationContract, Backend, BackendFamily, Environment,
//!     EnvironmentConsumer, RuntimeFamily, Target,
//! };
//!
//! let mut contract = ApplicationContract::new("example");
//! contract.targets.insert(Target::Web);
//! contract.environments.push(Environment::new(
//!     "client",
//!     Target::Web,
//!     EnvironmentConsumer::Client,
//!     RuntimeFamily::Browser,
//! ));
//! contract.backends.push(Backend::new(
//!     "server",
//!     BackendFamily::Rust,
//! ));
//!
//! assert!(contract.validate().is_empty());
//! ```

mod adapter;
mod canonical;
mod compatibility;
mod model;
mod test_run;
mod validate;

#[cfg(test)]
mod model_tests;

pub use adapter::{
    ADAPTER_CAPABILITY_FORMAT_VERSION, AdapterCapabilityCompatibilityReport,
    AdapterCapabilityDiagnostic, AdapterCapabilityDiagnosticCode, AdapterCapabilityManifest,
    AdapterCapabilityMismatch, AdapterCapabilityMismatchCode, AdapterCapabilityNegotiation,
    AdapterCapabilitySupport, NATIVE_ENGINE_CAPABILITY_IDS, NATIVE_ENGINE_CAPABILITY_VERSION,
    compare_adapter_capabilities, native_engine_capability_definitions,
    native_engine_capability_profile, negotiate_adapter_capabilities,
    validate_adapter_capability_manifest,
};
pub use canonical::{CanonicalContractError, canonical_json, contract_fingerprint};
pub use compatibility::{
    CompatibilityChange, CompatibilityChangeKind, CompatibilityInputDiagnostic,
    CompatibilityReport, compare_contracts,
};
pub use model::{
    ApplicationContract, Backend, BackendFamily, CONTRACT_FORMAT_VERSION, CapabilityDefinition,
    Environment, EnvironmentConsumer, Protocol, ProtocolFamily, RenderingMode, Route,
    RuntimeFamily, Target,
};
pub use test_run::{
    CanonicalTestRunError, CanonicalTransitionError, TEST_RUN_ADMISSION_PREFIX,
    TEST_RUN_CHECK_FORMAT, TEST_RUN_CHECK_FORMAT_VERSION, TEST_RUN_DENIAL_CODES,
    TEST_RUN_EVIDENCE_FORMAT, TEST_RUN_EVIDENCE_FORMAT_VERSION, TEST_RUN_MAX_SHARDS,
    TEST_RUN_MAX_SUITES, TEST_RUN_MAX_TARGETS, TEST_RUN_TRANSITION_FORMAT,
    TEST_RUN_TRANSITION_FORMAT_VERSION, TEST_RUN_TRANSITION_MAX_ACCEPTED, TestRunAdmissionDecision,
    TestRunArtifact, TestRunCandidate, TestRunCheck, TestRunDenialCode, TestRunEvidence,
    TestRunIsolation, TestRunRetainedDecision, TestRunRetainedDiagnostic, TestRunRetainedEvidence,
    TestRunRunner, TestRunSelection, TestRunSuiteExecution, TestRunSuiteKind, TestRunSuiteOutcome,
    TestRunTargetExecution, TestRunTargetKind, TestRunTransition, TestRunVerification,
    TestRunVerificationOutcome, admit_test_run, canonical_test_run_json,
    canonical_test_run_transition_json, decide_test_run_admission, parse_test_run_admission_id,
    test_run_admission_id, test_run_denial_code, test_run_fingerprint,
    test_run_transition_fingerprint, validate_test_run, validate_test_run_check,
    validate_test_run_transition, verify_test_run_check, verify_test_run_transition,
};
pub use validate::{ContractDiagnostic, DiagnosticSeverity, validate_contract};

/// Canonical JSON Schema for serialized application contracts.
///
/// The schema is embedded so CLI, editor, backend generator, and AI tooling
/// consumers can expose the exact contract without resolving a filesystem path.
pub const APPLICATION_CONTRACT_JSON_SCHEMA: &str =
    include_str!("../schema/application-contract.schema.json");

/// Canonical JSON Schema for serialized adapter capability manifests.
pub const ADAPTER_CAPABILITY_MANIFEST_JSON_SCHEMA: &str =
    include_str!("../schema/adapter-capability-manifest.schema.json");

/// Canonical JSON Schema for serialized test-run evidence records.
///
/// The schema is embedded so CLI, promotion, and AI tooling consumers can
/// expose the exact record contract without resolving a filesystem path.
pub const TEST_RUN_EVIDENCE_JSON_SCHEMA: &str =
    include_str!("../schema/test-run-evidence.schema.json");

/// Canonical JSON Schema for test-run admission decisions.
///
/// The schema declares the append-only denial-code vocabulary and the
/// structured decision shape every backend family must produce identically.
/// It is embedded so deployment gates and tooling can expose the exact
/// contract without resolving a filesystem path.
pub const TEST_RUN_ADMISSION_JSON_SCHEMA: &str =
    include_str!("../schema/test-run-admission.schema.json");

/// Canonical JSON Schema for retained release-bound tests checks.
///
/// The schema declares the record a release decision must retain as its
/// `tests` evidence: the exact `test-run:<sha256>` admission id, the six
/// candidate facts, and the independent observer. It replaces every generic
/// test-result reference and is embedded so deployment gates and tooling
/// can expose the exact contract without resolving a filesystem path.
pub const TEST_RUN_CHECK_JSON_SCHEMA: &str = include_str!("../schema/test-run-check.schema.json");

/// Canonical JSON Schema for durable release transitions.
///
/// The schema declares the single atomic record binding one release
/// decision to the complete accepted anti-replay state, chained by
/// canonical SHA-256 fingerprints, together with the durability contract a
/// conforming host must guarantee. It is embedded so deployment gates and
/// tooling can expose the exact contract without resolving a filesystem
/// path.
pub const TEST_RUN_TRANSITION_JSON_SCHEMA: &str =
    include_str!("../schema/test-run-transition.schema.json");

#[cfg(test)]
mod schema_tests {
    #[test]
    fn embedded_adapter_schema_tracks_the_current_format() {
        let schema: serde_json::Value =
            serde_json::from_str(super::ADAPTER_CAPABILITY_MANIFEST_JSON_SCHEMA).unwrap();
        assert_eq!(
            schema["$defs"]["manifest"]["properties"]["formatVersion"]["const"],
            super::ADAPTER_CAPABILITY_FORMAT_VERSION
        );
    }

    #[test]
    fn embedded_schema_is_valid_json_and_tracks_the_current_format() {
        let schema: serde_json::Value =
            serde_json::from_str(super::APPLICATION_CONTRACT_JSON_SCHEMA).unwrap();
        assert_eq!(
            schema["$defs"]["contract"]["properties"]["formatVersion"]["const"],
            1
        );
    }

    #[test]
    fn embedded_test_run_schema_tracks_the_current_format() {
        let schema: serde_json::Value =
            serde_json::from_str(super::TEST_RUN_EVIDENCE_JSON_SCHEMA).unwrap();
        let evidence = &schema["$defs"]["evidence"]["properties"];
        assert_eq!(evidence["format"]["const"], super::TEST_RUN_EVIDENCE_FORMAT);
        assert_eq!(
            evidence["formatVersion"]["const"],
            super::TEST_RUN_EVIDENCE_FORMAT_VERSION
        );
    }

    #[test]
    fn embedded_admission_schema_declares_the_exact_denial_vocabulary() {
        let schema: serde_json::Value =
            serde_json::from_str(super::TEST_RUN_ADMISSION_JSON_SCHEMA).unwrap();
        let declared = schema["$defs"]["denialCode"]["enum"].as_array().unwrap();
        let implemented = super::TEST_RUN_DENIAL_CODES
            .map(|code| serde_json::to_value(code).unwrap())
            .to_vec();
        assert_eq!(declared, &implemented);
    }

    #[test]
    fn embedded_check_schema_tracks_the_current_format() {
        let schema: serde_json::Value =
            serde_json::from_str(super::TEST_RUN_CHECK_JSON_SCHEMA).unwrap();
        let check = &schema["$defs"]["check"]["properties"];
        assert_eq!(check["format"]["const"], super::TEST_RUN_CHECK_FORMAT);
        assert_eq!(
            check["formatVersion"]["const"],
            super::TEST_RUN_CHECK_FORMAT_VERSION
        );
    }

    #[test]
    fn embedded_transition_schema_tracks_the_current_format() {
        let schema: serde_json::Value =
            serde_json::from_str(super::TEST_RUN_TRANSITION_JSON_SCHEMA).unwrap();
        let transition = &schema["$defs"]["transition"]["properties"];
        assert_eq!(
            transition["format"]["const"],
            super::TEST_RUN_TRANSITION_FORMAT
        );
        assert_eq!(
            transition["formatVersion"]["const"],
            super::TEST_RUN_TRANSITION_FORMAT_VERSION
        );
        assert_eq!(
            transition["accepted"]["maxItems"],
            super::TEST_RUN_TRANSITION_MAX_ACCEPTED
        );
    }
}
