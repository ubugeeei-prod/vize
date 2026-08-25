//! Opt-in cross-file lint analysis (provide/inject, reactivity flow, race risks).

use std::path::{Path, PathBuf};
use vize_armature::Parser;
use vize_atelier_sfc::{
    SfcParseOptions,
    croquis::{SfcCroquisOptions, analyze_sfc_descriptor},
    parse_sfc,
};
use vize_croquis::Croquis;
use vize_croquis_cf::{
    CrossFileAnalyzer, CrossFileDiagnostic, CrossFileDiagnosticKind, CrossFileOptions,
    DiagnosticSeverity, FileId,
};
use vize_curator::complexity::render_complexity_markdown;
use vize_patina::{HelpLevel, LintDiagnostic, LintResult};
use vize_s0::{Allocator, CompactString, FxHashMap, String, ToCompactString, cstr};

pub(super) struct CrossFileLintOutput {
    pub(super) results: Vec<LintResult>,
    pub(super) provide_inject_tree: Option<String>,
    pub(super) complexity_report: Option<String>,
}

pub(super) type CliLintFileResult = (PathBuf, String, String, LintResult);

#[derive(Clone, Copy, Debug, Default)]
struct CrossFileSourceOffsets {
    script: u32,
    template: u32,
}

pub(super) fn apply_sfc_cross_file_lint(
    results: &mut [CliLintFileResult],
    help_level: HelpLevel,
    include_tree: bool,
    include_complexity: bool,
) -> Option<String> {
    let targets: Vec<_> = results
        .iter()
        .enumerate()
        .filter(|(_, (path, _, _, _))| is_sfc_cross_file_target(path))
        .map(|(index, _)| index)
        .collect();
    let inputs: Vec<_> = targets
        .iter()
        .map(|index| {
            let (path, _, source, _) = &results[*index];
            (path.clone(), source.clone())
        })
        .collect();
    let output = build_cross_file_lint_output_with_report(
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
        if let Some((_, _, _, result)) = results.get_mut(target_index) {
            merge_lint_result(result, cross_result);
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

pub(super) fn build_cross_file_lint_output_with_report<S: AsRef<str>>(
    files: &[(PathBuf, S)],
    help_level: HelpLevel,
    include_tree: bool,
    include_complexity: bool,
) -> CrossFileLintOutput {
    let root = std::env::current_dir().unwrap_or_default();
    let mut analyzer = CrossFileAnalyzer::with_project_root(patina_cross_file_options(), root);
    let mut file_indexes: FxHashMap<FileId, usize> = FxHashMap::default();
    let mut source_offsets: FxHashMap<FileId, CrossFileSourceOffsets> = FxHashMap::default();
    let mut results: Vec<_> = files
        .iter()
        .map(|(path, _)| LintResult {
            filename: path.to_string_lossy().to_compact_string(),
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        })
        .collect();

    for (index, (path, source)) in files.iter().enumerate() {
        let source = source.as_ref();
        let Some((analysis, offsets)) = analyze_sfc_for_cross_file(source, path) else {
            continue;
        };
        let file_id = analyzer.add_file_with_analysis(path, source, analysis);
        file_indexes.insert(file_id, index);
        source_offsets.insert(file_id, offsets);
    }

    analyzer.rebuild_import_edges();
    analyzer.rebuild_component_edges();
    let cross_file_result = analyzer.analyze();

    for diagnostic in &cross_file_result.diagnostics {
        let Some(index) = file_indexes.get(&diagnostic.primary_file).copied() else {
            continue;
        };
        let offsets = source_offsets
            .get(&diagnostic.primary_file)
            .copied()
            .unwrap_or_default();
        let source_len = files[index].1.as_ref().len();
        results[index]
            .diagnostics
            .push(cross_file_diagnostic_to_lint(
                diagnostic, offsets, source_len, help_level,
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
        .then(|| {
            cross_file_result
                .provide_inject_tree
                .as_ref()
                .map(|tree| tree.to_markdown(analyzer.registry()))
        })
        .flatten();
    let complexity_report = include_complexity.then(|| {
        render_complexity_markdown(
            &cross_file_result.complexity_report,
            &cross_file_result.complexity_hotspots,
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

fn patina_cross_file_options() -> CrossFileOptions {
    CrossFileOptions::minimal()
        .with_provide_inject(true)
        .with_unique_ids(true)
        .with_server_client_boundary(true)
        .with_reactivity_tracking(true)
        .with_race_conditions(true)
}

fn analyze_sfc_for_cross_file(
    source: &str,
    path: &Path,
) -> Option<(Croquis, CrossFileSourceOffsets)> {
    let filename = path.to_string_lossy();
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.as_ref().into(),
            ..Default::default()
        },
    )
    .ok()?;

    let mut offsets = CrossFileSourceOffsets::default();

    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        offsets.script = if descriptor.script.is_some() {
            descriptor
                .script
                .as_ref()
                .map(|script| script.loc.start as u32)
                .unwrap_or(script_setup.loc.start as u32)
        } else {
            script_setup.loc.start as u32
        };
    } else if let Some(script) = descriptor.script.as_ref() {
        offsets.script = script.loc.start as u32;
    }

    let analysis = if let Some(template) = descriptor.template.as_ref() {
        offsets.template = template.loc.start as u32;
        let allocator = Allocator::with_capacity((template.content.len() * 4).max(64 * 1024));
        let parser = Parser::new(&allocator, template.content.as_ref());
        let (root, parse_errors) = parser.parse();
        let template_ast = if parse_errors.iter().any(|error| !error.is_recoverable()) {
            None
        } else {
            Some(&root)
        };
        analyze_sfc_descriptor(&descriptor, template_ast, SfcCroquisOptions::full())
    } else {
        analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::full())
    };

    Some((analysis, offsets))
}

fn is_sfc_cross_file_target(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("vue")
    )
}

fn cross_file_diagnostic_to_lint(
    diagnostic: &CrossFileDiagnostic,
    offsets: CrossFileSourceOffsets,
    source_len: usize,
    help_level: HelpLevel,
) -> LintDiagnostic {
    let source_len = source_len as u32;
    let offset = cross_file_diagnostic_offset(diagnostic, offsets);
    let start = (diagnostic.primary_offset + offset).min(source_len);
    let raw_end = diagnostic.primary_end_offset + offset;
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

fn cross_file_diagnostic_offset(
    diagnostic: &CrossFileDiagnostic,
    offsets: CrossFileSourceOffsets,
) -> u32 {
    match diagnostic.kind {
        CrossFileDiagnosticKind::DuplicateElementId { .. }
        | CrossFileDiagnosticKind::NonUniqueIdInLoop { .. }
        | CrossFileDiagnosticKind::BrowserApiInSsr { .. } => offsets.template,
        _ => offsets.script,
    }
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
