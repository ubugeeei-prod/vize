//! Lint command - Lint Vue SFC files

use clap::Args;
use glob::glob;
use ignore::Walk;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use vize_carton::{cstr, profiler::global_profiler, String, ToCompactString};
use vize_patina::{format_results, format_summary, HelpLevel, LintPreset, Linter, OutputFormat};

use crate::commands::profile::{
    print_profile_report, ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport,
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

    /// Output format (text, json)
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

    /// Lint preset: happy-path (default), opinionated, essential, nuxt
    #[arg(long, default_value = "happy-path")]
    pub preset: String,

    /// Show detailed timing profile
    #[arg(long)]
    pub profile: bool,

    /// Slow file threshold in milliseconds for profile output
    #[arg(long, default_value = "100")]
    pub slow_threshold: u64,
}

pub fn run(args: LintArgs) {
    let start = Instant::now();

    // Collect .vue files using glob patterns or directory walking
    let collect_start = Instant::now();
    let files: Vec<PathBuf> = args
        .patterns
        .iter()
        .flat_map(|pattern| {
            // Check if pattern contains glob characters
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                // Use glob for pattern matching
                glob(pattern)
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|r| r.ok())
                    .filter(|p| {
                        p.extension().is_some_and(|ext| ext == "vue")
                            && !p.components().any(|c| c.as_os_str() == "node_modules")
                    })
                    .collect::<Vec<_>>()
            } else {
                // Use directory walking for paths (respects .gitignore)
                Walk::new(pattern)
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "vue"))
                    .map(|e| e.path().to_path_buf())
                    .collect::<Vec<_>>()
            }
        })
        .collect();
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
    let linter = Linter::with_preset(preset).with_help_level(help_level);
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
    let results: Vec<_> = files
        .par_iter()
        .filter_map(|path| {
            let file_start = args.profile.then(Instant::now);
            let read_start = args.profile.then(Instant::now);
            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", path.display(), e);
                    return None;
                }
            };
            let read_time = read_start
                .map(|start| start.elapsed())
                .unwrap_or(Duration::ZERO);

            let filename = path.to_string_lossy().to_compact_string();
            let lint_file_start = args.profile.then(Instant::now);
            let result = linter.lint_sfc(&source, &filename);
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

            Some((filename, source, result))
        })
        .collect();
    let lint_time = lint_start.elapsed();
    let operation_summary = if args.profile {
        let profiler = global_profiler();
        let summary = profiler.summary();
        profiler.disable();
        Some(summary)
    } else {
        None
    };

    let total_errors = error_count.load(Ordering::Relaxed);
    let total_warnings = warning_count.load(Ordering::Relaxed);

    // Determine output format
    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Text,
    };

    // Format and print results
    let output_start = Instant::now();
    if !args.quiet || total_errors > 0 || total_warnings > 0 {
        let lint_results: Vec<_> = results.iter().map(|(_, _, r)| r).cloned().collect();
        let sources: Vec<_> = results
            .iter()
            .map(|(f, s, _)| (f.clone(), vize_carton::String::from(s.as_str())))
            .collect();

        let output = format_results(&lint_results, &sources, format);
        if !output.trim().is_empty() {
            print!("{}", output);
        }
    }
    let output_time = output_start.elapsed();

    // Print summary
    let elapsed = start.elapsed();
    if format == OutputFormat::Text {
        println!(
            "\n{}",
            format_summary(total_errors, total_warnings, files.len())
        );
        println!("Linted {} files in {:.4?}", files.len(), elapsed);
    }

    // Fix mode warning
    if args.fix {
        eprintln!("\nNote: --fix is not yet implemented");
    }

    if args.profile {
        let mut file_rows = profile_rows
            .and_then(|profile_rows| profile_rows.into_inner().ok())
            .unwrap_or_default();
        file_rows.sort_by(|left, right| right.total.cmp(&left.total));

        let total_read = file_rows
            .iter()
            .fold(Duration::ZERO, |acc, row| acc + row.primary);
        let total_lint = file_rows
            .iter()
            .fold(Duration::ZERO, |acc, row| acc + row.secondary);
        let total_bytes = file_rows.iter().fold(0usize, |acc, row| acc + row.bytes);
        let phases = [
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
            ProfilePhase {
                name: "render output",
                duration: output_time,
                kind: ProfilePhaseKind::Wall,
                note: "diagnostic formatting",
            },
        ];
        let slow_threshold = Duration::from_millis(args.slow_threshold);
        let mut recommendations: Vec<String> = Vec::new();
        if let Some(summary) = operation_summary.as_ref() {
            if let Some(entry) = summary.entries.first() {
                recommendations.push(cstr!(
                    "Deepest hot operation: {} took {:.2}ms total across {} call(s).",
                    entry.name,
                    entry.total.as_secs_f64() * 1000.0,
                    entry.count
                ));
            }
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
            phases: &phases,
            files: &file_rows,
            slow_threshold,
            throughput_bytes: Some(total_bytes),
            operations: operation_summary.as_ref(),
            recommendations: &recommendations,
        };
        print_profile_report(&report);
    }

    // Exit with appropriate code
    if total_errors > 0 {
        std::process::exit(1);
    }

    if let Some(max) = args.max_warnings {
        if total_warnings > max {
            eprintln!("\nToo many warnings ({} > max {})", total_warnings, max);
            std::process::exit(1);
        }
    }
}
