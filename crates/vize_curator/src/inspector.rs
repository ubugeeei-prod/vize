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
    LADDER_STEP_KEY, LADDER_WALK_KEY, LadderClock, LadderRun, LadderStep, SpolveroFeed,
    SpolveroPage, SpolveroRemark, l1_page, ladder_pages, ladder_profile, ladder_run,
    spolvero_value, spolvero_value_with_remarks, template_remarks,
};
