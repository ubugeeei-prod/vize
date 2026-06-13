//! SFC type checking functionality for Vue Single File Components.
//!
//! This module provides AST-based type analysis for Vue SFCs.
//! It leverages croquis for semantic analysis and scope tracking.
//!
//! ## Features
//!
//! - Props type validation (defineProps)
//! - Emits type validation (defineEmits)
//! - Template binding validation (undefined references)
//! - Virtual TypeScript generation with scope-aware code
//!
//! ## Architecture
//!
//! ```text
//! Vue SFC (.vue)
//!     |
//!     v
//! +-------------------------------------+
//! |  vize_atelier_sfc::parse_sfc        |
//! +-------------------------------------+
//!     |
//!     v
//! +-------------------------------------+
//! |  vize_croquis::Drawer               |
//! |  - Script analysis (bindings)       |
//! |  - Template analysis (scopes)       |
//! |  - Macro tracking (defineProps)     |
//! +-------------------------------------+
//!     |
//!     v
//! +-------------------------------------+
//! |  type_check_sfc()                   |
//! |  - check_props_typing()             |
//! |  - check_emits_typing()             |
//! |  - check_template_bindings()        |
//! |  - generate_virtual_ts_with_scopes()|
//! +-------------------------------------+
//! ```

mod analysis;
mod checks;
mod runner;
#[cfg(test)]
mod tests;
mod virtual_ts;

pub use analysis::{
    SfcRelatedLocation, SfcTypeCheckOptions, SfcTypeCheckResult, SfcTypeDiagnostic, SfcTypeSeverity,
};
pub use runner::{
    type_check_sfc, type_check_sfc_with_legacy_vue2, type_check_sfc_with_options_api,
};
