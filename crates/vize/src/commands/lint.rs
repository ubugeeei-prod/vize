//! Lint command - Lint Vue and script files

mod aggregate;
mod args;
mod collect;
mod cross_file;
mod entry_rules;
mod fix;
mod patterns;
mod stdout;

#[cfg(test)]
mod tests;

pub use args::LintArgs;

use crate::profile_support;
use aggregate::{LintRunAccumulator, should_retain_file_results};
use collect::{LintIgnoreSet, collect_lint_files, resolve_lint_config_path};
use cross_file::apply_sfc_cross_file_lint;
use entry_rules::LinterRuleResolver;
use fix::lint_source_with_optional_fix;
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use vize_curator::profile::{
    ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report,
};
use vize_patina::{HelpLevel, LintPreset, OutputFormat, format_results};
use vize_s0::{String, ToCompactString, cstr, profile, profiler::global_profiler};

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
    let render_details = aggregate::should_render_details(format, args.quiet);
    crate::config::write_schema(None);
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (loaded_config, linter_plan, linter_features) = if args.no_config {
        (
            crate::config::LoadedConfigWithFeatures::default(),
            crate::config::LinterConfigPlanWithConfigRuleOptions::default(),
            crate::config::LinterFeatureFlags::default(),
        )
    } else {
        crate::config::load_config_and_linter_plan_with_config_rule_options_and_lint_features_and_source(
            args.config.as_deref(),
        )
    };
    let linter_enabled = linter_plan.plan.base.enabled;
    let config_dir = loaded_config
        .source_path
        .as_deref()
        .and_then(Path::parent)
        .unwrap_or(cwd.as_path());
    let config = loaded_config.config;
    if !linter_enabled {
        eprintln!("[vize] Skipping lint because linter.enabled is false in vize.config.");
        return;
    }
    let configured_corsa_path = config
        .type_checker
        .runtime_path()
        .map(|path| resolve_lint_config_path(config_dir, path));
    let ignore_set = LintIgnoreSet::new(&linter_plan.plan.global_ignores, config_dir);
    let collect_start = Instant::now();
    let files = collect_lint_files(&args.patterns, ignore_set.as_ref());
    let collect_time = collect_start.elapsed();

    if files.is_empty() {
        patterns::write_no_files(format, &args.patterns);
        return;
    }
    let help_level = match args.help_level.as_str() {
        "none" => HelpLevel::None,
        "short" => HelpLevel::Short,
        _ => HelpLevel::Full,
    };
    let preset_name: String = args
        .preset
        .as_deref()
        .or(linter_plan.plan.base.preset.as_deref())
        .unwrap_or("happy-path")
        .into();
    let preset = LintPreset::parse(preset_name.as_str()).unwrap_or_default();
    let resolved_rules =
        LinterRuleResolver::new(linter_plan, config_dir, &cwd).resolve_files(&files, &cwd);
    let linters = resolved_rules.build_linters(
        preset,
        help_level,
        &args,
        linter_features,
        configured_corsa_path,
    );
    let write_failures = AtomicUsize::new(0);
    let profile_rows = args.profile.then(|| Mutex::new(Vec::new()));
    args.profile_export.begin(args.profile);

    let lint_start = Instant::now();
    let cross_file_enabled = args.cross_file || args.cross_file_tree || args.cross_file_complexity;
    let retain_file_results = should_retain_file_results(render_details, cross_file_enabled);
    let lint_run = files
        .par_iter()
        .zip(resolved_rules.file_config_indices.par_iter())
        .filter_map(|(path, config_index)| {
            let linter = &linters[*config_index];
            let file_start = args.profile.then(Instant::now);
            let read_start = args.profile.then(Instant::now);
            let source: String = match profile!("cli.lint.file.read", fs::read_to_string(path)) {
                Ok(source) => {
                    global_profiler().record_fs_read_to_string(source.len());
                    source.into()
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
                lint_source_with_optional_fix(linter, path, source, &filename, args.fix)
            });
            let (source, result, fixed) = result
                .inspect_err(|_| {
                    write_failures.fetch_add(1, Ordering::Relaxed);
                })
                .ok()?;
            let lint_time = lint_file_start
                .map(|start| start.elapsed())
                .unwrap_or(Duration::ZERO);

            if let (Some(file_start), Some(profile_rows)) = (file_start, profile_rows.as_ref()) {
                let note = if fixed {
                    cstr!(
                        "{} error(s), {} warning(s), fixed",
                        result.error_count,
                        result.warning_count
                    )
                } else {
                    cstr!(
                        "{} error(s), {} warning(s)",
                        result.error_count,
                        result.warning_count
                    )
                };
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
        .fold(
            || LintRunAccumulator::new(retain_file_results),
            LintRunAccumulator::push,
        )
        .reduce(
            || LintRunAccumulator::new(retain_file_results),
            LintRunAccumulator::merge,
        );
    let (mut results, quiet_totals) = lint_run.into_parts();
    let lint_time = lint_start.elapsed();

    let mut cross_file_report = None;
    let cross_file_start = args.profile.then(Instant::now);
    if cross_file_enabled {
        cross_file_report = profile!(
            "cli.lint.cross_file.build",
            apply_sfc_cross_file_lint(
                &mut results,
                help_level,
                args.cross_file_tree,
                args.cross_file_complexity
            )
        );
    }
    let cross_file_time = cross_file_start
        .map(|start| start.elapsed())
        .unwrap_or(Duration::ZERO);

    let (lint_error_count, total_warnings) = aggregate::sorted_totals(quiet_totals, &mut results);
    let total_errors = lint_error_count + write_failures.load(Ordering::Relaxed);

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
                .map(|(_, f, s, _)| (f.clone(), vize_s0::String::from(s.as_str())))
                .collect()
        );

        let output = profile!(
            "cli.lint.output.format_results",
            format_results(&lint_results, &sources, format)
        );
        if !output.trim().is_empty() {
            stdout::write(output.as_bytes());
        }
    }
    let output_time = output_start.elapsed();
    args.profile_export.finish("lint", args.profile);
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

    let elapsed = start.elapsed();
    if format == OutputFormat::Text {
        stdout::write_text_summary(
            total_errors,
            total_warnings,
            files.len(),
            elapsed,
            cross_file_report.as_deref(),
        );
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
