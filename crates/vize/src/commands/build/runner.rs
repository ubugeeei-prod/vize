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
use vize_carton::cstr;
use vize_carton::hash::hash_str;
use vize_carton::profile;
use vize_carton::profiler::{allocation_snapshot, global_profiler};
use vize_carton::{FxHashMap, String, ToCompactString};

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

impl CompileFileSettings {
    /// Packs every compile option that can change stats output into a tiny cache key.
    ///
    /// `record_profile_totals` is intentionally excluded: enabling profiling changes
    /// accounting side effects, not parse/compile output. Script extension is included
    /// because preserving TypeScript can change generated code size.
    fn cache_bits(self) -> u8 {
        u8::from(self.ssr)
            | (u8::from(self.vapor) << 1)
            | (u8::from(self.custom_renderer) << 2)
            | (u8::from(self.vue_parser_quirks) << 3)
            | match self.script_ext {
                ScriptExtension::Preserve => 1 << 4,
                ScriptExtension::Downcompile => 0,
            }
    }
}

/// Fingerprint for sharing stats-only SFC compiles across repeated file bodies.
///
/// The stats formatter never writes generated code, so it only needs stable
/// aggregate properties: output byte count and block sizes. Keeping a full copy
/// of every source in the key would bring back the memory pressure this path is
/// trying to avoid, so the key uses the existing fast hash plus source length.
///
/// `component_name_len` is part of the key because generated `__name` text
/// affects byte counts even when the actual component name is otherwise
/// irrelevant to stats. The actual name is deliberately not stored in the key;
/// `should_cache_stats_compile` rejects cases where the source can observe the
/// specific component name through self-component resolution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct StatsCompileCacheKey {
    /// Fast content fingerprint used to group identical generated benchmark bodies.
    source_hash: u64,
    /// Guards the fingerprint and keeps same-hash, different-length sources apart.
    source_len: usize,
    /// Captures byte-size changes from the filename-derived `__name` field.
    component_name_len: usize,
    /// Compact representation of output-affecting CLI compile options.
    settings: u8,
}

/// Cached result of a stats-only compile.
///
/// Success entries hold only the values needed to reproduce aggregate stats and
/// file-profile metadata. Failure entries are cached as well so repeated invalid
/// inputs do not re-run the parser/compiler only to report the same phase.
#[derive(Clone)]
enum StatsCompileCacheEntry {
    Success {
        output_bytes: usize,
        template_size: usize,
        script_size: usize,
        style_count: usize,
    },
    Failure {
        phase: ErrorPhase,
        message: String,
    },
}

/// Shared cache for one `vize build --format stats` invocation.
///
/// A single mutex is enough here because the expensive work is parse/compile,
/// not the lookup. If two worker threads miss the same key at the same time they
/// may both compile once; the `entry(...).or_insert_with(...)` write keeps the
/// cache deterministic, and correctness does not depend on suppressing that
/// benign race.
#[derive(Default)]
struct StatsCompileCache {
    entries: Mutex<FxHashMap<StatsCompileCacheKey, StatsCompileCacheEntry>>,
}

/// Returns whether a source can reuse another file's stats-only compile result.
///
/// Filename-derived output is mostly byte-count stable: generated scope IDs are
/// fixed-width hashes, and different component names only matter by length.
/// Self-component resolution is the exception. If the template mentions its own
/// component name, changing the filename can change whether that tag is treated
/// as a component, which can alter helper usage and generated code shape. Those
/// cases are compiled normally.
fn should_cache_stats_compile(source: &str, component_name: &str) -> bool {
    if component_name.is_empty() {
        return true;
    }

    !source.contains(component_name)
        && !source.contains(component_name_to_kebab_case(component_name).as_str())
}

/// Converts a PascalCase filename stem to the kebab-case spelling Vue templates use.
///
/// This is a conservative guard for self-component detection, not a general Vue
/// name normalizer. Exact stem matching is checked separately, and non-ASCII
/// names fall back to that exact spelling path.
fn component_name_to_kebab_case(component_name: &str) -> String {
    let mut out = String::with_capacity(component_name.len());
    for (index, ch) in component_name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index != 0 {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// Compiles one file for `--format stats`, using content-addressed reuse.
///
/// The normal build path returns `CompileOutput` because JavaScript/JSON output
/// must preserve per-file generated code. The stats path only needs aggregate
/// counters, so repeated source bodies can skip parse/compile and reuse the
/// cached output length and block-size metadata.
///
/// Every file is still read and counted. Cache hits get zero parse/compile time
/// in their `FileProfile` so profile totals represent actual compiler work
/// instead of multiplying one compile by the number of duplicates.
fn compile_file_stats_with_cache(
    path: &PathBuf,
    settings: CompileFileSettings,
    stats: &CompileStats,
    cache: &StatsCompileCache,
) -> Result<(usize, FileProfile), CompileError> {
    let file_start = Instant::now();

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
    let component_name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
    let cache_key =
        should_cache_stats_compile(&source, component_name).then(|| StatsCompileCacheKey {
            source_hash: hash_str(&source),
            source_len: file_size,
            component_name_len: component_name.len(),
            settings: settings.cache_bits(),
        });

    if let Some(key) = cache_key
        && let Some(entry) = cache
            .entries
            .lock()
            .map(|entries| entries.get(&key).cloned())
            .unwrap_or(None)
    {
        return match entry {
            StatsCompileCacheEntry::Success {
                output_bytes,
                template_size,
                script_size,
                style_count,
            } => Ok((
                output_bytes,
                FileProfile {
                    path: path.clone(),
                    file_size,
                    parse_time: Duration::ZERO,
                    compile_time: Duration::ZERO,
                    total_time: file_start.elapsed(),
                    template_size,
                    script_size,
                    style_count,
                },
            )),
            StatsCompileCacheEntry::Failure { phase, message } => Err(CompileError {
                path: path.clone(),
                error: message,
                phase,
            }),
        };
    }

    let parse_start = Instant::now();
    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };
    let descriptor = match profile!("atelier.sfc.parse", parse_sfc(&source, parse_opts)) {
        Ok(descriptor) => descriptor,
        Err(error) => {
            if let Some(key) = cache_key
                && let Ok(mut entries) = cache.entries.lock()
            {
                entries
                    .entry(key)
                    .or_insert_with(|| StatsCompileCacheEntry::Failure {
                        phase: ErrorPhase::Parse,
                        message: error.message.clone(),
                    });
            }
            return Err(CompileError {
                path: path.clone(),
                error: error.message,
                phase: ErrorPhase::Parse,
            });
        }
    };
    let parse_time = parse_start.elapsed();
    if settings.record_profile_totals {
        stats.add_parse_time(parse_time);
    }

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
            id: filename,
            scoped: has_scoped,
            ..Default::default()
        },
        vapor: settings.vapor,
        scope_id: None,
    };

    let result = match profile!(
        "atelier.sfc.compile",
        if settings.vue_parser_quirks {
            compile_sfc_with_vue_parser_quirks(&descriptor, compile_opts)
        } else {
            compile_sfc(&descriptor, compile_opts)
        }
    ) {
        Ok(result) => result,
        Err(error) => {
            if let Some(key) = cache_key
                && let Ok(mut entries) = cache.entries.lock()
            {
                entries
                    .entry(key)
                    .or_insert_with(|| StatsCompileCacheEntry::Failure {
                        phase: ErrorPhase::Compile,
                        message: error.message.clone(),
                    });
            }
            return Err(CompileError {
                path: path.clone(),
                error: error.message,
                phase: ErrorPhase::Compile,
            });
        }
    };
    let compile_time = compile_start.elapsed();
    if settings.record_profile_totals {
        stats.add_compile_time(compile_time);
    }

    let output_bytes = result.code.len();
    if let Some(key) = cache_key
        && let Ok(mut entries) = cache.entries.lock()
    {
        entries
            .entry(key)
            .or_insert_with(|| StatsCompileCacheEntry::Success {
                output_bytes,
                template_size,
                script_size,
                style_count,
            });
    }

    Ok((
        output_bytes,
        FileProfile {
            path: path.clone(),
            file_size,
            parse_time,
            compile_time,
            total_time: file_start.elapsed(),
            template_size,
            script_size,
            style_count,
        },
    ))
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
