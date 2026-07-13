//! Opt-in cross-file lint analysis (provide/inject, reactivity flow, race risks).

#[cfg(test)]
#[path = "cross_file/artifact_tests.rs"]
mod artifact_tests;

use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use vize_atlas::Shared;
use vize_carton::{CompactString, FxHashMap, String, ToCompactString, cstr};
use vize_croquis_cf::{CrossFileDiagnostic, DiagnosticSeverity};
use vize_curator::complexity::render_complexity_markdown;
#[cfg(test)]
use vize_patina::Linter;
use vize_patina::{HelpLevel, LintDiagnostic, LintResult};

use super::{artifact_graph::LintArtifactGraph, pipeline::LintedFile};

pub(super) struct CrossFileLintOutput {
    pub(super) results: Vec<LintResult>,
    pub(super) provide_inject_tree: Option<String>,
    pub(super) complexity_report: Option<String>,
}

struct CrossFileInput<'a> {
    graph_index: usize,
    path: &'a Path,
    source: &'a str,
}

pub(super) fn apply_sfc_cross_file_lint(
    graph: &LintArtifactGraph,
    results: &mut [LintedFile],
    help_level: HelpLevel,
    include_tree: bool,
    include_complexity: bool,
) -> Option<String> {
    let targets: Vec<_> = results
        .iter()
        .enumerate()
        .filter(|(_, file)| is_sfc_cross_file_target(&file.path))
        .map(|(result_index, file)| (result_index, file.source_index))
        .collect();
    let inputs: Vec<_> = targets
        .iter()
        .map(|(result_index, graph_index)| {
            let file = &results[*result_index];
            CrossFileInput {
                graph_index: *graph_index,
                path: &file.path,
                source: &file.source,
            }
        })
        .collect();
    let output = build_cross_file_lint_output_from_graph(
        graph,
        &inputs,
        help_level,
        include_tree,
        include_complexity,
    );
    let report = combine_cross_file_report(
        output.provide_inject_tree.as_deref(),
        output.complexity_report.as_deref(),
    );

    for ((target_index, _), cross_result) in targets.into_iter().zip(output.results) {
        if let Some(file) = results.get_mut(target_index) {
            merge_lint_result(&mut file.result, cross_result);
        }
    }

    report
}

#[cfg(test)]
pub(super) fn build_cross_file_lint_output<S: AsRef<str>>(
    files: &[(PathBuf, S)],
    help_level: HelpLevel,
    include_tree: bool,
) -> CrossFileLintOutput {
    build_cross_file_lint_output_with_report(files, help_level, include_tree, false)
}

#[cfg(test)]
pub(super) fn build_cross_file_lint_output_with_report<S: AsRef<str>>(
    files: &[(PathBuf, S)],
    help_level: HelpLevel,
    include_tree: bool,
    include_complexity: bool,
) -> CrossFileLintOutput {
    let inputs: Vec<_> = files
        .iter()
        .enumerate()
        .map(|(graph_index, (path, source))| CrossFileInput {
            graph_index,
            path,
            source: source.as_ref(),
        })
        .collect();
    let graph = LintArtifactGraph::new(
        Shared::new(Linter::new()),
        vize_carton::config::VueVersion::V3,
        files
            .iter()
            .map(|(path, source)| (path.as_path(), source.as_ref())),
    )
    .expect("cross-file test graph must register");
    build_cross_file_lint_output_from_graph(
        &graph,
        &inputs,
        help_level,
        include_tree,
        include_complexity,
    )
}

fn build_cross_file_lint_output_from_graph(
    graph: &LintArtifactGraph,
    files: &[CrossFileInput<'_>],
    help_level: HelpLevel,
    include_tree: bool,
    include_complexity: bool,
) -> CrossFileLintOutput {
    if files.is_empty() {
        return CrossFileLintOutput {
            results: Vec::new(),
            provide_inject_tree: None,
            complexity_report: None,
        };
    }
    let artifact = graph
        .query_cross_file(files[0].graph_index)
        .unwrap_or_else(|error| panic!("Atlas cross-file analysis failed: {error}"))
        .artifact;
    let file_indexes: FxHashMap<_, _> = files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            (
                graph
                    .source(file.graph_index)
                    .expect("cross-file source must remain registered"),
                index,
            )
        })
        .collect();
    let mut results: Vec<_> = files
        .iter()
        .map(|file| LintResult {
            filename: file.path.to_string_lossy().to_compact_string(),
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        })
        .collect();

    for diagnostic in &artifact.result().diagnostics {
        let Some((source, start, end)) = artifact.diagnostic_range(diagnostic) else {
            continue;
        };
        let Some(index) = file_indexes.get(&source).copied() else {
            continue;
        };
        let source_len = files[index].source.len();
        results[index]
            .diagnostics
            .push(cross_file_diagnostic_to_lint(
                diagnostic, start, end, source_len, help_level,
            ));
    }

    for result in &mut results {
        result.error_count = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == vize_patina::Severity::Error)
            .count();
        result.warning_count = result.diagnostics.len() - result.error_count;
        result
            .diagnostics
            .sort_unstable_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
    }

    let provide_inject_tree = include_tree
        .then(|| artifact.provide_inject_tree().map(String::from))
        .flatten();
    let complexity_report = include_complexity.then(|| {
        render_complexity_markdown(
            &artifact.result().complexity_report,
            &artifact.result().complexity_hotspots,
        )
    });

    CrossFileLintOutput {
        results,
        provide_inject_tree,
        complexity_report,
    }
}

fn combine_cross_file_report(tree: Option<&str>, complexity: Option<&str>) -> Option<String> {
    let mut report = String::default();
    if let Some(tree) = tree {
        report.push_str(tree);
    }
    if let Some(complexity) = complexity {
        if !report.is_empty() {
            report.push('\n');
        }
        report.push_str(complexity);
    }
    (!report.is_empty()).then_some(report)
}

fn is_sfc_cross_file_target(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("vue" | "js" | "ts" | "jsx" | "tsx" | "mjs" | "mts" | "cjs" | "cts")
    )
}

fn cross_file_diagnostic_to_lint(
    diagnostic: &CrossFileDiagnostic,
    start: u32,
    raw_end: u32,
    source_len: usize,
    help_level: HelpLevel,
) -> LintDiagnostic {
    let source_len = source_len as u32;
    let start = start.min(source_len);
    let end = raw_end.max(start.saturating_add(1)).min(source_len);
    let message = cstr!("{}: {}", diagnostic.code(), diagnostic.message);
    let help = help_level.process(diagnostic.to_markdown().as_str());

    let mut lint = match diagnostic.severity {
        DiagnosticSeverity::Error => LintDiagnostic::error("cross-file", message, start, end),
        DiagnosticSeverity::Warning | DiagnosticSeverity::Info | DiagnosticSeverity::Hint => {
            LintDiagnostic::warn("cross-file", message, start, end)
        }
    };

    if let Some(help) = help {
        lint = lint.with_help(CompactString::new(help.as_str()));
    }

    lint
}

pub(super) fn merge_lint_result(target: &mut LintResult, mut extra: LintResult) {
    if extra.diagnostics.is_empty() {
        return;
    }

    target.error_count += extra.error_count;
    target.warning_count += extra.warning_count;
    target.diagnostics.append(&mut extra.diagnostics);
    target
        .diagnostics
        .sort_unstable_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn cross_file_complexity_report_mentions_hotspot_reason() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("App.vue");
        let child = dir.path().join("Child.vue");

        fs::write(&app, r#"<script setup lang="ts">
import { reactive } from 'vue'
import Child from './Child.vue'
const ready = true
const enabled = true
const fallback = false
const state = reactive({ count: 0 })
</script>
<template><Child v-if="ready && enabled" :item="state" /><Child v-if="fallback" :item="state" /></template>
"#).unwrap();
        fs::write(
            &child,
            r#"<script setup lang="ts">
defineProps<{ item: { count: number } }>()
</script>
"#,
        )
        .unwrap();

        let files = [&app, &child]
            .into_iter()
            .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
            .collect::<Vec<_>>();
        let output =
            build_cross_file_lint_output_with_report(&files, HelpLevel::Short, false, true);
        let report = output
            .complexity_report
            .as_deref()
            .expect("complexity report should be rendered");

        assert!(report.contains("## Cross-file Complexity"));
        assert!(report.contains("App.vue"));
        assert!(report.contains("template-control-flow"));
        assert!(report.contains("v-if=2"));
        assert!(report.contains("prop edges=2"));
    }

    #[test]
    fn combined_cross_file_report_keeps_tree_before_complexity() {
        let report = combine_cross_file_report(Some("tree"), Some("complexity")).unwrap();

        assert_eq!(report, "tree\ncomplexity");
    }
}
