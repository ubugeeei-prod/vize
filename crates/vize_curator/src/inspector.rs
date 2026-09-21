//! Compiler inspector payload helpers.

mod diff;
mod graph;
mod imports;
mod payload;
mod spolvero;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "inspector/tests_semantic.rs"]
mod tests_semantic;

pub use diff::{
    InspectorDiff, InspectorDiffLine, InspectorDiffStats, build_diff, build_line_diff, diff_stats,
};
pub use graph::{InspectorGraph, InspectorGraphEdge, InspectorGraphNode, build_graph};
pub use payload::{
    InspectorAgentReport, InspectorOptions, InspectorPayload, InspectorSourceFile, InspectorTarget,
    InspectorTemplateSyntax, build_agent_report, build_payload, build_playground_url,
    serialize_agent_report, serialize_payload,
};
pub use spolvero::{
    SpolveroFeed, SpolveroPage, SpolveroRemark, ladder_pages, s1_page, spolvero_value,
    spolvero_value_with_remarks, template_remarks,
};
