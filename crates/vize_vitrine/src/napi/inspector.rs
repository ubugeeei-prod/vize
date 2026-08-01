//! N-API bindings for compiler inspector helpers.

#![allow(clippy::disallowed_types)]

use napi::{Error, Result, Status};
use napi_derive::napi;
use std::path::Path;
use vize_curator::inspector::{InspectorSourceFile, build_graph};

#[napi(object)]
pub struct InspectorSourceFileNapi {
    pub path: String,
    pub source: String,
}

#[napi(object)]
pub struct InspectorGraphNodeNapi {
    pub path: String,
    pub kind: String,
    pub is_entry: bool,
    pub source_bytes: u32,
    pub source_lines: u32,
}

#[napi(object)]
pub struct InspectorGraphEdgeNapi {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub specifier: String,
}

#[napi(object)]
pub struct InspectorGraphNapi {
    pub nodes: Vec<InspectorGraphNodeNapi>,
    pub edges: Vec<InspectorGraphEdgeNapi>,
}

#[napi(js_name = "buildInspectorGraph")]
pub fn build_inspector_graph(files: Vec<InspectorSourceFileNapi>) -> InspectorGraphNapi {
    let files = files
        .into_iter()
        .map(|file| InspectorSourceFile {
            path: file.path.into(),
            source: file.source.into(),
        })
        .collect::<Vec<_>>();

    build_graph(&files).into()
}

/// Resolve lint severities and their winning config items with CLI-identical glob semantics.
#[napi(js_name = "inspectLintPlan")]
pub fn inspect_lint_plan(plan: String, root: String, files: Vec<String>) -> Result<String> {
    let plan = serde_json::from_str::<vize::lint_plan::InspectorLintPlan>(&plan)
        .map_err(|error| Error::new(Status::InvalidArg, format!("Invalid lint plan: {error}")))?;
    serde_json::to_string(&vize::lint_plan::inspect_lint_plan(
        &plan,
        Path::new(&root),
        &files.into_iter().map(Into::into).collect::<Vec<_>>(),
    ))
    .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))
}

impl From<vize_curator::inspector::InspectorGraph> for InspectorGraphNapi {
    fn from(graph: vize_curator::inspector::InspectorGraph) -> Self {
        Self {
            nodes: graph.nodes.into_iter().map(Into::into).collect(),
            edges: graph.edges.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<vize_curator::inspector::InspectorGraphNode> for InspectorGraphNodeNapi {
    fn from(node: vize_curator::inspector::InspectorGraphNode) -> Self {
        Self {
            path: node.path.into(),
            kind: node.kind.into(),
            is_entry: node.is_entry,
            source_bytes: node.source_bytes as u32,
            source_lines: node.source_lines as u32,
        }
    }
}

impl From<vize_curator::inspector::InspectorGraphEdge> for InspectorGraphEdgeNapi {
    fn from(edge: vize_curator::inspector::InspectorGraphEdge) -> Self {
        Self {
            from: edge.from.into(),
            to: edge.to.into(),
            kind: edge.kind.into(),
            specifier: edge.specifier.into(),
        }
    }
}
