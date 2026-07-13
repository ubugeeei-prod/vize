//! Atlas products for inspector source analysis and complete agent reports.

#[path = "artifact/report.rs"]
mod report;
#[path = "artifact/source.rs"]
mod source;

pub use report::{
    InspectorAgentReportProduct, InspectorAgentReportProvider, InspectorAgentRequest,
    InspectorAgentRequestInput, register_inspector_atlas_providers,
};
pub use source::{InspectorSourceAnalysis, InspectorSourceAnalysisProduct};

pub(super) use report::InspectorReportGraph;
pub(super) use source::analyze_source_compatibility;

#[cfg(test)]
#[path = "artifact/tests.rs"]
mod tests;
