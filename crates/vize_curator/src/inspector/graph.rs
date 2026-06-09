//! Inspector module graph: nodes, edges, and path resolution.

use vize_carton::{String, ToCompactString};

use super::imports::{extract_imports, skip_whitespace};
use super::payload::InspectorSourceFile;

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

pub(super) fn line_count(source: &str) -> usize {
    if source.is_empty() {
        0
    } else {
        source.split('\n').count()
    }
}
