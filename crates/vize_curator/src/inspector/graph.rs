//! Inspector module graph: nodes, edges, and path resolution.

use vize_s0::{String, ToCompactString};

use super::imports::analyze_file;
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
        .map(|file| {
            let path = normalize_path(file.path.as_str());
            let analysis = analyze_file(path.as_str(), file.source.as_str());
            (path, file.source.as_str(), analysis)
        })
        .collect();

    let nodes = normalized_files
        .iter()
        .map(|(path, source, _)| InspectorGraphNode {
            path: path.clone(),
            kind: file_kind(path.as_str()),
            is_entry: is_entry_path(path.as_str()),
            source_bytes: source.len(),
            source_lines: line_count(source),
        })
        .collect();

    let mut edges = Vec::new();
    for (path, _, analysis) in &normalized_files {
        for import in &analysis.imports {
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

                if import.kind == "import"
                    && is_component_module_path(to.as_str())
                    && component_is_used(&analysis.template_used_ids, &import.locals)
                {
                    push_graph_edge(
                        &mut edges,
                        InspectorGraphEdge {
                            from: path.clone(),
                            to,
                            kind: "component",
                            specifier: import.specifier.clone(),
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

fn component_is_used(template_used_ids: &vize_s0::FxHashSet<String>, locals: &[String]) -> bool {
    locals
        .iter()
        .any(|local| template_used_ids.contains(local.as_str()))
}

fn push_graph_edge(edges: &mut Vec<InspectorGraphEdge>, edge: InspectorGraphEdge) {
    if !edges.contains(&edge) {
        edges.push(edge);
    }
}

fn resolve_import(
    files: &[(String, &str, super::imports::FileAnalysis)],
    from: &str,
    specifier: &str,
) -> Option<String> {
    if !specifier.starts_with('.') {
        return None;
    }

    import_candidates(from, specifier)
        .into_iter()
        .find(|candidate| {
            files
                .iter()
                .any(|(path, _, _)| path.as_str() == candidate.as_str())
        })
}

fn import_candidates(from: &str, specifier: &str) -> Vec<String> {
    let base = normalize_path(join_path(parent_path(from).as_str(), specifier).as_str());
    let mut candidates = vec![base.clone()];

    if !has_known_extension(base.as_str()) {
        for extension in [
            ".vue", ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs",
        ] {
            let mut candidate = base.clone();
            candidate.push_str(extension);
            candidates.push(candidate);
        }

        for extension in [
            "/index.vue",
            "/index.ts",
            "/index.tsx",
            "/index.mts",
            "/index.cts",
            "/index.js",
            "/index.jsx",
            "/index.mjs",
            "/index.cjs",
        ] {
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
    path.rsplit_once('.').is_some_and(|(_, extension)| {
        matches!(
            extension,
            "vue" | "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs"
        )
    })
}

fn is_component_module_path(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, extension)| matches!(extension, "vue" | "tsx" | "jsx"))
}

fn file_kind(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, extension)| extension) {
        Some("vue") => "vue",
        Some("ts") | Some("tsx") | Some("mts") | Some("cts") => "typescript",
        Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => "javascript",
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
