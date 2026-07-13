//! Opt-in cross-file lint analysis (provide/inject, reactivity flow, race risks).

#[cfg(test)]
#[path = "cross_file/artifact_tests.rs"]
mod artifact_tests;

use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use vize_atlas::{Compilation, Shared, SourceId};
use vize_carton::{CompactString, FxHashMap, String, ToCompactString, cstr};
use vize_croquis_cf::{
    CrossFileAnalysisArtifact, CrossFileAnalysisInput, CrossFileAnalysisProduct,
    CrossFileAnalysisRequest, CrossFileDiagnostic, CrossFileOptions, DiagnosticSeverity,
};
use vize_curator::complexity::render_complexity_markdown;
use vize_patina::{HelpLevel, LintDiagnostic, LintResult};

use super::pipeline::LintedFile;

pub(super) struct CrossFileLintOutput {
    pub(super) results: Vec<LintResult>,
    pub(super) provide_inject_tree: Option<String>,
    pub(super) complexity_report: Option<String>,
}

struct CrossFileInput<'a> {
    path: &'a Path,
    source: &'a str,
}

pub(super) fn apply_sfc_cross_file_lint(
    results: &mut [LintedFile],
    help_level: HelpLevel,
    include_tree: bool,
    include_complexity: bool,
) -> Option<String> {
    let targets: Vec<_> = results
        .iter()
        .enumerate()
        .filter(|(_, file)| is_sfc_cross_file_target(&file.path))
        .map(|(index, _)| index)
        .collect();
    let inputs: Vec<_> = targets
        .iter()
        .map(|index| {
            let file = &results[*index];
            CrossFileInput {
                path: &file.path,
                source: &file.source,
            }
        })
        .collect();
    let output = build_cross_file_lint_output_from_inputs(
        &inputs,
        help_level,
        include_tree,
        include_complexity,
    );
    let report = combine_cross_file_report(
        output.provide_inject_tree.as_deref(),
        output.complexity_report.as_deref(),
    );

    for (target_index, cross_result) in targets.into_iter().zip(output.results) {
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
        .map(|(path, source)| CrossFileInput {
            path,
            source: source.as_ref(),
        })
        .collect();
    build_cross_file_lint_output_from_inputs(&inputs, help_level, include_tree, include_complexity)
}

fn build_cross_file_lint_output_from_inputs(
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
    let root = std::env::current_dir().unwrap_or_default();
    let (artifact, file_indexes) = query_cross_file_artifact(files, root);
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

fn query_cross_file_artifact(
    files: &[CrossFileInput<'_>],
    root: std::path::PathBuf,
) -> (
    Shared<CrossFileAnalysisArtifact>,
    FxHashMap<SourceId, usize>,
) {
    let mut compilation = Compilation::new();
    vize_atelier_sfc::register_atlas_providers(&mut compilation)
        .expect("SFC cross-file providers must register");
    vize_atelier_jsx::register_atlas_providers(&mut compilation)
        .expect("JSX cross-file providers must register");
    vize_croquis_cf::register_atlas_provider(&mut compilation)
        .expect("Croquis cross-file provider must register");
    compilation
        .set_input::<CrossFileAnalysisInput>(
            CrossFileAnalysisRequest::new(patina_cross_file_options()).with_project_root(root),
        )
        .expect("cross-file options must install");
    let mut file_indexes = FxHashMap::default();
    let mut anchor = None;
    for (index, file) in files.iter().enumerate() {
        let source = compilation
            .add_source(file.path.to_string_lossy().into_owned(), file.source)
            .expect("lint source must fit Atlas identity space");
        anchor.get_or_insert(source);
        file_indexes.insert(source, index);
    }
    let anchor = anchor.expect("cross-file lint requires at least one supported source");
    let artifact = compilation
        .query::<CrossFileAnalysisProduct>(anchor)
        .unwrap_or_else(|error| panic!("Atlas cross-file analysis failed: {error}"))
        .shared();
    (artifact, file_indexes)
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

fn patina_cross_file_options() -> CrossFileOptions {
    CrossFileOptions::minimal()
        .with_provide_inject(true)
        .with_unique_ids(true)
        .with_server_client_boundary(true)
        .with_reactivity_tracking(true)
        .with_race_conditions(true)
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
