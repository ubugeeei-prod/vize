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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorDiff {
    pub lines: Vec<InspectorDiffLine>,
    pub stats: InspectorDiffStats,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorDiffLine {
    pub kind: &'static str,
    pub left_line: Option<usize>,
    pub right_line: Option<usize>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct InspectorDiffStats {
    pub additions: usize,
    pub removals: usize,
    pub unchanged: usize,
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

pub fn build_graph(files: &[InspectorSourceFile]) -> InspectorGraph {
    let normalized_files: Vec<_> = files
        .iter()
        .map(|file| (normalize_path(file.path.as_str()), file.source.as_str()))
        .collect();

    let nodes = normalized_files
        .iter()
        .map(|(path, source)| InspectorGraphNode {
            path: path.clone(),
            kind: file_kind(path.as_str()),
            is_entry: is_entry_path(path.as_str()),
            source_bytes: source.len(),
            source_lines: line_count(source),
        })
        .collect();

    let mut edges = Vec::new();
    for (path, source) in &normalized_files {
        for import in extract_imports(source) {
            if let Some(to) =
                resolve_import(&normalized_files, path.as_str(), import.specifier.as_str())
            {
                push_graph_edge(
                    &mut edges,
                    InspectorGraphEdge {
                        from: path.clone(),
                        to: to.clone(),
                        kind: import.kind,
                        specifier: import.specifier.clone(),
                    },
                );

                if to.ends_with(".vue")
                    && import.kind == "import"
                    && component_is_used(source, &import.locals)
                {
                    push_graph_edge(
                        &mut edges,
                        InspectorGraphEdge {
                            from: path.clone(),
                            to,
                            kind: "component",
                            specifier: import.specifier,
                        },
                    );
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

pub fn build_diff(left: &str, right: &str) -> InspectorDiff {
    let lines = build_line_diff(left, right);
    let stats = diff_stats(&lines);
    InspectorDiff { lines, stats }
}

pub fn build_line_diff(left: &str, right: &str) -> Vec<InspectorDiffLine> {
    let left_lines = split_diff_lines(left);
    let right_lines = split_diff_lines(right);
    let rows = left_lines.len() + 1;
    let cols = right_lines.len() + 1;
    let mut table = vec![vec![0usize; cols]; rows];

    for left_index in (0..left_lines.len()).rev() {
        for right_index in (0..right_lines.len()).rev() {
            let same_score =
                diff_line_match_weight(&left_lines[left_index], &right_lines[right_index]);
            let take_same = if same_score > 0 {
                table[left_index + 1][right_index + 1] + same_score
            } else {
                0
            };
            table[left_index][right_index] = take_same
                .max(table[left_index + 1][right_index])
                .max(table[left_index][right_index + 1]);
        }
    }

    let mut diff = Vec::new();
    let mut left_index = 0;
    let mut right_index = 0;

    while left_index < left_lines.len() && right_index < right_lines.len() {
        let same_score = diff_line_match_weight(&left_lines[left_index], &right_lines[right_index]);
        let take_same = if same_score > 0 {
            table[left_index + 1][right_index + 1] + same_score
        } else {
            0
        };
        if same_score > 0
            && take_same >= table[left_index + 1][right_index]
            && take_same >= table[left_index][right_index + 1]
        {
            diff.push(InspectorDiffLine {
                kind: "same",
                left_line: Some(left_index + 1),
                right_line: Some(right_index + 1),
                text: left_lines[left_index].clone(),
            });
            left_index += 1;
            right_index += 1;
        } else if table[left_index + 1][right_index] >= table[left_index][right_index + 1] {
            diff.push(InspectorDiffLine {
                kind: "remove",
                left_line: Some(left_index + 1),
                right_line: None,
                text: left_lines[left_index].clone(),
            });
            left_index += 1;
        } else {
            diff.push(InspectorDiffLine {
                kind: "add",
                left_line: None,
                right_line: Some(right_index + 1),
                text: right_lines[right_index].clone(),
            });
            right_index += 1;
        }
    }

    while left_index < left_lines.len() {
        diff.push(InspectorDiffLine {
            kind: "remove",
            left_line: Some(left_index + 1),
            right_line: None,
            text: left_lines[left_index].clone(),
        });
        left_index += 1;
    }

    while right_index < right_lines.len() {
        diff.push(InspectorDiffLine {
            kind: "add",
            left_line: None,
            right_line: Some(right_index + 1),
            text: right_lines[right_index].clone(),
        });
        right_index += 1;
    }

    diff
}

fn diff_line_match_weight(left: &str, right: &str) -> usize {
    if left != right {
        return 0;
    }

    let trimmed = left.trim();
    if trimmed.is_empty() {
        1
    } else if trimmed
        .chars()
        .any(|character| character.is_alphanumeric() || character == '_' || character == '$')
    {
        32 + trimmed.chars().take(80).count()
    } else {
        2 + trimmed.chars().take(8).count()
    }
}

pub fn diff_stats(lines: &[InspectorDiffLine]) -> InspectorDiffStats {
    lines
        .iter()
        .fold(InspectorDiffStats::default(), |mut stats, line| {
            match line.kind {
                "add" => stats.additions += 1,
                "remove" => stats.removals += 1,
                "same" => stats.unchanged += 1,
                _ => {}
            }
            stats
        })
}

fn split_diff_lines(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }

    value
        .replace("\r\n", "\n")
        .split('\n')
        .map(|line| line.to_compact_string())
        .collect()
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
    locals: Vec<String>,
}

fn extract_imports(source: &str) -> Vec<ImportEdge> {
    let mut imports = Vec::new();
    collect_static_imports(source, &mut imports);
    collect_dynamic_imports(source, &mut imports);
    imports
}

fn collect_static_imports(source: &str, imports: &mut Vec<ImportEdge>) {
    let mut offset = 0;
    while let Some(index) = source[offset..].find("import") {
        let import_start = offset + index;
        let clause_start = import_start + "import".len();
        if !is_word_boundary(source, import_start, clause_start) {
            offset = clause_start;
            continue;
        }

        let start = skip_whitespace(source, clause_start);
        if source[start..].starts_with('(') {
            offset = start + 1;
            continue;
        }
        if source[start..].starts_with('.') {
            offset = start + 1;
            continue;
        }

        if let Some((specifier, end)) = read_quoted(source, start) {
            imports.push(ImportEdge {
                specifier,
                kind: "import",
                locals: Vec::new(),
            });
            offset = end;
            continue;
        }

        let Some(from_index) = find_import_from(source, start) else {
            offset = clause_start;
            continue;
        };

        if let Some((specifier, end)) = read_quoted(source, from_index + "from".len()) {
            let clause = &source[start..from_index];
            imports.push(ImportEdge {
                specifier,
                kind: "import",
                locals: extract_import_locals(clause),
            });
            offset = end;
        } else {
            offset = from_index + "from".len();
        }
    }
}

fn collect_dynamic_imports(source: &str, imports: &mut Vec<ImportEdge>) {
    let mut offset = 0;
    while let Some(index) = source[offset..].find("import") {
        let import_start = offset + index;
        let import_end = import_start + "import".len();
        if !is_word_boundary(source, import_start, import_end) {
            offset = import_end;
            continue;
        }

        let start = skip_whitespace(source, import_end);
        if !source[start..].starts_with('(') {
            offset = import_end;
            continue;
        }

        if let Some((specifier, end)) = read_quoted(source, start + 1) {
            imports.push(ImportEdge {
                specifier,
                kind: "dynamic-import",
                locals: Vec::new(),
            });
            offset = end;
        } else {
            offset = start + 1;
        }
    }
}

fn skip_whitespace(source: &str, mut index: usize) -> usize {
    let bytes = source.as_bytes();
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    index
}

fn is_word_boundary(source: &str, start: usize, end: usize) -> bool {
    let bytes = source.as_bytes();
    let before = start
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .is_none_or(|byte| !is_identifier_byte(*byte));
    let after = bytes.get(end).is_none_or(|byte| !is_identifier_byte(*byte));
    before && after
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn find_import_from(source: &str, start: usize) -> Option<usize> {
    let mut offset = start;
    while let Some(index) = source[offset..].find("from") {
        let from_index = offset + index;
        let from_end = from_index + "from".len();
        if is_word_boundary(source, from_index, from_end) {
            return Some(from_index);
        }
        offset = from_end;
    }
    None
}

fn read_quoted(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    let mut index = skip_whitespace(source, start);
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

fn extract_import_locals(clause: &str) -> Vec<String> {
    let mut locals = Vec::new();
    let trimmed = clause.trim();
    if trimmed.starts_with("type ") {
        return locals;
    }

    if let Some(default_name) = trimmed.split(',').next().map(str::trim)
        && is_identifier(default_name)
    {
        locals.push(default_name.to_compact_string());
    }

    if let Some(namespace_name) = trimmed.strip_prefix("* as ").map(str::trim)
        && is_identifier(namespace_name)
    {
        locals.push(namespace_name.to_compact_string());
    }

    if let Some(named_start) = trimmed.find('{')
        && let Some(named_end) = trimmed[named_start + 1..].find('}')
    {
        let named = &trimmed[named_start + 1..named_start + 1 + named_end];
        for part in named.split(',') {
            let part = part.trim();
            if part.starts_with("type ") {
                continue;
            }
            let local = part
                .rsplit_once(" as ")
                .map(|(_, local)| local)
                .unwrap_or(part);
            if is_identifier(local.trim()) {
                locals.push(local.trim().to_compact_string());
            }
        }
    }

    locals
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|char| char == '_' || char == '$' || char.is_ascii_alphanumeric())
}

fn component_is_used(source: &str, locals: &[String]) -> bool {
    locals.iter().any(|local| {
        tag_is_used(source, local.as_str())
            || tag_is_used(source, to_kebab_case(local.as_str()).as_str())
    })
}

fn tag_is_used(source: &str, tag: &str) -> bool {
    if tag.is_empty() {
        return false;
    }

    let bytes = source.as_bytes();
    let tag_bytes = tag.as_bytes();
    let mut offset = 0;
    while let Some(index) = source[offset..].find('<') {
        let mut cursor = offset + index + 1;
        cursor = skip_whitespace(source, cursor);
        if bytes
            .get(cursor..cursor + tag_bytes.len())
            .is_some_and(|candidate| candidate == tag_bytes)
            && bytes
                .get(cursor + tag_bytes.len())
                .is_some_and(|byte| byte.is_ascii_whitespace() || matches!(*byte, b'/' | b'>'))
        {
            return true;
        }
        offset = cursor.saturating_add(1);
    }

    false
}

fn to_kebab_case(value: &str) -> String {
    let mut output = String::default();
    for (index, char) in value.chars().enumerate() {
        if char == '_' {
            output.push('-');
        } else if char.is_ascii_uppercase() {
            if index > 0 {
                output.push('-');
            }
            output.extend(char.to_lowercase());
        } else {
            output.push(char);
        }
    }
    output
}

fn push_graph_edge(edges: &mut Vec<InspectorGraphEdge>, edge: InspectorGraphEdge) {
    if !edges.contains(&edge) {
        edges.push(edge);
    }
}

fn resolve_import(files: &[(String, &str)], from: &str, specifier: &str) -> Option<String> {
    if !specifier.starts_with('.') {
        return None;
    }

    import_candidates(from, specifier)
        .into_iter()
        .find(|candidate| {
            files
                .iter()
                .any(|(path, _)| path.as_str() == candidate.as_str())
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
        source.split('\n').count()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        InspectorOptions, InspectorSourceFile, InspectorTarget, InspectorTemplateSyntax,
        build_agent_report, build_diff, build_graph, build_line_diff, build_payload,
        build_playground_url, serialize_agent_report, serialize_payload,
    };
    use vize_carton::cstr;

    #[test]
    fn builds_inspector_payload_json_and_url() {
        let payload = build_payload(
            InspectorTarget::Dom,
            InspectorOptions {
                custom_renderer: false,
                template_syntax: InspectorTemplateSyntax::Quirks,
            },
            vec![InspectorSourceFile {
                path: cstr!("src/App.vue"),
                source: cstr!("<template><div>msg</div></template>"),
            }],
        );
        let json = serialize_payload(&payload).expect("payload serializes");

        assert_eq!(
            json.as_str(),
            r#"{"version":1,"target":"dom","selectedFile":"src/App.vue","options":{"customRenderer":false,"templateSyntax":"quirks"},"files":[{"path":"src/App.vue","source":"<template><div>msg</div></template>"}]}"#
        );
        assert_eq!(
            build_playground_url("https://vizejs.dev/play/?foo=bar#old", json.as_str()).as_str(),
            "https://vizejs.dev/play/?foo=bar&tab=inspector#inspector=%7B%22version%22%3A1%2C%22target%22%3A%22dom%22%2C%22selectedFile%22%3A%22src%2FApp.vue%22%2C%22options%22%3A%7B%22customRenderer%22%3Afalse%2C%22templateSyntax%22%3A%22quirks%22%7D%2C%22files%22%3A%5B%7B%22path%22%3A%22src%2FApp.vue%22%2C%22source%22%3A%22%3Ctemplate%3E%3Cdiv%3Emsg%3C%2Fdiv%3E%3C%2Ftemplate%3E%22%7D%5D%7D"
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
    fn builds_graph_component_edges_for_used_imports() {
        let files = vec![
            InspectorSourceFile {
                path: cstr!("./src/App.vue"),
                source: cstr!(
                    "<script setup>import ChildCard from './ChildCard.vue'</script><template><child-card /></template>"
                ),
            },
            InspectorSourceFile {
                path: cstr!("src/ChildCard.vue"),
                source: cstr!("<template><span /></template>\n"),
            },
        ];

        let graph = build_graph(&files);

        assert_eq!(graph.nodes[0].path.as_str(), "src/App.vue");
        assert_eq!(graph.edges.len(), 2);
        assert_eq!(graph.edges[0].kind, "component");
        assert_eq!(graph.edges[1].kind, "import");
    }

    #[test]
    fn graph_ignores_import_meta_and_type_only_component_imports() {
        let files = vec![
            InspectorSourceFile {
                path: cstr!("src/App.vue"),
                source: cstr!(
                    "<script setup lang=\"ts\">
const mode = import.meta.env.MODE;
import type TypeOnly from './TypeOnly.vue';
import RuntimeOnly from './RuntimeOnly.vue';
</script>
<template><TypeOnly /><RuntimeOnly /></template>"
                ),
            },
            InspectorSourceFile {
                path: cstr!("src/TypeOnly.vue"),
                source: cstr!("<template><span /></template>\n"),
            },
            InspectorSourceFile {
                path: cstr!("src/RuntimeOnly.vue"),
                source: cstr!("<template><span /></template>\n"),
            },
        ];

        let graph = build_graph(&files);
        let component_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|edge| edge.kind == "component")
            .collect();

        assert_eq!(component_edges.len(), 1);
        assert_eq!(component_edges[0].to.as_str(), "src/RuntimeOnly.vue");
    }

    #[test]
    fn builds_line_diff_and_stats() {
        let diff = build_diff("one\ntwo\nthree", "one\nTWO\nthree\nfour");

        assert_eq!(diff.stats.additions, 2);
        assert_eq!(diff.stats.removals, 1);
        assert_eq!(diff.stats.unchanged, 2);
        assert_eq!(diff.lines.len(), 5);
        assert_eq!(diff.lines[0].kind, "same");
        assert_eq!(diff.lines[1].kind, "remove");
        assert_eq!(diff.lines[2].kind, "add");
        assert_eq!(diff.lines[4].right_line, Some(4));
    }

    #[test]
    fn line_diff_prefers_content_matches_over_empty_line_anchors() {
        let left = "\
import { defineComponent as _defineComponent } from 'vue'
import { computed, watch } from 'vue'

// Reactive Props Destructure
export default {}";
        let right = "\
import { defineComponent as _defineComponent } from 'vue'
import {
  openBlock as _openBlock,
} from 'vue'

import { computed, watch } from 'vue'

export default {}";

        let diff = build_line_diff(left, right);
        let matched_import = diff
            .iter()
            .find(|line| line.text == "import { computed, watch } from 'vue'")
            .expect("matching import line exists");

        assert_eq!(matched_import.kind, "same");
        assert_eq!(matched_import.left_line, Some(2));
        assert_eq!(matched_import.right_line, Some(6));
        assert!(!diff.iter().any(|line| {
            line.kind == "remove" && line.text == "import { computed, watch } from 'vue'"
        }));
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
                template_syntax: InspectorTemplateSyntax::Standard,
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
