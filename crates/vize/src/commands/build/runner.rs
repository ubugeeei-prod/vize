//! Build command execution logic.
//!
//! Contains the main compilation pipeline, file collection, pattern matching,
//! and per-file compilation with profiling.

use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, atomic::Ordering},
    time::{Duration, Instant},
};

use ignore::Walk;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
    TemplateCompileOptions, compile_sfc, compile_sfc_with_vue_parser_quirks, parse_sfc,
};
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_carton::cstr;
use vize_carton::profile;
use vize_carton::profiler::{allocation_snapshot, global_profiler};

use vize_curator::profile::{
    ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report,
};

use super::{
    BuildArgs, OutputFormat, ScriptExtension,
    config::{
        CompileError, CompileOutput, CompileStats, ErrorPhase, FileProfile, get_output_extension,
    },
};

/// Main entry point for the build command.
pub(crate) fn run(args: BuildArgs) {
    let start = Instant::now();
    let slow_threshold = Duration::from_millis(args.slow_threshold);
    if let Some(config) = args.config.as_ref()
        && !args.no_config
        && !config.exists()
    {
        eprintln!("Could not find config file: {}", config.display());
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
        vue_parser_quirks: args.vue_parser_quirks,
        script_ext: args.script_ext,
        record_profile_totals: args.profile,
    };
    let results: Vec<_> = files
        .par_iter()
        .map(|path| {
            match compile_file_with_profile(path, compile_settings, &stats) {
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
            }
        })
        .collect();
    let compile_elapsed = compile_start.elapsed();

    let io_start = Instant::now();
    match args.format {
        OutputFormat::Stats => {}
        OutputFormat::Js | OutputFormat::Json => {
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
        let allocation_summary = allocation_snapshot();
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
            allocations: Some(allocation_summary),
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

/// Collect `.vue` files matching the given glob patterns.
#[allow(clippy::disallowed_types)]
fn collect_files(patterns: &[std::string::String]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for pattern in patterns {
        let (root, glob_pattern) = parse_pattern(pattern);

        for entry in Walk::new(&root).flatten() {
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "vue")
                && pattern_matches(path, &glob_pattern)
            {
                files.push(path.to_path_buf());
            }
        }
    }

    files.sort();
    files.dedup();
    files
}

/// Extract a root directory and glob pattern from a user-provided pattern string.
fn parse_pattern(pattern: &str) -> (String, String) {
    if let Some(pos) = pattern.find(['*', '?']) {
        let root_part = &pattern[..pos];
        if let Some(last_slash) = root_part.rfind('/') {
            let root = &pattern[..last_slash];
            let root = if root.is_empty() { "." } else { root };
            return (root.to_compact_string(), pattern.to_compact_string());
        }
    }

    let path = std::path::Path::new(pattern);
    if path.is_dir() {
        return (pattern.to_compact_string(), cstr!("{}/**/*.vue", pattern));
    }

    if path.is_file()
        && pattern.ends_with(".vue")
        && let Some(parent) = path.parent()
    {
        let parent_str = parent.to_string_lossy();
        let parent_str = if parent_str.is_empty() {
            "."
        } else {
            &parent_str
        };
        return (parent_str.to_compact_string(), pattern.to_compact_string());
    }

    (".".into(), pattern.to_compact_string())
}

/// Check whether a file path matches a glob-like pattern.
#[allow(clippy::disallowed_types, clippy::disallowed_methods)]
fn pattern_matches(path: &std::path::Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy().replace("\\", "/");

    if pattern == "./**/*.vue" || pattern == "**/*.vue" {
        return path_str.ends_with(".vue");
    }

    if pattern.contains("**/*.vue")
        && let Some(prefix_end) = pattern.find("**")
    {
        let prefix = &pattern[..prefix_end];
        let prefix_normalized = prefix.trim_end_matches('/');
        let has_prefix_dir = prefix_normalized.is_empty()
            || path_str.match_indices(prefix_normalized).any(|(idx, _)| {
                path_str.as_bytes().get(idx + prefix_normalized.len()) == Some(&b'/')
            });
        return has_prefix_dir && path_str.ends_with(".vue");
    }

    if pattern.ends_with(".vue") {
        let pattern_normalized = pattern.replace("\\", "/");
        if path_str == pattern_normalized {
            return true;
        }

        if !path_str.ends_with(pattern_normalized.as_str()) {
            return false;
        }

        let prefix_len = path_str.len() - pattern_normalized.len();
        let Some(separator_idx) = prefix_len.checked_sub(1) else {
            return false;
        };
        return path_str.as_bytes().get(separator_idx) == Some(&b'/');
    }

    path_str.ends_with(".vue")
}

/// Compile a single `.vue` file with profiling information.
#[derive(Clone, Copy)]
struct CompileFileSettings {
    ssr: bool,
    vapor: bool,
    custom_renderer: bool,
    vue_parser_quirks: bool,
    script_ext: ScriptExtension,
    record_profile_totals: bool,
}

fn compile_file_with_profile(
    path: &PathBuf,
    settings: CompileFileSettings,
    stats: &CompileStats,
) -> Result<(CompileOutput, FileProfile), CompileError> {
    let file_start = Instant::now();

    // Read file
    let source = match profile!("cli.build.file.read", fs::read_to_string(path)) {
        Ok(source) => {
            global_profiler().record_fs_read_to_string(source.len());
            source
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            return Err(CompileError {
                path: path.clone(),
                error: cstr!("Failed to read file: {}", error),
                phase: ErrorPhase::Read,
            });
        }
    };

    let file_size = source.len();
    stats.total_bytes.fetch_add(file_size, Ordering::Relaxed);

    let filename: String = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("anonymous.vue")
        .into();

    // Parse
    let parse_start = Instant::now();
    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };

    let descriptor =
        profile!("atelier.sfc.parse", parse_sfc(&source, parse_opts)).map_err(|e| {
            CompileError {
                path: path.clone(),
                error: e.message,
                phase: ErrorPhase::Parse,
            }
        })?;
    let parse_time = parse_start.elapsed();
    if settings.record_profile_totals {
        stats.add_parse_time(parse_time);
    }

    let script_lang = descriptor
        .script_setup
        .as_ref()
        .and_then(|s| s.lang.as_deref())
        .or_else(|| descriptor.script.as_ref().and_then(|s| s.lang.as_deref()))
        .unwrap_or("js")
        .to_compact_string();

    // Calculate sizes
    let template_size = descriptor
        .template
        .as_ref()
        .map(|t| t.content.len())
        .unwrap_or(0);
    let script_size = descriptor
        .script
        .as_ref()
        .map(|s| s.content.len())
        .unwrap_or(0)
        + descriptor
            .script_setup
            .as_ref()
            .map(|s| s.content.len())
            .unwrap_or(0);
    let style_count = descriptor.styles.len();

    // Compile
    let compile_start = Instant::now();
    let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
    let is_ts = matches!(settings.script_ext, ScriptExtension::Preserve);
    let compile_opts = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            is_ts,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            scoped: has_scoped,
            ssr: settings.ssr,
            is_ts,
            custom_renderer: settings.custom_renderer,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename.clone(),
            scoped: has_scoped,
            ..Default::default()
        },
        vapor: settings.vapor,
        scope_id: None,
    };

    let result = profile!(
        "atelier.sfc.compile",
        if settings.vue_parser_quirks {
            compile_sfc_with_vue_parser_quirks(&descriptor, compile_opts)
        } else {
            compile_sfc(&descriptor, compile_opts)
        }
    )
    .map_err(|e| CompileError {
        path: path.clone(),
        error: e.message,
        phase: ErrorPhase::Compile,
    })?;
    let compile_time = compile_start.elapsed();
    if settings.record_profile_totals {
        stats.add_compile_time(compile_time);
    }

    let total_time = file_start.elapsed();

    let profile = FileProfile {
        path: path.clone(),
        file_size,
        parse_time,
        compile_time,
        total_time,
        template_size,
        script_size,
        style_count,
    };

    let output = CompileOutput {
        filename,
        code: result.code,
        css: result.css,
        errors: result.errors.into_iter().map(|e| e.message).collect(),
        warnings: result.warnings.into_iter().map(|e| e.message).collect(),
        script_lang,
        macro_artifacts: result.macro_artifacts,
    };

    Ok((output, profile))
}
