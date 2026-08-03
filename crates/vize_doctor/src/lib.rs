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

#[cfg(feature = "application-analysis")]
pub mod application_analysis;
mod model;
mod report;

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
