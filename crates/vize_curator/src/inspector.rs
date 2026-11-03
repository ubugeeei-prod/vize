//! Compiler inspector payload helpers.

mod diff;
pub mod feed;
mod graph;
mod imports;
mod payload;
mod product_capture;
mod stages;

#[cfg(test)]
mod tests;

#[cfg(test)]
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
pub use product_capture::{
    PRODUCT_STAGE_FEED_VERSION, ProductCaptureSource, product_capture_value,
};
