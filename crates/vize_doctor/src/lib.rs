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
//! - Serialized property and enum names are language-neutral and versioned.
//! - Findings contain no timestamps or process-specific values.
//!
//! # Example
//!
//! ```
//! use vize_doctor::{
//!     AnalysisProvenance, DoctorCategory, DoctorFinding, FindingAssessment,
//!     FindingConfidence, FindingImpact, FindingSeverity, HealthPenalty,
//!     RuleCost, SourceLocation,
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
//! assert_eq!(finding.code, "VIZE_DOCTOR_REACTIVITY_001");
//! assert_eq!(finding.primary.path, "src/Counter.vue");
//! ```

mod model;

pub use model::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, EvidenceKind, FindingAssessment,
    FindingConfidence, FindingContext, FindingEvidence, FindingFix, FindingImpact, FindingSeverity,
    FixSafety, HealthPenalty, RelatedLocation, RuleCost, SourceLocation, SuppressionPolicy,
    TextEdit,
};
