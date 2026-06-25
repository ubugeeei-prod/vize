//! Compiler inspector payload helpers.

mod diff;
mod graph;
mod imports;
mod payload;

#[cfg(test)]
mod tests;

pub use diff::{
    InspectorDiff, InspectorDiffLine, InspectorDiffStats, build_diff, build_line_diff, diff_stats,
};
pub use graph::{InspectorGraph, InspectorGraphEdge, InspectorGraphNode, build_graph};
pub use payload::{
    InspectorAgentReport, InspectorOptions, InspectorPayload, InspectorSourceFile, InspectorTarget,
    InspectorTemplateSyntax, build_agent_report, build_payload, build_playground_url,
    serialize_agent_report, serialize_payload,
};
