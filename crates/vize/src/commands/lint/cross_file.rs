//! Opt-in cross-file lint analysis (provide/inject, reactivity flow, race risks).

use std::path::{Path, PathBuf};
use vize_armature::Parser;
use vize_atelier_sfc::{
    SfcParseOptions,
    croquis::{SfcCroquisOptions, analyze_sfc_descriptor},
    parse_sfc,
};
use vize_carton::{Allocator, CompactString, FxHashMap, String, ToCompactString, cstr};
use vize_croquis::Croquis;
use vize_croquis_cf::{
    CrossFileAnalyzer, CrossFileDiagnostic, CrossFileDiagnosticKind, CrossFileOptions,
    DiagnosticSeverity, FileId,
};
use vize_patina::{HelpLevel, LintDiagnostic, LintResult};

pub(super) struct CrossFileLintOutput {
    pub(super) results: Vec<LintResult>,
    pub(super) provide_inject_tree: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
struct CrossFileSourceOffsets {
    script: u32,
    template: u32,
}

pub(super) fn build_cross_file_lint_output<S: AsRef<str>>(
    files: &[(PathBuf, S)],
    help_level: HelpLevel,
    include_tree: bool,
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

    CrossFileLintOutput {
        results,
        provide_inject_tree,
    }
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
        let parser = Parser::new(allocator.as_bump(), template.content.as_ref());
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
