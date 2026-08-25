//! Build command execution logic.
//!
//! Contains the main compilation pipeline, file collection, pattern matching,
//! and per-file compilation with profiling.

mod cache;
mod collect;
mod compile;
mod compile_stats;
mod declarations;
mod fallback;
mod output;
mod profile_facts;
mod settings;

use std::{
    sync::{Mutex, atomic::Ordering},
    time::{Duration, Instant},
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use vize_s0::{String, cstr, profiler::global_profiler};

use crate::profile_support;
use vize_curator::profile::{
    ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report,
};

use super::{
    BuildArgs, OutputFormat,
    config::{CompileError, CompileStats, FileProfile},
};

use cache::StatsCompileCache;
use collect::{CollectedFiles, collect_files_or_exit};
use compile::compile_file_with_profile;
use compile_stats::compile_file_stats_with_cache;
use output::{CompiledBuildOutput, plan_inputs, preflight_outputs, write_outputs};
use settings::{CompileFileSettings, load_build_config};

/// Main entry point for the build command.
pub(crate) fn run(args: BuildArgs) {
    let start = Instant::now();
    let slow_threshold = Duration::from_millis(args.slow_threshold);
    if let Some(path) = args.config.as_deref()
        && !args.no_config
        && let Err(error) = crate::config::validate_explicit_config_path(path)
    {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(1);
    }
    let build_config = load_build_config(args.no_config, args.config.as_deref());
    if build_config
        .dialect
        .is_some_and(|dialect| dialect.is_legacy())
        && build_config.host_compiler == Some(false)
    {
        eprintln!(
            "\x1b[31mError:\x1b[0m compiler.compatibility.hostCompiler=false is unsupported for Vue 2 compatibility mode"
        );
        std::process::exit(1);
    }
    if args.declaration && matches!(args.format, OutputFormat::Stats) {
        eprintln!("\x1b[31mError:\x1b[0m --declaration cannot be combined with --format stats");
        std::process::exit(1);
    }

    if let Some(threads) = args.threads
        && let Err(error) = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
    {
        eprintln!("Failed to configure thread pool: {error}");
        std::process::exit(1);
    }

    let CollectedFiles { mut files, roots } = collect_files_or_exit(&args.patterns);

    if files.is_empty() {
        eprintln!("No .vue files found matching the patterns");
        std::process::exit(1);
    }

    let stats_only = matches!(args.format, OutputFormat::Stats);
    let planned_inputs = if stats_only {
        Vec::new()
    } else {
        let inputs = match plan_inputs(std::mem::take(&mut files), &roots) {
            Ok(inputs) => inputs,
            Err(error) => {
                eprintln!("\x1b[31mError:\x1b[0m {error}");
                std::process::exit(1);
            }
        };
        if let Err(error) = preflight_outputs(&inputs, &args.output, args.format, args.script_ext) {
            eprintln!("\x1b[31mError:\x1b[0m {error}");
            std::process::exit(1);
        }
        inputs
    };

    let total_files = if stats_only {
        files.len()
    } else {
        planned_inputs.len()
    };
    let stats = CompileStats::new(total_files);
    let collect_elapsed = start.elapsed();

    args.profile_export.begin(args.profile);
    if args.profile {
        eprintln!(
            "Found {} files in {:.4}s. Compiling using {} threads...",
            total_files,
            collect_elapsed.as_secs_f64(),
            rayon::current_num_threads()
        );
        eprintln!();
    }

    let errors: Mutex<Vec<CompileError>> = Mutex::new(Vec::new());
    let slow_files: Mutex<Vec<FileProfile>> = Mutex::new(Vec::new());
    let profiles: Mutex<Vec<FileProfile>> = Mutex::new(Vec::new());

    let compile_start = Instant::now();
    let compile_settings = CompileFileSettings::resolve(&args, build_config);

    let results: Vec<_> = if stats_only {
        let compile_cache = StatsCompileCache::default();
        files.par_iter().for_each(|path| {
            match compile_file_stats_with_cache(path, &compile_settings, &stats, &compile_cache) {
                Ok((output_bytes, profile)) => {
                    stats.success.fetch_add(1, Ordering::Relaxed);
                    stats
                        .output_bytes
                        .fetch_add(output_bytes, Ordering::Relaxed);

                    if profile.is_slow(slow_threshold)
                        && let Ok(mut slow) = slow_files.lock()
                    {
                        slow.push(profile.clone());
                    }

                    if args.profile
                        && let Ok(mut p) = profiles.lock()
                    {
                        p.push(profile);
                    }
                }
                Err(err) => {
                    stats.failed.fetch_add(1, Ordering::Relaxed);

                    if let Ok(mut errs) = errors.lock() {
                        errs.push(err);
                    }
                }
            }
        });
        Vec::new()
    } else {
        planned_inputs
            .par_iter()
            .map(|input| {
                match compile_file_with_profile(&input.source, &compile_settings, &stats) {
                    Ok((output, profile)) => {
                        stats.success.fetch_add(1, Ordering::Relaxed);
                        stats
                            .output_bytes
                            .fetch_add(output.code.len(), Ordering::Relaxed);

                        // Check for slow files
                        if profile.is_slow(slow_threshold)
                            && let Ok(mut slow) = slow_files.lock()
                        {
                            slow.push(profile.clone());
                        }

                        if args.profile
                            && let Ok(mut p) = profiles.lock()
                        {
                            p.push(profile);
                        }

                        Some(CompiledBuildOutput { input, output })
                    }
                    Err(err) => {
                        stats.failed.fetch_add(1, Ordering::Relaxed);
                        fallback::record_error(&errors, err.clone());
                        args.continue_on_error.then(|| CompiledBuildOutput {
                            input,
                            output: fallback::fallback_output(&input.source, &err),
                        })
                    }
                }
            })
            .collect()
    };
    let compile_elapsed = compile_start.elapsed();

    let io_start = Instant::now();
    match args.format {
        OutputFormat::Stats => {}
        OutputFormat::Js | OutputFormat::Json => {
            if let Err(error) = write_outputs(
                results.into_iter().flatten(),
                &args.output,
                args.format,
                args.script_ext,
            ) {
                eprintln!("\x1b[31mError:\x1b[0m {error}");
                std::process::exit(1);
            }
        }
    }
    let io_elapsed = io_start.elapsed();

    let total_elapsed = start.elapsed();
    let success = stats.success.load(Ordering::Relaxed);
    let failed = stats.failed.load(Ordering::Relaxed);

    // Show slow file warnings
    let slow_files = slow_files.into_inner().unwrap_or_default();
    if !slow_files.is_empty() {
        eprintln!();
        eprintln!(
            "\x1b[33m\u{26a0} {} slow file(s) detected (>{} ms):\x1b[0m",
            slow_files.len(),
            args.slow_threshold
        );
        eprintln!();

        let mut sorted_slow = slow_files;
        sorted_slow.sort_by_key(|file| std::cmp::Reverse(file.total_time));

        for file in sorted_slow.iter().take(10) {
            eprintln!(
                "  \x1b[33m{}\x1b[0m - {:.2}ms (parse: {:.2}ms, compile: {:.2}ms)",
                file.path.display(),
                file.total_time.as_secs_f64() * 1000.0,
                file.parse_time.as_secs_f64() * 1000.0,
                file.compile_time.as_secs_f64() * 1000.0,
            );

            let suggestions = file.suggestions();
            for suggestion in suggestions {
                eprintln!("    \x1b[90m\u{2192} {}\x1b[0m", suggestion);
            }
        }

        if sorted_slow.len() > 10 {
            eprintln!("  ... and {} more", sorted_slow.len() - 10);
        }
        eprintln!();
    }

    // Show collected errors
    let errors = errors.into_inner().unwrap_or_default();
    fallback::report_errors(&errors);

    // Before the failure exit below, so failed compiles still report what ran.
    args.profile_export.finish("build", args.profile);

    // Profile breakdown
    if args.profile {
        let profiler = global_profiler();
        let allocation_summary = profile_support::allocation_snapshot();
        let counter_summary = profiler.counter_summary();
        let operation_summary = profiler.summary();
        profiler.disable();
        let total_parse = stats.total_parse_time();
        let total_compile = stats.total_compile_time();

        let mut all_profiles = profiles.into_inner().unwrap_or_default();
        all_profiles.sort_by_key(|profile| std::cmp::Reverse(profile.total_time));

        let phases = [
            ProfilePhase {
                name: "collect files",
                duration: collect_elapsed,
                kind: ProfilePhaseKind::Wall,
                note: "ignore-aware walk",
            },
            ProfilePhase {
                name: "compile wall",
                duration: compile_elapsed,
                kind: ProfilePhaseKind::Wall,
                note: "parallel worker elapsed time",
            },
            ProfilePhase {
                name: "parse total",
                duration: total_parse,
                kind: ProfilePhaseKind::Cumulative,
                note: "sum across worker threads",
            },
            ProfilePhase {
                name: "compile total",
                duration: total_compile,
                kind: ProfilePhaseKind::Cumulative,
                note: "sum across worker threads",
            },
            ProfilePhase {
                name: "write outputs",
                duration: io_elapsed,
                kind: ProfilePhaseKind::Wall,
                note: "filesystem writes",
            },
        ];

        let file_rows: Vec<_> = all_profiles
            .iter()
            .map(|file| ProfileFileRow {
                path: file.path.clone(),
                bytes: file.file_size,
                total: file.total_time,
                primary_label: "parse",
                primary: file.parse_time,
                secondary_label: "compile",
                secondary: file.compile_time,
                note: Some(file.profile_note.clone()),
            })
            .collect();

        let mut recommendations: Vec<String> = Vec::new();
        if let Some(entry) = operation_summary.entries.first() {
            recommendations.push(cstr!(
                "Deepest hot operation: {} took {:.2}ms total across {} call(s).",
                entry.name,
                entry.total.as_secs_f64() * 1000.0,
                entry.count
            ));
        }
        for file in all_profiles
            .iter()
            .filter(|file| file.is_slow(slow_threshold))
            .take(4)
        {
            let suggestions = file.suggestions();
            if suggestions.is_empty() {
                recommendations.push(cstr!(
                    "{} crossed the slow threshold; inspect template/script balance first.",
                    file.path.display()
                ));
            } else {
                for suggestion in suggestions.into_iter().take(2) {
                    recommendations.push(cstr!("{}: {}", file.path.display(), suggestion));
                }
            }
        }
        let total_bytes = stats.total_bytes.load(Ordering::Relaxed);
        let output_bytes = stats.output_bytes.load(Ordering::Relaxed);
        if matches!(args.format, OutputFormat::Js | OutputFormat::Json)
            && io_elapsed > compile_elapsed
        {
            recommendations.push(
                "Output I/O is larger than compile wall time; use --format stats when profiling compiler cost only."
                    .into(),
            );
        }

        let summary = cstr!(
            "{} of {} file(s) compiled, {} failed, {} output byte(s), {} worker thread(s)",
            success,
            stats.total_files,
            failed,
            output_bytes,
            rayon::current_num_threads()
        );
        let report = ProfileReport {
            title: "build",
            summary: summary.as_str(),
            total: total_elapsed,
            phases: &phases,
            files: &file_rows,
            slow_threshold,
            throughput_bytes: Some(total_bytes),
            operations: Some(&operation_summary),
            counters: Some(&counter_summary),
            allocations: allocation_summary,
            recommendations: &recommendations,
        };
        print_profile_report(&report);
    }

    // Final summary
    if failed > 0 {
        eprintln!(
            "\x1b[31m\u{2717} {} file(s) failed\x1b[0m, {} compiled in {:.4}s",
            failed,
            success,
            total_elapsed.as_secs_f64()
        );
        if args.declaration {
            eprintln!("Skipping declaration emit because the build failed.");
        }
        std::process::exit(1);
    } else {
        let file_word = if success == 1 { "file" } else { "files" };
        eprintln!(
            "\x1b[32m\u{2713} {} {} compiled in {:.4}s\x1b[0m",
            success,
            file_word,
            total_elapsed.as_secs_f64()
        );
    }

    if args.declaration {
        declarations::emit(&args, &planned_inputs);
    }
}
