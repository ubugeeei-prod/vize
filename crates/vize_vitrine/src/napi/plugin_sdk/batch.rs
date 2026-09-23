//! One plugin's visit batch, its content key, and its reports (P4-16).
//!
//! A batch is **one** serialized crossing per plugin per document: the
//! nodes of the kinds the manifest visits (page order), the dense parent
//! array for every node (so ancestor walks never call back), and exactly the
//! demanded fact groups. JS answers with one report array; the host maps
//! each report's node id back to its own span, so a plugin can never report
//! a range the document does not have.

#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use serde::{Deserialize, Serialize};
use vize_davinci::fact::FactManager;

use super::document::{PluginDocument, PluginNode};
use super::error::HostError;
use super::facts::{demanded_facts, resolve_demands};

/// The batch wire format's version; part of every content key.
pub const BATCH_SCHEMA: u32 = 1;

/// Every S2 mnemonic a manifest may visit.
pub const NODE_KINDS: &[&str] = &[
    "ui.element",
    "ui.component",
    "ui.text",
    "ui.interpolation",
    "ui.comment",
    "ui.if",
    "ui.for",
    "ui.slot",
    "ui.bind",
    "ui.on",
    "ui.model",
    "ui.slot-content",
    "vue.directive",
    "vue.css-bind",
    "vue.sync",
    "vue.slot-scope",
    "vue.once",
    "vue.memo",
    "vue.show",
    "vue.html",
    "vue.text",
    "vue.cloak",
];

/// A plugin manifest as the host reads it.
#[derive(Debug, Clone, Copy)]
pub struct PluginSpec<'p> {
    pub name: &'p str,
    pub version: &'p str,
    /// The SDK's digest of the plugin's own code (rule sources).
    pub fingerprint: &'p str,
    /// Node kinds to batch; `None` batches every kind.
    pub visit: Option<&'p [String]>,
    pub demands: &'p [String],
}

#[derive(Serialize)]
struct Batch<'d> {
    schema: u32,
    plugin: &'d str,
    file: &'d str,
    parents: Vec<i64>,
    nodes: Vec<&'d PluginNode>,
    facts: serde_json::Map<String, serde_json::Value>,
}

/// A built batch: the JSON text and how many nodes it carries.
#[derive(Debug)]
pub struct BuiltBatch {
    pub json: String,
    pub nodes: u32,
}

/// Build `spec`'s batch over `document`, computing its demanded facts.
///
/// # Errors
///
/// An unknown node kind or fact name in the manifest.
pub fn build_batch(
    document: &PluginDocument,
    spec: &PluginSpec<'_>,
    manager: &mut FactManager<'_, PluginDocument>,
) -> Result<BuiltBatch, HostError> {
    let demand = validate_spec(spec)?;
    let visited = |node: &&PluginNode| {
        spec.visit
            .is_none_or(|kinds| kinds.iter().any(|k| k == node.kind))
    };
    let nodes: Vec<&PluginNode> = document.nodes.iter().filter(visited).collect();
    let parent = |node: &PluginNode| node.parent.map_or(-1, i64::from);
    let batch = Batch {
        schema: BATCH_SCHEMA,
        plugin: spec.name,
        file: &document.filename,
        parents: document.nodes.iter().map(parent).collect(),
        nodes,
        facts: demanded_facts(manager, document, demand),
    };
    let count = batch.nodes.len() as u32;
    let json =
        serde_json::to_string(&batch).map_err(|error| HostError::Split(error.to_string()))?;
    Ok(BuiltBatch { json, nodes: count })
}

/// Reject unknown manifest inputs before a cache hit can skip batch building.
pub fn validate_spec(spec: &PluginSpec<'_>) -> Result<vize_davinci::fact::Demand, HostError> {
    if let Some(kind) = spec
        .visit
        .into_iter()
        .flatten()
        .find(|k| !NODE_KINDS.contains(&k.as_str()))
    {
        return Err(HostError::UnknownKind {
            plugin: spec.name.to_owned(),
            kind: kind.clone(),
        });
    }
    resolve_demands(spec.name, spec.demands)
}

/// One report as a plugin returns it.
#[derive(Debug, Deserialize)]
struct Report {
    rule: String,
    node: u32,
    message: String,
}

/// One plugin diagnostic, anchored on the host's own span for the node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDiagnostic {
    pub rule_id: String,
    pub plugin: String,
    pub message: String,
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// Map a plugin's report array onto diagnostics, in (start, end, rule,
/// message) order.
///
/// # Errors
///
/// A malformed report array, or a report naming a node the document lacks.
pub fn diagnostics(
    document: &PluginDocument,
    plugin: &str,
    reports: &str,
) -> Result<Vec<PluginDiagnostic>, HostError> {
    let reports: Vec<Report> =
        serde_json::from_str(reports).map_err(|error| HostError::BadReports {
            plugin: plugin.to_owned(),
            detail: error.to_string(),
        })?;
    let mut out = Vec::with_capacity(reports.len());
    for report in reports {
        let Some(node) = document.nodes.get(report.node as usize) else {
            return Err(HostError::UnknownNode {
                plugin: plugin.to_owned(),
                rule: report.rule,
                node: report.node,
            });
        };
        let (line, column) = document.position(node.start);
        let (end_line, end_column) = document.position(node.end);
        out.push(PluginDiagnostic {
            rule_id: format!("{plugin}/{}", report.rule),
            plugin: plugin.to_owned(),
            message: report.message,
            start: node.start,
            end: node.end,
            line,
            column,
            end_line,
            end_column,
        });
    }
    sort(&mut out);
    Ok(out)
}

/// The host's one diagnostic order.
pub fn sort(diagnostics: &mut [PluginDiagnostic]) {
    diagnostics.sort_by(|a, b| {
        (a.start, a.end, &a.rule_id, &a.message).cmp(&(b.start, b.end, &b.rule_id, &b.message))
    });
}
