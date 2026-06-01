//! Type checking functionality for Vue SFC.
//!
//! This module re-exports the type checking functionality from vize_canon.
//! The core implementation has been moved to vize_canon for better separation
//! of concerns - vize_vitrine is now purely a bindings layer.
//!
//! ## Migration Note
//!
//! If you were using types directly from this module, please update your
//! imports to use vize_canon instead:
//!
//! ```ignore
//! // Before
//! use vize_vitrine::typecheck::{type_check_sfc, TypeCheckOptions};
//!
//! // After
//! use vize_canon::{type_check_sfc, SfcTypeCheckOptions};
//! ```

// Re-export from canon for backwards compatibility
pub use vize_canon::{
    SfcRelatedLocation, SfcTypeCheckOptions, SfcTypeCheckResult, SfcTypeDiagnostic,
    SfcTypeSeverity, type_check_sfc, type_check_sfc_with_legacy_vue2,
};

// Type aliases for backwards compatibility
pub type TypeSeverity = SfcTypeSeverity;
pub type TypeDiagnostic = SfcTypeDiagnostic;
pub type RelatedLocation = SfcRelatedLocation;
pub type TypeCheckResult = SfcTypeCheckResult;
pub type TypeCheckOptions = SfcTypeCheckOptions;
