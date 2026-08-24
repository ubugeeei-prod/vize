#![deny(missing_docs)]

//! Whole-application health analysis contracts for Vize.
//!
//! **Experimental:** report serialization is versioned, but rule and scoring
//! APIs may evolve while the doctor integrates every Vize analysis graph.
//!
//! `vize_doctor` defines the stable finding, evidence, fix, provenance, and
//! health-assessment contracts shared by the CLI, editor, CI, Musea, framework,
//! and AI consumers. Analyzer crates produce findings through these types
//! instead of inventing reporter-specific diagnostics.
//!
//! # Design guarantees
//!
//! - Findings preserve primary and related authored source spans.
//! - Severity, confidence, impact, fix safety, and analysis cost are explicit.
//! - Reports are deterministically ranked and scored.
//! - Serialized property and enum names are language-neutral and versioned.
//! - Reports contain no timestamps or process-specific values.
//! - A blocking proven error remains blocking regardless of its health score.
//! - The optional `application-analysis` adapter is disabled by default.
//! - Enabled adapters reuse registered whole-project analysis without reparsing.
//! - Reporters are registered explicitly and never mutate global process state.
//! - Reporter descriptors are versioned, machine-readable, and deterministically ordered.
//! - AI context is vendor-neutral, explicitly source-fed, budgeted, and wire-validated.
//! - Capability cache keys are domain-separated and explain every invalidation boundary.
//! - Cacheable capability outputs validate provenance and their complete payload identity.
//!
//! # Example
//!
//! ```
//! use vize_doctor::{
//!     AnalysisProvenance, DoctorCategory, DoctorFinding, DoctorReport,
//!     FindingAssessment, FindingConfidence, FindingImpact, FindingSeverity,
//!     HealthPenalty, RuleCost, SourceLocation,
//! };
//!
//! let finding = DoctorFinding::new(
//!     "VIZE_DOCTOR_REACTIVITY_001",
//!     DoctorCategory::Correctness,
//!     FindingAssessment::new(
//!         FindingSeverity::Error,
//!         FindingConfidence::Certain,
//!         FindingImpact::High,
//!         HealthPenalty::new(30, "Proven stale state read"),
//!     ),
//!     SourceLocation::new("src/Counter.vue", 120, 132),
//!     "State is read outside its reactive owner",
//!     "Move the read into the derived computation that owns the dependency.",
//!     AnalysisProvenance::new("reactivity-graph", RuleCost::Low),
//! );
//! let report = DoctorReport::new("example", [finding]);
//!
//! assert!(report.summary().has_blocking_errors);
//! assert_eq!(report.findings().len(), 1);
//! ```

mod ai_context;
#[cfg(feature = "application-analysis")]
pub mod application_analysis;
mod cache_identity;
mod capability_execution;
mod capability_snapshot;
mod contract;
mod filter;
mod fingerprint;
mod model;
mod report;
mod reporter;

pub use ai_context::{
    AiContextBudget, AiContextError, AiContextOmissions, AiContextPacket, AiEditOperation,
    AiEditPlan, AiEvidenceEdge, AiEvidenceGraph, AiEvidenceNode, AiEvidenceNodeKind,
    AiEvidenceRelation, AiFindingContext, AiSourceSnippet, AiVerificationStep,
    DOCTOR_AI_CONTEXT_FORMAT_VERSION, build_ai_context,
};
pub use cache_identity::{
    CAPABILITY_CACHE_KEY_PREFIX, CapabilityCacheIdentity, CapabilityCacheIdentityError,
    CapabilityCacheInput, CapabilityCacheKey, CapabilityCacheKeyParseError, CapabilityInvalidation,
    CapabilityInvalidationTelemetry, DOCTOR_CAPABILITY_CACHE_IDENTITY_VERSION,
};
pub use capability_execution::{
    CapabilityExecutionCacheStatus, CapabilityExecutionError, CapabilityExecutionOutcome,
    CapabilityExecutionTelemetry, CapabilitySnapshotCache, MemoryCapabilitySnapshotCache,
    MemoryCapabilitySnapshotCacheError, execute_cached_capability,
};
pub use capability_snapshot::{
    CapabilitySnapshot, CapabilitySnapshotError, DOCTOR_CAPABILITY_SNAPSHOT_FORMAT_VERSION,
};
pub use filter::{DoctorFilter, DoctorFilterDimension, DoctorFilterError, DoctorFilterSpec};
pub use fingerprint::{
    CONTENT_FINGERPRINT_PREFIX, ContentFingerprint, ContentFingerprintParseError,
};
pub use model::{
    AnalysisProvenance, DEFAULT_UNAVAILABLE_FIX_REASON, DoctorCategory, DoctorFinding,
    EvidenceKind, FindingAssessment, FindingConfidence, FindingContext, FindingEvidence,
    FindingFix, FindingImpact, FindingSeverity, FixSafety, HealthPenalty, RelatedLocation,
    RuleCost, SourceLocation, SuppressionPolicy, TextEdit,
};
pub use report::{
    CategoryHealth, DOCTOR_REPORT_FORMAT_VERSION, DOCTOR_SCORING_VERSION, DoctorReport,
    DoctorSummary, FindingCounts,
};
pub use reporter::{
    DOCTOR_REPORTER_CONTRACT_VERSION, DoctorReporter, JsonReporter, ReporterAudience,
    ReporterCapability, ReporterContractError, ReporterDescriptor, ReporterError,
    ReporterErrorKind, ReporterFailure, ReporterOutput, ReporterReceipt, ReporterRegistrationError,
    ReporterSet, ReporterTransport, SarifMissingSourcePolicy, SarifReporter, SarifSource,
    SarifSourceError, render_report,
};
