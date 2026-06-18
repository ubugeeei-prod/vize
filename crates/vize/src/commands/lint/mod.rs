//! Lint command - Lint Vue SFC files

mod args;
mod collect;
mod cross_file;

#[cfg(test)]
mod tests;

pub use args::LintArgs;

use crate::profile_support;
use collect::{collect_lint_files, is_standalone_html_path, resolve_lint_config_path};
use cross_file::{build_cross_file_lint_output, merge_lint_result};
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::time::Instant;
use vize_carton::{String, ToCompactString, cstr, profile, profiler::global_profiler};
use vize_curator::profile::{
    ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report,
};
use vize_patina::{
    HelpLevel, JsxLang, LintPreset, Linter, OutputFormat, format_results, format_summary,
};

pub fn run(args: LintArgs) {
    let start = Instant::now();
    if let Some(path) = args.config.as_deref()
        && !args.no_config
        && let Err(error) = crate::config::validate_explicit_config_path(path)
    {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(2);
    }
    let format = OutputFormat::parse(args.format.as_str()).unwrap_or_else(|| {
        eprintln!(
            "Unknown lint output format '{}'. Expected one of: text, ansi, plain, json, stylish, markdown, html, agent",
            args.format
        );
        std::process::exit(2);
    });
    let render_details = should_render_lint_details(format, args.quiet);
    crate::config::write_schema(None);
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // Load the shared config and linter section from the same raw parse. The
    // lint command is often used in watch/CI loops, so avoiding a second config
    // evaluation keeps startup overhead out of the per-run profile.
    let (loaded_config, linter_config) = if args.no_config {
        (
            crate::config::LoadedConfig {
                config: crate::config::VizeConfig::default(),
                source_path: None,
            },
            crate::config::LinterConfig::default(),
        )
    } else {
        crate::config::load_config_and_linter_with_source(args.config.as_deref())
    };
    let config_dir = loaded_config
        .source_path
        .as_deref()
        .and_then(Path::parent)
        .unwrap_or(cwd.as_path());
    let config = loaded_config.config;
    if !linter_config.enabled {
        eprintln!("[vize] Skipping lint because linter.enabled is false in vize.config.");
        return;
    }
    let configured_corsa_path = config
        .type_checker
        .runtime_path()
        .map(|path| resolve_lint_config_path(config_dir, path));
    // Collect .vue, standalone .html, and JSX/TSX files using glob patterns or directory walking
    let collect_start = Instant::now();
    let files = collect_lint_files(&args.patterns);
    let collect_time = collect_start.elapsed();

    if files.is_empty() {
        eprintln!(
            "No .vue, .html, .jsx, or .tsx files found matching patterns: {:?}",
            args.patterns
        );
        return;
    }

    let help_level = match args.help_level.as_str() {
        "none" => HelpLevel::None,
        "short" => HelpLevel::Short,
        _ => HelpLevel::Full,
    };
    let preset_name = args
        .preset
        .as_deref()
        .or(linter_config.preset.as_deref())
        .unwrap_or("ecosystem");
    let preset = LintPreset::parse(preset_name).unwrap_or_default();
    let type_aware_enabled =
        args.type_aware || args.strict_reactivity || linter_config.type_aware_lint_enabled();
    let mut linter = Linter::with_preset(preset)
        .with_additional_rules(linter_config.enabled_rules())
        .with_disabled_rules(linter_config.disabled_rules())
        .with_help_level(help_level)
        .with_type_aware_lint(type_aware_enabled);
    #[cfg(not(target_arch = "wasm32"))]
    {
        linter = linter.with_corsa_path(configured_corsa_path);
    }
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
            let result = profile!("cli.lint.file.lint", {
                if is_standalone_html_path(path) {
                    linter.lint_standalone_html(&source, &filename)
                } else if let Some(lang) = jsx_lang_for_path(path) {
                    linter.lint_jsx(&source, &filename, lang)
                } else {
                    linter.lint_sfc(&source, &filename)
                }
            });
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
        let allocation = profile_support::allocation_snapshot();
        let counters = profiler.counter_summary();
        let operations = profiler.summary();
        profiler.disable();
        (Some(operations), Some(counters), allocation)
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
            preset_name
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

    // `process::exit` below bypasses normal stdout teardown, so flush report output first.
    let _ = std::io::stdout().flush();

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

fn jsx_lang_for_path(path: &Path) -> Option<JsxLang> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("jsx") => Some(JsxLang::Jsx),
        Some("tsx") => Some(JsxLang::Tsx),
        _ => None,
    }
}

#[inline]
fn should_render_lint_details(format: OutputFormat, quiet: bool) -> bool {
    format.renders_details_when_quiet() || !quiet
}
