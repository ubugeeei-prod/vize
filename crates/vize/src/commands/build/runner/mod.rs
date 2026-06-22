//! Build command execution logic.
//!
//! Contains the main compilation pipeline, file collection, pattern matching,
//! and per-file compilation with profiling.

mod cache;
mod collect;
mod compile;
mod settings;

use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, atomic::Ordering},
    time::{Duration, Instant},
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use vize_carton::String;
use vize_carton::cstr;
use vize_carton::profile;
use vize_carton::profiler::global_profiler;

use crate::profile_support;
use vize_curator::profile::{
    ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report,
};

use super::{
    BuildArgs, OutputFormat,
    config::{CompileError, CompileStats, ErrorPhase, FileProfile, get_output_extension},
};

use cache::StatsCompileCache;
use collect::collect_files;
use compile::{compile_file_stats_with_cache, compile_file_with_profile};
use settings::{CompileFileSettings, template_syntax_mode};

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
    let compiler_template_syntax = if args.no_config {
        None
    } else {
        crate::config::load_compiler_template_syntax(args.config.as_deref())
    };
    let configured_dialect = if args.no_config {
        None
    } else {
        crate::config::load_compiler_vue_version(args.config.as_deref())
    };
    let configured_host_compiler = if args.no_config {
        None
    } else {
        crate::config::load_compiler_host_compiler(args.config.as_deref())
    };
    if configured_dialect.is_some_and(|dialect| dialect.is_legacy())
        && configured_host_compiler == Some(false)
    {
        eprintln!(
            "\x1b[31mError:\x1b[0m compiler.compatibility.hostCompiler=false is unsupported for Vue 2 compatibility mode"
        );
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

    let files = collect_files(&args.patterns);

    if files.is_empty() {
        eprintln!("No .vue files found matching the patterns");
        std::process::exit(1);
    }

    let stats = CompileStats::new(files.len());
    let collect_elapsed = start.elapsed();

    if args.profile {
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
        eprintln!(
            "Found {} files in {:.4}s. Compiling using {} threads...",
            files.len(),
            collect_elapsed.as_secs_f64(),
            rayon::current_num_threads()
        );
        eprintln!();
    }

    // Collect errors and slow files
    let errors: Mutex<Vec<CompileError>> = Mutex::new(Vec::new());
    let slow_files: Mutex<Vec<FileProfile>> = Mutex::new(Vec::new());
    let profiles: Mutex<Vec<FileProfile>> = Mutex::new(Vec::new());

    let compile_start = Instant::now();
    let compile_settings = CompileFileSettings {
        ssr: args.ssr,
        vapor: args.vapor,
        custom_renderer: args.custom_renderer,
        template_syntax: args
            .template_syntax
            .map(Into::into)
            .unwrap_or_else(|| template_syntax_mode(compiler_template_syntax)),
        dialect: configured_dialect.unwrap_or_default(),
        script_ext: args.script_ext,
        record_profile_totals: args.profile,
    };

    let stats_only = matches!(args.format, OutputFormat::Stats);
    let results: Vec<_> = if stats_only {
        let compile_cache = StatsCompileCache::default();
        files.par_iter().for_each(|path| {
            match compile_file_stats_with_cache(path, compile_settings, &stats, &compile_cache) {
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
        files
            .par_iter()
            .map(
                |path| match compile_file_with_profile(path, compile_settings, &stats) {
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

                        Some((path.clone(), output))
                    }
                    Err(err) => {
                        stats.failed.fetch_add(1, Ordering::Relaxed);

                        if let Ok(mut errs) = errors.lock() {
                            errs.push(err);
                        }

                        None
                    }
                },
            )
            .collect()
    };
    let compile_elapsed = compile_start.elapsed();

    let io_start = Instant::now();
    match args.format {
        OutputFormat::Stats => {}
        OutputFormat::Js | OutputFormat::Json => {
            // Create the output directory once per build, then write every
            // generated file. Calling `create_dir_all` from each worker looked
            // harmless but showed up in profiles as repeated metadata syscalls,
            // especially when benchmarking many generated SFCs.
            match profile!(
                "cli.build.output.create_dir_all",
                fs::create_dir_all(&args.output)
            ) {
                Ok(()) => global_profiler().record_fs_create_dir_all(),
                Err(error) => {
                    global_profiler().record_fs_create_dir_all_failure();
                    eprintln!(
                        "Failed to create output directory {}: {error}",
                        args.output.display()
                    );
                    std::process::exit(1);
                }
            }

            for (path, output) in results.into_iter().flatten() {
                let ext = match args.format {
                    OutputFormat::Js => get_output_extension(&output.script_lang, args.script_ext),
                    OutputFormat::Json => "json",
                    // Panic path by control-flow invariant: this match is inside
                    // the `OutputFormat::Js | OutputFormat::Json` arm above.
                    // Keeping the enum match explicit lets the compiler keep
                    // checking newly added output formats here.
                    OutputFormat::Stats => unreachable!(),
                };

                let filename = path
                    .file_name()
                    .map(|f| PathBuf::from(f).with_extension(ext))
                    .unwrap_or_else(|| PathBuf::from("output").with_extension(ext));
                let out_path = args.output.join(filename);

                let content: String = match args.format {
                    OutputFormat::Js => output.code,
                    OutputFormat::Json =>
                    {
                        #[allow(clippy::disallowed_methods)]
                        serde_json::to_string_pretty(&output)
                            .unwrap_or_default()
                            .into()
                    }
                    // Panic path by the same outer-match invariant as `ext`.
                    OutputFormat::Stats => unreachable!(),
                };

                let bytes = content.len();
                match profile!(
                    "cli.build.output.write",
                    fs::write(&out_path, content.as_str())
                ) {
                    Ok(()) => global_profiler().record_fs_write(bytes),
                    Err(error) => {
                        global_profiler().record_fs_write_failure(bytes);
                        eprintln!("Failed to write {}: {}", out_path.display(), error);
                    }
                }
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
    if !errors.is_empty() {
        eprintln!();
        eprintln!(
            "\x1b[31m\u{2717} {} error(s) occurred:\x1b[0m",
            errors.len()
        );
        eprintln!();

        // Group errors by phase
        let read_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.phase == ErrorPhase::Read)
            .collect();
        let parse_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.phase == ErrorPhase::Parse)
            .collect();
        let compile_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.phase == ErrorPhase::Compile)
            .collect();

        if !read_errors.is_empty() {
            eprintln!("  \x1b[31mRead errors ({}):\x1b[0m", read_errors.len());
            for err in &read_errors {
                eprintln!("    {} - {}", err.path.display(), err.error);
            }
            eprintln!();
        }

        if !parse_errors.is_empty() {
            eprintln!("  \x1b[31mParse errors ({}):\x1b[0m", parse_errors.len());
            for err in &parse_errors {
                eprintln!("    \x1b[1m{}\x1b[0m", err.path.display());
                for line in err.error.lines() {
                    eprintln!("      {}", line);
                }
            }
            eprintln!();
        }

        if !compile_errors.is_empty() {
            eprintln!(
                "  \x1b[31mCompile errors ({}):\x1b[0m",
                compile_errors.len()
            );
            for err in &compile_errors {
                eprintln!("    \x1b[1m{}\x1b[0m", err.path.display());
                for line in err.error.lines() {
                    eprintln!("      {}", line);
                }
            }
            eprintln!();
        }
    }

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
                note: Some(cstr!(
                    "template {} B, script {} B, styles {}",
                    file.template_size,
                    file.script_size,
                    file.style_count
                )),
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
}
