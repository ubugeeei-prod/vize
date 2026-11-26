//! Backend-neutral analyses over canonical L2 artifacts.

pub mod constant_expression;
pub mod for_facts;
pub mod fragment_facts;
pub mod if_facts;
pub mod model_facts;
pub mod pass_facts;
pub mod slot_facts;
pub mod static_facts;
pub mod template_complexity;
pub mod text_facts;

/// Joint compiler and tooling queries over a shared canonical region.
#[cfg(any(test, feature = "composed-template-analysis"))]
pub mod template {
    pub use super::static_facts::composed::{OwnerAnalysis, TemplateAnalysis, run};
}
