//! Inspector payload, agent report, and playground URL helpers.

use std::fmt::Write as _;
use vize_carton::{String, ToCompactString};

use super::graph::{InspectorGraph, build_graph, line_count};

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspectorTarget {
    Dom,
    Ssr,
    Vapor,
}

impl InspectorTarget {
    fn as_payload_str(self) -> &'static str {
        match self {
            Self::Dom => "dom",
            Self::Ssr => "ssr",
            Self::Vapor => "vapor",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspectorTemplateSyntax {
    #[default]
    Standard,
    Strict,
    Quirks,
}

#[derive(Debug, Clone, Copy)]
pub struct InspectorOptions {
    pub custom_renderer: bool,
    pub template_syntax: InspectorTemplateSyntax,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct InspectorSourceFile {
    pub path: String,
    pub source: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorPayload {
    version: u8,
    target: &'static str,
    selected_file: Option<String>,
    options: InspectorPayloadOptions,
    files: Vec<InspectorPayloadFile>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorPayloadOptions {
    custom_renderer: bool,
    template_syntax: InspectorTemplateSyntax,
}

#[derive(serde::Serialize)]
struct InspectorPayloadFile {
    path: String,
    source: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorAgentReport {
    schema: &'static str,
    version: u8,
    generated_by: &'static str,
    playground_url: String,
    summary: InspectorAgentSummary,
    graph: InspectorGraph,
    payload: InspectorPayload,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorAgentSummary {
    target: &'static str,
    selected_file: Option<String>,
    file_count: usize,
    source_bytes: usize,
    source_lines: usize,
    options: InspectorPayloadOptions,
}

pub fn build_payload(
    target: InspectorTarget,
    options: InspectorOptions,
    files: Vec<InspectorSourceFile>,
) -> InspectorPayload {
    let files: Vec<_> = files
        .into_iter()
        .map(|file| InspectorPayloadFile {
            path: file.path,
            source: file.source,
        })
        .collect();

    let selected_file = files.first().map(|file| file.path.clone());

    InspectorPayload {
        version: 1,
        target: target.as_payload_str(),
        selected_file,
        options: InspectorPayloadOptions {
            custom_renderer: options.custom_renderer,
            template_syntax: options.template_syntax,
        },
        files,
    }
}

pub fn build_agent_report(
    payload: InspectorPayload,
    playground_url: String,
    files: Vec<InspectorSourceFile>,
) -> InspectorAgentReport {
    let source_bytes = files.iter().map(|file| file.source.len()).sum();
    let source_lines = files
        .iter()
        .map(|file| line_count(file.source.as_str()))
        .sum();
    let graph = build_graph(&files);
    let summary = InspectorAgentSummary {
        target: payload.target,
        selected_file: payload.selected_file.clone(),
        file_count: payload.files.len(),
        source_bytes,
        source_lines,
        options: InspectorPayloadOptions {
            custom_renderer: payload.options.custom_renderer,
            template_syntax: payload.options.template_syntax,
        },
    };

    InspectorAgentReport {
        schema: "vize.inspector.agent",
        version: 1,
        generated_by: "vize_curator",
        playground_url,
        summary,
        graph,
        payload,
    }
}

pub fn serialize_payload(payload: &InspectorPayload) -> Result<String, serde_json::Error> {
    serde_json::to_string(payload).map(|json| json.to_compact_string())
}

pub fn serialize_agent_report(report: &InspectorAgentReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report).map(|json| json.to_compact_string())
}

pub fn build_playground_url(base: &str, payload_json: &str) -> String {
    let base_without_hash = base.split('#').next().unwrap_or(base);
    let separator = if base_without_hash.contains('?') {
        if base_without_hash.ends_with('?') || base_without_hash.ends_with('&') {
            ""
        } else {
            "&"
        }
    } else {
        "?"
    };

    let mut url = String::default();
    url.push_str(base_without_hash);
    url.push_str(separator);
    url.push_str("tab=inspector#inspector=");
    url.push_str(percent_encode(payload_json).as_str());
    url
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::default();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}
