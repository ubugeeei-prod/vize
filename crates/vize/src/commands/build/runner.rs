//! Build command execution logic.
//!
//! Contains the main compilation pipeline, file collection, pattern matching,
//! and per-file compilation with profiling.

#![allow(clippy::disallowed_macros)]

use std::{
    fs,
    path::PathBuf,
    sync::{atomic::Ordering, Mutex},
    time::{Duration, Instant},
};

use ignore::Walk;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use vize_atelier_sfc::{
    compile_sfc, parse_sfc, ScriptCompileOptions, SfcCompileOptions, SfcParseOptions,
    StyleCompileOptions, TemplateCompileOptions,
};
use vize_carton::cstr;
use vize_carton::profile;
use vize_carton::profiler::global_profiler;
use vize_carton::String;
use vize_carton::ToCompactString;

use crate::commands::profile::{
    print_profile_report, ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport,
};

use super::{
    config::{
        get_output_extension, CompileError, CompileOutput, CompileStats, ErrorPhase, FileProfile,
    },
    BuildArgs, OutputFormat, ScriptExtension,
};

/// Main entry point for the build command.
pub(crate) fn run(args: BuildArgs) {
    let start = Instant::now();
    let slow_threshold = Duration::from_millis(args.slow_threshold);

    if let Some(threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .expect("Failed to configure thread pool");
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
    let results: Vec<_> = files
        .par_iter()
        .map(|path| {
            let source_size = fs::metadata(path).map(|m| m.len() as usize).unwrap_or(0);
            stats.total_bytes.fetch_add(source_size, Ordering::Relaxed);

            match compile_file_with_profile(path, args.ssr, args.script_ext, &stats) {
                Ok((output, profile)) => {
                    stats.success.fetch_add(1, Ordering::Relaxed);
                    stats
                        .output_bytes
                        .fetch_add(output.code.len(), Ordering::Relaxed);

                    // Check for slow files
                    if profile.is_slow(slow_threshold) {
                        if let Ok(mut slow) = slow_files.lock() {
                            slow.push(profile.clone());
                        }
                    }

                    if args.profile {
                        if let Ok(mut p) = profiles.lock() {
                            p.push(profile);
                        }
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
            fs::create_dir_all(&args.output).expect("Failed to create output directory");

            for (path, output) in results.into_iter().flatten() {
                let ext = match args.format {
                    OutputFormat::Js => get_output_extension(&output.script_lang, args.script_ext),
                    OutputFormat::Json => "json",
                    OutputFormat::Stats => unreachable!(),
                };

                let filename = path
                    .file_name()
                    .map(|f| PathBuf::from(f).with_extension(ext))
                    .unwrap_or_else(|| PathBuf::from("output").with_extension(ext));
                let out_path = args.output.join(filename);

                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).expect("Failed to create output subdirectory");
                }

                let content: String = match args.format {
                    OutputFormat::Js => output.code,
                    OutputFormat::Json =>
                    {
                        #[allow(clippy::disallowed_methods)]
                        serde_json::to_string_pretty(&output)
                            .unwrap_or_default()
                            .into()
                    }
                    OutputFormat::Stats => unreachable!(),
                };

                fs::write(&out_path, content).unwrap_or_else(|e| {
                    eprintln!("Failed to write {}: {}", out_path.display(), e);
                });
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
        sorted_slow.sort_by(|a, b| b.total_time.cmp(&a.total_time));

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
        let operation_summary = profiler.summary();
        profiler.disable();
        let total_parse = stats
            .total_parse_time
            .lock()
            .map(|d| *d)
            .unwrap_or(Duration::ZERO);
        let total_compile = stats
            .total_compile_time
            .lock()
            .map(|d| *d)
            .unwrap_or(Duration::ZERO);

        let mut all_profiles = profiles.into_inner().unwrap_or_default();
        all_profiles.sort_by(|a, b| b.total_time.cmp(&a.total_time));

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

    if path.is_file() && pattern.ends_with(".vue") {
        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            let parent_str = if parent_str.is_empty() {
                "."
            } else {
                &parent_str
            };
            return (parent_str.to_compact_string(), pattern.to_compact_string());
        }
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

    if pattern.contains("**/*.vue") {
        if let Some(prefix_end) = pattern.find("**") {
            let prefix = &pattern[..prefix_end];
            let prefix_normalized = prefix.trim_end_matches('/');
            return path_str.contains(&format!("{}/", prefix_normalized))
                && path_str.ends_with(".vue");
        }
    }

    if pattern.ends_with(".vue") {
        let pattern_normalized = pattern.replace("\\", "/");
        return path_str == pattern_normalized
            || path_str.ends_with(&format!("/{}", pattern_normalized));
    }

    path_str.ends_with(".vue")
}

/// Detect the script language from `<script lang="...">` in the SFC source.
fn detect_script_lang(source: &str) -> String {
    let script_pattern = regex_lite::Regex::new(r#"<script[^>]*\blang\s*=\s*["']([^"']+)["']"#)
        .expect("Invalid regex");

    if let Some(captures) = script_pattern.captures(source) {
        if let Some(lang) = captures.get(1) {
            return lang.as_str().to_compact_string();
        }
    }

    "js".into()
}

/// Compile a single `.vue` file with profiling information.
fn compile_file_with_profile(
    path: &PathBuf,
    ssr: bool,
    script_ext: ScriptExtension,
    stats: &CompileStats,
) -> Result<(CompileOutput, FileProfile), CompileError> {
    let file_start = Instant::now();

    // Read file
    let source = fs::read_to_string(path).map_err(|e| CompileError {
        path: path.clone(),
        error: cstr!("Failed to read file: {}", e),
        phase: ErrorPhase::Read,
    })?;

    let file_size = source.len();

    let filename: String = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("anonymous.vue")
        .into();

    let script_lang = detect_script_lang(&source);

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
    stats.add_parse_time(parse_time);

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
    let is_ts = matches!(script_ext, ScriptExtension::Preserve);
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
            ssr,
            is_ts,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename.clone(),
            scoped: has_scoped,
            ..Default::default()
        },
        vapor: false,
        scope_id: None,
    };

    let result = profile!(
        "atelier.sfc.compile",
        compile_sfc(&descriptor, compile_opts)
    )
    .map_err(|e| CompileError {
        path: path.clone(),
        error: e.message,
        phase: ErrorPhase::Compile,
    })?;
    let compile_time = compile_start.elapsed();
    stats.add_compile_time(compile_time);

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
    };

    Ok((output, profile))
}
