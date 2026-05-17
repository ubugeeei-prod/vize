//! Lint command - Lint Vue SFC files

use clap::Args;
use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::time::Instant;
use vize_armature::Parser;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{
    Allocator, CompactString, FxHashMap, FxHashSet, String, ToCompactString, cstr, profile,
    profiler::{allocation_snapshot, global_profiler},
};
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};
use vize_croquis_cf::{
    CrossFileAnalyzer, CrossFileDiagnostic, CrossFileDiagnosticKind, CrossFileOptions,
    DiagnosticSeverity, FileId,
};
use vize_patina::{
    HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter, OutputFormat, format_results,
    format_summary,
};

use crate::commands::profile::{
    ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report,
};

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct LintArgs {
    /// Glob pattern(s) to match .vue files
    #[arg(default_value = "./**/*.vue")]
    pub patterns: Vec<String>,

    /// Automatically fix problems (not yet implemented)
    #[arg(long)]
    pub fix: bool,

    /// Config file path (not yet implemented)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Do not load a config file
    #[arg(long)]
    pub no_config: bool,

    /// Output format (text, ansi, plain, json, stylish, markdown, html, agent)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Maximum number of warnings before failing
    #[arg(long)]
    pub max_warnings: Option<usize>,

    /// Quiet mode - only show summary
    #[arg(short, long)]
    pub quiet: bool,

    /// Help display level: full (default), short, none
    #[arg(long, default_value = "full")]
    pub help_level: String,

    /// Lint preset: happy-path (default), opinionated, essential, incremental, nuxt
    #[arg(long, default_value = "happy-path")]
    pub preset: String,

    /// Enable opt-in cross-file lint checks for provide/inject, reactivity flow, and race risks.
    #[arg(long)]
    pub cross_file: bool,

    /// Print the provide/inject tree when cross-file lint is enabled.
    #[arg(long)]
    pub cross_file_tree: bool,

    /// Enable opt-in type-aware reactivity-loss linting through the native checker.
    #[arg(long)]
    pub strict_reactivity: bool,

    /// Show detailed timing profile
    #[arg(long)]
    pub profile: bool,

    /// Slow file threshold in milliseconds for profile output
    #[arg(long, default_value = "100")]
    pub slow_threshold: u64,
}

pub fn run(args: LintArgs) {
    let start = Instant::now();
    let format = OutputFormat::parse(args.format.as_str()).unwrap_or_else(|| {
        eprintln!(
            "Unknown lint output format '{}'. Expected one of: text, ansi, plain, json, stylish, markdown, html, agent",
            args.format
        );
        std::process::exit(2);
    });
    let render_details = should_render_lint_details(format, args.quiet);

    // Collect .vue files using glob patterns or directory walking
    let collect_start = Instant::now();
    let files = collect_lint_files(&args.patterns);
    let collect_time = collect_start.elapsed();

    if files.is_empty() {
        eprintln!("No .vue files found matching patterns: {:?}", args.patterns);
        return;
    }

    let help_level = match args.help_level.as_str() {
        "none" => HelpLevel::None,
        "short" => HelpLevel::Short,
        _ => HelpLevel::Full,
    };
    let preset = LintPreset::parse(&args.preset).unwrap_or_default();
    let mut linter = Linter::with_preset(preset).with_help_level(help_level);
    #[cfg(not(target_arch = "wasm32"))]
    if args.strict_reactivity {
        linter = linter.with_rule(Box::new(
            vize_patina::rules::type_aware::NoReactivityLoss::new(),
        ));
    }
    let error_count = AtomicUsize::new(0);
    let warning_count = AtomicUsize::new(0);
    let profile_rows = args.profile.then(|| Mutex::new(Vec::new()));
    if args.profile {
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
    }

    // Lint all files in parallel and collect results
    let lint_start = Instant::now();
    let mut results: Vec<_> = files
        .par_iter()
        .filter_map(|path| {
            let file_start = args.profile.then(Instant::now);
            let read_start = args.profile.then(Instant::now);
            let source = match profile!("cli.lint.file.read", fs::read_to_string(path)) {
                Ok(s) => {
                    global_profiler().record_fs_read_to_string(s.len());
                    s
                }
                Err(e) => {
                    global_profiler().record_fs_read_to_string_failure();
                    eprintln!("Failed to read {}: {}", path.display(), e);
                    return None;
                }
            };
            let read_time = read_start
                .map(|start| start.elapsed())
                .unwrap_or(Duration::ZERO);

            let filename = path.to_string_lossy().to_compact_string();
            let lint_file_start = args.profile.then(Instant::now);
            let result = profile!(
                "cli.lint.file.lint_sfc",
                linter.lint_sfc(&source, &filename)
            );
            let lint_time = lint_file_start
                .map(|start| start.elapsed())
                .unwrap_or(Duration::ZERO);

            error_count.fetch_add(result.error_count, Ordering::Relaxed);
            warning_count.fetch_add(result.warning_count, Ordering::Relaxed);

            if let (Some(file_start), Some(profile_rows)) = (file_start, profile_rows.as_ref()) {
                let note = cstr!(
                    "{} error(s), {} warning(s)",
                    result.error_count,
                    result.warning_count
                );
                if let Ok(mut rows) = profile_rows.lock() {
                    rows.push(ProfileFileRow {
                        path: path.clone(),
                        bytes: source.len(),
                        total: file_start.elapsed(),
                        primary_label: "read",
                        primary: read_time,
                        secondary_label: "lint",
                        secondary: lint_time,
                        note: Some(note),
                    });
                }
            }

            Some((path.clone(), filename, source, result))
        })
        .collect();
    let lint_time = lint_start.elapsed();

    let mut cross_file_tree = None;
    let cross_file_enabled = args.cross_file || args.cross_file_tree;
    let cross_file_start = args.profile.then(Instant::now);
    if cross_file_enabled {
        let cross_file_inputs: Vec<_> = results
            .iter()
            .map(|(path, _, source, _)| (path.clone(), source.as_str()))
            .collect();
        let cross_file_output = profile!(
            "cli.lint.cross_file.build",
            build_cross_file_lint_output(&cross_file_inputs, help_level, args.cross_file_tree)
        );
        cross_file_tree = cross_file_output.provide_inject_tree;

        profile!("cli.lint.cross_file.merge", {
            for (index, cross_result) in cross_file_output.results.into_iter().enumerate() {
                if let Some((_, _, _, result)) = results.get_mut(index) {
                    merge_lint_result(result, cross_result);
                }
            }
        });
    }
    let cross_file_time = cross_file_start
        .map(|start| start.elapsed())
        .unwrap_or(Duration::ZERO);

    let total_errors: usize = results
        .iter()
        .map(|(_, _, _, result)| result.error_count)
        .sum();
    let total_warnings: usize = results
        .iter()
        .map(|(_, _, _, result)| result.warning_count)
        .sum();

    // Format and print results
    let output_start = Instant::now();
    if render_details {
        let lint_results: Vec<_> = profile!(
            "cli.lint.output.clone_results",
            results.iter().map(|(_, _, _, r)| r).cloned().collect()
        );
        let sources: Vec<_> = profile!(
            "cli.lint.output.clone_sources",
            results
                .iter()
                .map(|(_, f, s, _)| (f.clone(), vize_carton::String::from(s.as_str())))
                .collect()
        );

        let output = profile!(
            "cli.lint.output.format_results",
            format_results(&lint_results, &sources, format)
        );
        if !output.trim().is_empty() {
            print!("{}", output);
        }
    }
    let output_time = output_start.elapsed();
    let (operation_summary, counter_summary, allocation_summary) = if args.profile {
        let profiler = global_profiler();
        let allocation = allocation_snapshot();
        let counters = profiler.counter_summary();
        let operations = profiler.summary();
        profiler.disable();
        (Some(operations), Some(counters), Some(allocation))
    } else {
        (None, None, None)
    };

    // Print summary
    let elapsed = start.elapsed();
    if format == OutputFormat::Text {
        println!(
            "\n{}",
            format_summary(total_errors, total_warnings, files.len())
        );
        println!("Linted {} files in {:.4?}", files.len(), elapsed);
        if args.cross_file_tree
            && let Some(tree) = cross_file_tree.as_deref()
        {
            println!("\n{tree}");
        }
    }

    // Fix mode warning
    if args.fix {
        eprintln!("\nNote: --fix is not yet implemented");
    }

    if args.profile {
        let mut file_rows = profile_rows
            .and_then(|profile_rows| profile_rows.into_inner().ok())
            .unwrap_or_default();
        file_rows.sort_by_key(|row| std::cmp::Reverse(row.total));

        let total_read = file_rows
            .iter()
            .fold(Duration::ZERO, |acc, row| acc + row.primary);
        let total_lint = file_rows
            .iter()
            .fold(Duration::ZERO, |acc, row| acc + row.secondary);
        let total_bytes = file_rows.iter().fold(0usize, |acc, row| acc + row.bytes);
        let mut phases = vec![
            ProfilePhase {
                name: "collect files",
                duration: collect_time,
                kind: ProfilePhaseKind::Wall,
                note: "glob and ignore-aware walk",
            },
            ProfilePhase {
                name: "lint wall",
                duration: lint_time,
                kind: ProfilePhaseKind::Wall,
                note: "parallel worker elapsed time",
            },
            ProfilePhase {
                name: "read total",
                duration: total_read,
                kind: ProfilePhaseKind::Cumulative,
                note: "sum across worker threads",
            },
            ProfilePhase {
                name: "lint total",
                duration: total_lint,
                kind: ProfilePhaseKind::Cumulative,
                note: "sum across worker threads",
            },
        ];
        if cross_file_enabled {
            phases.push(ProfilePhase {
                name: "cross-file lint",
                duration: cross_file_time,
                kind: ProfilePhaseKind::Wall,
                note: "project graph diagnostics",
            });
        }
        phases.push(ProfilePhase {
            name: "render output",
            duration: output_time,
            kind: ProfilePhaseKind::Wall,
            note: "diagnostic formatting",
        });
        let slow_threshold = Duration::from_millis(args.slow_threshold);
        let mut recommendations: Vec<String> = Vec::new();
        if let Some(summary) = operation_summary.as_ref()
            && let Some(entry) = summary.entries.first()
        {
            recommendations.push(cstr!(
                "Deepest hot operation: {} took {:.2}ms total across {} call(s).",
                entry.name,
                entry.total.as_secs_f64() * 1000.0,
                entry.count
            ));
        }
        for row in file_rows
            .iter()
            .filter(|row| row.total > slow_threshold)
            .take(4)
        {
            recommendations.push(cstr!(
                "{} exceeded the slow threshold; start with the lint rule preset and script/template size.",
                row.path.display()
            ));
        }
        if output_time > lint_time {
            recommendations.push(
                "Output rendering is heavier than linting; use --quiet during profiling runs that only need totals."
                    .into(),
            );
        }

        let summary = cstr!(
            "{} file(s), {} error(s), {} warning(s), preset '{}'",
            files.len(),
            total_errors,
            total_warnings,
            args.preset
        );
        let report = ProfileReport {
            title: "lint",
            summary: summary.as_str(),
            total: elapsed,
            phases: phases.as_slice(),
            files: &file_rows,
            slow_threshold,
            throughput_bytes: Some(total_bytes),
            operations: operation_summary.as_ref(),
            counters: counter_summary.as_ref(),
            allocations: allocation_summary,
            recommendations: &recommendations,
        };
        print_profile_report(&report);
    }

    // Exit with appropriate code
    if total_errors > 0 {
        std::process::exit(1);
    }

    if let Some(max) = args.max_warnings
        && total_warnings > max
    {
        eprintln!("\nToo many warnings ({} > max {})", total_warnings, max);
        std::process::exit(1);
    }
}

fn collect_lint_files(patterns: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();

    for pattern in patterns {
        let candidate = PathBuf::from(pattern);
        if candidate.exists() {
            if candidate.is_file() {
                add_lint_file(&candidate, &mut files, &mut seen);
                continue;
            }
            if candidate.is_dir() {
                collect_lint_files_from_dir(&candidate, None, &mut files, &mut seen);
                continue;
            }
        }

        let base_dir = base_dir_from_lint_pattern(pattern);
        let matcher = LintInputGlob::new(pattern);
        collect_lint_files_from_dir(&base_dir, matcher.as_ref(), &mut files, &mut seen);
    }

    files.sort();
    files
}

fn collect_lint_files_from_dir(
    dir: &Path,
    matcher: Option<&LintInputGlob>,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) {
    for entry in WalkBuilder::new(dir)
        .standard_filters(true)
        .hidden(true)
        .build()
    {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_file() && matcher.is_none_or(|matcher| matcher.matches(path)) {
            add_lint_file(path, files, seen);
        }
    }
}

fn add_lint_file(path: &Path, files: &mut Vec<PathBuf>, seen: &mut FxHashSet<PathBuf>) {
    if path.extension().and_then(|extension| extension.to_str()) != Some("vue") {
        return;
    }
    let normalized = normalize_lint_input_path(path);
    let canonical = path.canonicalize().unwrap_or_else(|_| normalized.clone());
    if seen.insert(canonical) {
        files.push(normalized);
    }
}

fn base_dir_from_lint_pattern(pattern: &str) -> PathBuf {
    let glob_start = pattern.find(['*', '?', '[', '{']).unwrap_or(pattern.len());
    let prefix = &pattern[..glob_start];
    let base = if prefix.is_empty() {
        "."
    } else if let Some(index) = prefix.rfind('/') {
        &prefix[..index]
    } else {
        prefix
    };
    if base.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(base)
    }
}

struct LintInputGlob {
    pattern: Pattern,
    cwd: PathBuf,
    absolute: bool,
}

impl LintInputGlob {
    fn new(pattern: &str) -> Option<Self> {
        let normalized = normalize_lint_glob_pattern(pattern);
        let absolute = Path::new(normalized.as_str()).is_absolute();
        Pattern::new(normalized.as_str()).ok().map(|pattern| Self {
            pattern,
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            absolute,
        })
    }

    fn matches(&self, path: &Path) -> bool {
        let candidate = if self.absolute {
            let absolute = if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.cwd.join(path)
            };
            normalize_lint_path(&absolute)
        } else {
            normalize_lint_path(path)
        };

        self.pattern
            .matches_with(&candidate, lint_glob_match_options())
    }
}

fn normalize_lint_glob_pattern(pattern: &str) -> String {
    strip_lint_current_dir_prefix(&pattern.replace('\\', "/"))
}

fn normalize_lint_path(path: &Path) -> String {
    strip_lint_current_dir_prefix(&path.to_string_lossy().replace('\\', "/"))
}

fn strip_lint_current_dir_prefix(value: &str) -> String {
    let mut normalized = value;
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped;
    }
    normalized.into()
}

fn normalize_lint_input_path(path: &Path) -> PathBuf {
    PathBuf::from(normalize_lint_path(path))
}

fn lint_glob_match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
}

struct CrossFileLintOutput {
    results: Vec<LintResult>,
    provide_inject_tree: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
struct CrossFileSourceOffsets {
    script: u32,
    template: u32,
}

fn build_cross_file_lint_output<S: AsRef<str>>(
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

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    let mut offsets = CrossFileSourceOffsets::default();

    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        offsets.script = script_setup.loc.start as u32;
        let generic = script_setup
            .attrs
            .get("generic")
            .map(|value| value.as_ref());
        analyzer.analyze_script_setup_with_generic(script_setup.content.as_ref(), generic);
    } else if let Some(script) = descriptor.script.as_ref() {
        offsets.script = script.loc.start as u32;
        analyzer.analyze_script_plain(script.content.as_ref());
    }

    if let Some(template) = descriptor.template.as_ref() {
        offsets.template = template.loc.start as u32;
        let allocator = Allocator::with_capacity((template.content.len() * 4).max(64 * 1024));
        let parser = Parser::new(allocator.as_bump(), template.content.as_ref());
        let (root, _parse_errors) = parser.parse();
        analyzer.analyze_template(&root);
    }

    Some((analyzer.finish(), offsets))
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
        | CrossFileDiagnosticKind::NonUniqueIdInLoop { .. } => offsets.template,
        _ => offsets.script,
    }
}

fn merge_lint_result(target: &mut LintResult, mut extra: LintResult) {
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

#[inline]
fn should_render_lint_details(format: OutputFormat, quiet: bool) -> bool {
    format.renders_details_when_quiet() || !quiet
}

#[cfg(test)]
mod tests {
    use super::{build_cross_file_lint_output, should_render_lint_details};
    use std::fs;
    use vize_patina::{LintPreset, Linter, OutputFormat};

    #[test]
    fn quiet_text_output_skips_detailed_diagnostics() {
        assert!(!should_render_lint_details(OutputFormat::Text, true));
    }

    #[test]
    fn json_output_remains_machine_readable_in_quiet_mode() {
        assert!(should_render_lint_details(OutputFormat::Json, true));
    }

    #[test]
    fn report_formats_render_in_quiet_mode() {
        assert!(should_render_lint_details(OutputFormat::Ansi, true));
        assert!(should_render_lint_details(OutputFormat::Plain, true));
        assert!(should_render_lint_details(OutputFormat::Markdown, true));
        assert!(should_render_lint_details(OutputFormat::Html, true));
        assert!(should_render_lint_details(OutputFormat::Agent, true));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn strict_reactivity_can_be_enabled_without_opinionated_preset() {
        let linter = Linter::with_preset(LintPreset::HappyPath).with_rule(Box::new(
            vize_patina::rules::type_aware::NoReactivityLoss::new(),
        ));

        assert!(linter.registry().has_rule("type/no-reactivity-loss"));
    }

    #[test]
    fn cross_file_opt_in_reports_reactivity_and_tree() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("App.vue");
        let middle = dir.path().join("Middle.vue");
        let child = dir.path().join("Child.vue");

        fs::write(
            &app,
            r#"<script setup lang="ts">
import { provide, reactive } from 'vue'
import Middle from './Middle.vue'

const state = reactive({ count: 0 })
provide('state', state)
</script>

<template>
  <Middle />
</template>
"#,
        )
        .unwrap();
        fs::write(
            &middle,
            r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child />
</template>
"#,
        )
        .unwrap();
        fs::write(
            &child,
            r#"<script setup lang="ts">
import { inject } from 'vue'

const { count } = inject('state') as { count: number }
</script>
"#,
        )
        .unwrap();

        let files = [&app, &middle, &child]
            .into_iter()
            .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
            .collect::<Vec<_>>();
        let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, true);

        let child_result = output
            .results
            .iter()
            .find(|result| result.filename.ends_with("Child.vue"))
            .expect("child result should exist");
        assert!(child_result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("destructuring-breaks-reactivity")
        }));

        let tree = output
            .provide_inject_tree
            .as_deref()
            .expect("tree should be rendered");
        assert!(tree.contains("App"));
        assert!(tree.contains("Middle"));
        assert!(tree.contains("Child"));
    }

    #[test]
    fn cross_file_opt_in_reports_duplicate_element_ids_at_template_offsets() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("First.vue");
        let second = dir.path().join("Second.vue");

        let first_source = r#"<script setup lang="ts">
const ready = true
</script>

<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
"#;
        fs::write(&first, first_source).unwrap();
        fs::write(
            &second,
            r#"<template>
  <input id="email" />
</template>
"#,
        )
        .unwrap();

        let files = [&first, &second]
            .into_iter()
            .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
            .collect::<Vec<_>>();
        let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

        let first_result = output
            .results
            .iter()
            .find(|result| result.filename.ends_with("First.vue"))
            .expect("first result should exist");
        let diagnostic = first_result
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("duplicate-id"))
            .expect("duplicate element id should be reported");

        let expected_start = first_source.find("id=\"email\"").unwrap() as u32;
        assert_eq!(diagnostic.start, expected_start);
        assert!(diagnostic.end > diagnostic.start);
    }

    #[test]
    fn cross_file_opt_in_reports_reactive_prop_destructure() {
        let dir = tempfile::tempdir().unwrap();
        let parent = dir.path().join("Parent.vue");
        let child = dir.path().join("Child.vue");

        fs::write(
            &parent,
            r#"<script setup lang="ts">
import { reactive } from 'vue'
import Child from './Child.vue'

const state = reactive({ count: 0 })
</script>

<template>
  <Child :item="state" />
</template>
"#,
        )
        .unwrap();
        fs::write(
            &child,
            r#"<script setup lang="ts">
const props = defineProps<{ item: { count: number } }>()
const { item } = props
</script>
"#,
        )
        .unwrap();

        let files = [&parent, &child]
            .into_iter()
            .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
            .collect::<Vec<_>>();
        let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

        let child_result = output
            .results
            .iter()
            .find(|result| result.filename.ends_with("Child.vue"))
            .expect("child result should exist");
        assert!(child_result.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == vize_patina::Severity::Error
                && diagnostic
                    .message
                    .contains("destructuring-breaks-reactivity")
        }));
    }

    #[test]
    fn cross_file_opt_in_allows_direct_define_props_destructure_until_aliased() {
        let dir = tempfile::tempdir().unwrap();
        let direct = dir.path().join("Direct.vue");
        let alias = dir.path().join("Alias.vue");

        fs::write(
            &direct,
            r#"<script setup lang="ts">
const { item } = defineProps<{ item: { count: number } }>()
</script>
"#,
        )
        .unwrap();
        fs::write(
            &alias,
            r#"<script setup lang="ts">
const { item } = defineProps<{ item: { count: number } }>()
const item2 = item
</script>
"#,
        )
        .unwrap();

        let files = [&direct, &alias]
            .into_iter()
            .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
            .collect::<Vec<_>>();
        let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

        let direct_result = output
            .results
            .iter()
            .find(|result| result.filename.ends_with("Direct.vue"))
            .expect("direct result should exist");
        assert!(!direct_result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("destructuring-breaks-reactivity")
                || diagnostic
                    .message
                    .contains("value-extraction-breaks-reactivity")
        }));

        let alias_result = output
            .results
            .iter()
            .find(|result| result.filename.ends_with("Alias.vue"))
            .expect("alias result should exist");
        assert!(alias_result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("value-extraction-breaks-reactivity")
        }));
    }

    #[test]
    fn cross_file_opt_in_reports_loop_element_ids_at_template_offsets() {
        let dir = tempfile::tempdir().unwrap();
        let list = dir.path().join("List.vue");

        let source = r#"<script setup lang="ts">
const rows = [{ name: 'Ada' }]
</script>

<template>
  <ul>
    <li v-for="row in rows">
      <span id="row-label">{{ row.name }}</span>
    </li>
  </ul>
</template>
"#;
        fs::write(&list, source).unwrap();

        let files = [list]
            .into_iter()
            .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
            .collect::<Vec<_>>();
        let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

        let list_result = output
            .results
            .iter()
            .find(|result| result.filename.ends_with("List.vue"))
            .expect("list result should exist");
        let diagnostic = list_result
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("non-unique-id"))
            .expect("static id in v-for should be reported");

        let expected_start = source.find("id=\"row-label\"").unwrap() as u32;
        assert_eq!(diagnostic.start, expected_start);
        assert!(diagnostic.end > diagnostic.start);
    }

    #[test]
    fn cross_file_opt_in_reports_async_injected_state_race() {
        let dir = tempfile::tempdir().unwrap();
        let provider = dir.path().join("Provider.vue");
        let child = dir.path().join("Child.vue");

        fs::write(
            &provider,
            r#"<script setup lang="ts">
import { provide, reactive } from 'vue'
import Child from './Child.vue'

const store = reactive({ count: 0 })
provide('store', store)
</script>

<template>
  <Child />
</template>
"#,
        )
        .unwrap();
        fs::write(
            &child,
            r#"<script setup lang="ts">
import { inject, ref, watch } from 'vue'

const store = inject('store')!
const query = ref('')

watch(query, async () => {
  await load()
  store.count = 1
})
</script>
"#,
        )
        .unwrap();

        let files = [&provider, &child]
            .into_iter()
            .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
            .collect::<Vec<_>>();
        let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

        let child_result = output
            .results
            .iter()
            .find(|result| result.filename.ends_with("Child.vue"))
            .expect("child result should exist");
        assert!(child_result.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == vize_patina::Severity::Error
                && diagnostic.message.contains("injected-async-mutation-race")
        }));
    }
}
