//! Backend-neutral analyses over canonical L2 artifacts.

pub mod constant_expression;
pub mod for_analysis;
pub mod fragment_analysis;
pub mod if_analysis;
pub mod model_analysis;
pub mod pass_analysis;
pub mod slot_analysis;
pub mod static_analysis;
pub mod template_complexity;
pub mod text_analysis;

/// Joint compiler and tooling queries over a shared canonical region.
#[cfg(any(test, feature = "composed-template-analysis"))]
pub mod template {
    pub use super::static_analysis::composed::{OwnerAnalysis, TemplateAnalysis, run};
}
