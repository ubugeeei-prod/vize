//! Compiler inspector payload helpers.

use std::fmt::Write as _;
use vize_carton::{String, ToCompactString};

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspectorTarget {
    Dom,
    Ssr,
}

impl InspectorTarget {
    fn as_payload_str(self) -> &'static str {
        match self {
            Self::Dom => "dom",
            Self::Ssr => "ssr",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InspectorOptions {
    pub custom_renderer: bool,
    pub vue_parser_quirks: bool,
}

#[derive(Debug, Clone)]
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
    vue_parser_quirks: bool,
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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorGraph {
    pub nodes: Vec<InspectorGraphNode>,
    pub edges: Vec<InspectorGraphEdge>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorGraphNode {
    pub path: String,
    pub kind: &'static str,
    pub is_entry: bool,
    pub source_bytes: usize,
    pub source_lines: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorGraphEdge {
    pub from: String,
    pub to: String,
    pub kind: &'static str,
    pub specifier: String,
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
            vue_parser_quirks: options.vue_parser_quirks,
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
            vue_parser_quirks: payload.options.vue_parser_quirks,
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

pub fn build_graph(files: &[InspectorSourceFile]) -> InspectorGraph {
    let nodes = files
        .iter()
        .map(|file| InspectorGraphNode {
            path: file.path.clone(),
            kind: file_kind(file.path.as_str()),
            is_entry: is_entry_path(file.path.as_str()),
            source_bytes: file.source.len(),
            source_lines: line_count(file.source.as_str()),
        })
        .collect();

    let mut edges = Vec::new();
    for file in files {
        for import in extract_imports(file.source.as_str()) {
            if let Some(to) = resolve_import(files, file.path.as_str(), import.specifier.as_str()) {
                let edge = InspectorGraphEdge {
                    from: file.path.clone(),
                    to,
                    kind: import.kind,
                    specifier: import.specifier,
                };
                if !edges.contains(&edge) {
                    edges.push(edge);
                }
            }
        }
    }

    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.kind.cmp(right.kind))
            .then_with(|| left.specifier.cmp(&right.specifier))
    });

    InspectorGraph { nodes, edges }
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

#[derive(Debug)]
struct ImportEdge {
    specifier: String,
    kind: &'static str,
}

fn extract_imports(source: &str) -> Vec<ImportEdge> {
    let mut imports = Vec::new();
    collect_imports_after(source, "from", "import", &mut imports);
    collect_imports_after(source, "import(", "dynamic-import", &mut imports);
    imports
}

fn collect_imports_after(
    source: &str,
    marker: &str,
    kind: &'static str,
    imports: &mut Vec<ImportEdge>,
) {
    let mut offset = 0;
    while let Some(index) = source[offset..].find(marker) {
        let start = offset + index + marker.len();
        if let Some((specifier, end)) = read_quoted(source, start) {
            imports.push(ImportEdge { specifier, kind });
            offset = end;
        } else {
            offset = start;
        }
    }
}

fn read_quoted(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    let mut index = start;
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    let quote = *bytes.get(index)?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    index += 1;
    let value_start = index;
    while index < bytes.len() && bytes[index] != quote {
        index += 1;
    }
    if index >= bytes.len() {
        return None;
    }
    Some((String::from(&source[value_start..index]), index + 1))
}

fn resolve_import(files: &[InspectorSourceFile], from: &str, specifier: &str) -> Option<String> {
    if !specifier.starts_with('.') {
        return None;
    }

    import_candidates(from, specifier)
        .into_iter()
        .find(|candidate| {
            files
                .iter()
                .any(|file| file.path.as_str() == candidate.as_str())
        })
}

fn import_candidates(from: &str, specifier: &str) -> Vec<String> {
    let base = normalize_path(join_path(parent_path(from).as_str(), specifier).as_str());
    let mut candidates = vec![base.clone()];

    if !has_known_extension(base.as_str()) {
        for extension in [".vue", ".ts", ".tsx", ".js", ".jsx"] {
            let mut candidate = base.clone();
            candidate.push_str(extension);
            candidates.push(candidate);
        }

        for extension in ["/index.vue", "/index.ts", "/index.js"] {
            let mut candidate = base.clone();
            candidate.push_str(extension);
            candidates.push(candidate);
        }
    }

    candidates
}

fn parent_path(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_compact_string())
        .unwrap_or_default()
}

fn join_path(parent: &str, specifier: &str) -> String {
    if parent.is_empty() {
        specifier.to_compact_string()
    } else {
        let mut joined = parent.to_compact_string();
        joined.push('/');
        joined.push_str(specifier);
        joined
    }
}

fn normalize_path(path: &str) -> String {
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/").to_compact_string()
}

fn has_known_extension(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, extension)| matches!(extension, "vue" | "ts" | "tsx" | "js" | "jsx"))
}

fn file_kind(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, extension)| extension) {
        Some("vue") => "vue",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        _ => "other",
    }
}

fn is_entry_path(path: &str) -> bool {
    matches!(
        path.rsplit('/').next().unwrap_or(path),
        "App.vue" | "app.vue" | "index.vue" | "main.ts" | "main.js"
    )
}

fn line_count(source: &str) -> usize {
    if source.is_empty() {
        0
    } else {
        source.lines().count()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        InspectorOptions, InspectorSourceFile, InspectorTarget, build_agent_report, build_graph,
        build_payload, build_playground_url, serialize_agent_report, serialize_payload,
    };
    use vize_carton::cstr;

    #[test]
    fn builds_inspector_payload_json_and_url() {
        let payload = build_payload(
            InspectorTarget::Dom,
            InspectorOptions {
                custom_renderer: false,
                vue_parser_quirks: true,
            },
            vec![InspectorSourceFile {
                path: cstr!("src/App.vue"),
                source: cstr!("<template><div>msg</div></template>"),
            }],
        );
        let json = serialize_payload(&payload).expect("payload serializes");

        assert_eq!(
            json.as_str(),
            r#"{"version":1,"target":"dom","selectedFile":"src/App.vue","options":{"customRenderer":false,"vueParserQuirks":true},"files":[{"path":"src/App.vue","source":"<template><div>msg</div></template>"}]}"#
        );
        assert_eq!(
            build_playground_url("https://vizejs.dev/play/?foo=bar#old", json.as_str()).as_str(),
            "https://vizejs.dev/play/?foo=bar&tab=inspector#inspector=%7B%22version%22%3A1%2C%22target%22%3A%22dom%22%2C%22selectedFile%22%3A%22src%2FApp.vue%22%2C%22options%22%3A%7B%22customRenderer%22%3Afalse%2C%22vueParserQuirks%22%3Atrue%7D%2C%22files%22%3A%5B%7B%22path%22%3A%22src%2FApp.vue%22%2C%22source%22%3A%22%3Ctemplate%3E%3Cdiv%3Emsg%3C%2Fdiv%3E%3C%2Ftemplate%3E%22%7D%5D%7D"
        );
    }

    #[test]
    fn builds_graph_edges_for_relative_imports() {
        let files = vec![
            InspectorSourceFile {
                path: cstr!("src/App.vue"),
                source: cstr!("import Child from './Child.vue'\n"),
            },
            InspectorSourceFile {
                path: cstr!("src/Child.vue"),
                source: cstr!("<template><span /></template>\n"),
            },
        ];

        let graph = build_graph(&files);

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from.as_str(), "src/App.vue");
        assert_eq!(graph.edges[0].to.as_str(), "src/Child.vue");
        assert_eq!(graph.edges[0].kind, "import");
    }

    #[test]
    fn builds_agent_report_with_payload_url_and_graph() {
        let files = vec![
            InspectorSourceFile {
                path: cstr!("src/App.vue"),
                source: cstr!("import Child from './Child'\n"),
            },
            InspectorSourceFile {
                path: cstr!("src/Child.vue"),
                source: cstr!("<template><span /></template>\n"),
            },
        ];
        let payload = build_payload(
            InspectorTarget::Ssr,
            InspectorOptions {
                custom_renderer: true,
                vue_parser_quirks: false,
            },
            files.clone(),
        );
        let json = serialize_payload(&payload).expect("payload serializes");
        let url = build_playground_url("https://vizejs.dev/play/", json.as_str());
        let report = build_agent_report(payload, url, files);
        let report_json = serialize_agent_report(&report).expect("report serializes");

        assert!(report_json.contains(r#""schema": "vize.inspector.agent""#));
        assert!(report_json.contains(r#""target": "ssr""#));
        assert!(report_json.contains(r#""to": "src/Child.vue""#));
    }
}
