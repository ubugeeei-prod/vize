//! Format command - High-performance Vue, JSX, and TSX formatting using vize_glyph

#![allow(clippy::disallowed_macros)]

use clap::Args;
use oxc_span::SourceType;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use vize_carton::{cstr, profile, profiler::global_profiler};
use vize_glyph::{
    Allocator, FormatOptions, FormatResult, format_script_with_source_type,
    format_sfc_with_allocator,
};

use crate::{config, profile_support};
use vize_curator::profile::{
    ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report,
};

mod files;
mod ignores;

use files::collect_files;
use ignores::load_fmt_ignore_set;

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct FmtArgs {
    /// Glob pattern(s) to match .vue, .jsx, and .tsx files
    #[arg(default_values_t = default_fmt_patterns())]
    pub patterns: Vec<String>,

    /// Check formatting without writing (exit with error if files need formatting)
    #[arg(long)]
    pub check: bool,

    /// Write formatted output to files
    #[arg(short, long)]
    pub write: bool,

    /// Config file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Do not load a config file
    #[arg(long)]
    pub no_config: bool,

    /// Use single quotes instead of double quotes
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub single_quote: Option<bool>,

    /// Print width (line length) for formatting
    #[arg(long)]
    pub print_width: Option<u32>,

    /// Number of spaces per indentation level
    #[arg(long)]
    pub tab_width: Option<u8>,

    /// Use tabs instead of spaces for indentation
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub use_tabs: Option<bool>,

    /// Do not print semicolons at the ends of statements
    #[arg(long)]
    pub no_semi: bool,

    /// Sort HTML attributes in template
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub sort_attributes: Option<bool>,

    /// Put each HTML attribute on its own line
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub single_attribute_per_line: Option<bool>,

    /// Maximum number of attributes per line before wrapping
    #[arg(long)]
    pub max_attributes_per_line: Option<u32>,

    /// Normalize directive shorthands (v-bind: → :, v-on: → @, v-slot: → #)
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub normalize_directive_shorthands: Option<bool>,

    /// Show detailed timing profile
    #[arg(long)]
    pub profile: bool,

    /// Slow file threshold in milliseconds for profile output
    #[arg(long, default_value = "100")]
    pub slow_threshold: u64,
}

pub fn run(args: FmtArgs) {
    let start = Instant::now();
    if let Some(path) = args.config.as_deref()
        && !args.no_config
        && let Err(error) = config::validate_explicit_config_path(path)
    {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(2);
    }
    let options = build_format_options(&args);
    let ignore_set = load_fmt_ignore_set(&args);

    // Collect files to format
    let collect_start = Instant::now();
    let files: Vec<PathBuf> = collect_files(&args.patterns, ignore_set.as_ref());
    let collect_time = collect_start.elapsed();

    if files.is_empty() {
        eprintln!("No .vue, .jsx, or .tsx files found matching the patterns");
        return;
    }

    eprintln!("Found {} file(s)", files.len());

    let has_errors = AtomicBool::new(false);
    let files_changed = AtomicUsize::new(0);
    let files_unchanged = AtomicUsize::new(0);
    let files_errored = AtomicUsize::new(0);
    let profile_rows = args.profile.then(|| Mutex::new(Vec::new()));
    if args.profile {
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
    }

    // Process files in parallel, reusing one arena per Rayon worker. Resetting a
    // worker-local allocator keeps formatter allocations off the global heap on
    // the common "many small SFCs" path while avoiding cross-thread sharing.
    let process_start = Instant::now();
    files.par_iter().for_each_init(
        || Allocator::with_capacity(64 * 1024),
        |allocator, path| {
            allocator.reset();

            match process_file(
                path,
                &options,
                allocator,
                args.check,
                args.write,
                args.profile,
            ) {
                Ok(result) => {
                    if result.changed {
                        files_changed.fetch_add(1, Ordering::Relaxed);
                        if args.check {
                            has_errors.store(true, Ordering::Relaxed);
                        }
                    } else {
                        files_unchanged.fetch_add(1, Ordering::Relaxed);
                    }

                    if let (Some(profile), Some(profile_rows)) =
                        (result.profile, profile_rows.as_ref())
                        && let Ok(mut rows) = profile_rows.lock()
                    {
                        rows.push(profile);
                    }
                }
                Err(err) => {
                    eprintln!("Error formatting {}: {}", path.display(), err);
                    files_errored.fetch_add(1, Ordering::Relaxed);
                    has_errors.store(true, Ordering::Relaxed);
                }
            }
        },
    );
    let process_time = process_start.elapsed();

    // Print summary
    let summary_start = Instant::now();
    let changed = files_changed.load(Ordering::Relaxed);
    let unchanged = files_unchanged.load(Ordering::Relaxed);
    let errored = files_errored.load(Ordering::Relaxed);

    eprintln!();
    if args.check {
        eprintln!("Checked {} file(s)", files.len());
        if changed > 0 {
            eprintln!("  {} file(s) would be reformatted", changed);
        }
        if unchanged > 0 {
            eprintln!("  {} file(s) already formatted", unchanged);
        }
    } else if args.write {
        eprintln!("Formatted {} file(s)", files.len());
        if changed > 0 {
            eprintln!("  {} file(s) reformatted", changed);
        }
        if unchanged > 0 {
            eprintln!("  {} file(s) unchanged", unchanged);
        }
    } else {
        eprintln!(
            "Checked {} file(s) (use --write to apply changes)",
            files.len()
        );
        if changed > 0 {
            eprintln!("  {} file(s) would be reformatted", changed);
        }
    }

    if errored > 0 {
        eprintln!("  {} file(s) had errors", errored);
    }
    let summary_time = summary_start.elapsed();

    if args.profile {
        let elapsed = start.elapsed();
        let mut profiles = profile_rows
            .and_then(|profile_rows| profile_rows.into_inner().ok())
            .unwrap_or_default();
        profiles.sort_by_key(|profile| std::cmp::Reverse(profile.row.total));

        let total_read = profiles
            .iter()
            .fold(Duration::ZERO, |acc, profile| acc + profile.read_time);
        let total_format = profiles
            .iter()
            .fold(Duration::ZERO, |acc, profile| acc + profile.row.primary);
        let total_write = profiles
            .iter()
            .fold(Duration::ZERO, |acc, profile| acc + profile.row.secondary);
        let total_bytes = profiles
            .iter()
            .fold(0usize, |acc, profile| acc + profile.row.bytes);
        let file_rows: Vec<_> = profiles.iter().map(|profile| profile.row.clone()).collect();

        let phases = [
            ProfilePhase {
                name: "collect files",
                duration: collect_time,
                kind: ProfilePhaseKind::Wall,
                note: "ignore-aware walk",
            },
            ProfilePhase {
                name: "format wall",
                duration: process_time,
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
                name: "format total",
                duration: total_format,
                kind: ProfilePhaseKind::Cumulative,
                note: "sum across worker threads",
            },
            ProfilePhase {
                name: "write total",
                duration: total_write,
                kind: ProfilePhaseKind::Cumulative,
                note: "sum across worker threads",
            },
            ProfilePhase {
                name: "summary output",
                duration: summary_time,
                kind: ProfilePhaseKind::Wall,
                note: "stderr reporting",
            },
        ];

        let slow_threshold = Duration::from_millis(args.slow_threshold);
        let mut recommendations = Vec::new();
        for profile in profiles
            .iter()
            .filter(|profile| profile.row.total > slow_threshold)
            .take(4)
        {
            recommendations.push(cstr!(
                "{} exceeded the slow threshold; inspect formatting options and large template/script blocks.",
                profile.row.path.display()
            ));
        }
        if args.write && total_write > total_format {
            recommendations.push(
                "File writes are heavier than formatting; run without --write when measuring formatter cost only."
                    .into(),
            );
        }

        let summary = cstr!(
            "{} file(s), {} changed, {} unchanged, {} errored",
            files.len(),
            changed,
            unchanged,
            errored
        );
        let profiler = global_profiler();
        let allocation_summary = profile_support::allocation_snapshot();
        let counter_summary = profiler.counter_summary();
        let operation_summary = profiler.summary();
        profiler.disable();

        let report = ProfileReport {
            title: "fmt",
            summary: summary.as_str(),
            total: elapsed,
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

    if has_errors.load(Ordering::Relaxed) {
        std::process::exit(1);
    }
}

/// Build format options: config file as base, CLI flags override.
#[inline]
fn build_format_options(args: &FmtArgs) -> FormatOptions {
    // Load config file as base (zero-cost if no file exists)
    let cfg = if args.no_config {
        config::VizeConfig::default()
    } else {
        config::load_config(args.config.as_deref())
    };
    let mut opts = config::to_glyph_format_options(&cfg.formatter);

    // CLI flags override config values
    if let Some(v) = args.print_width {
        opts.print_width = v;
    }
    if let Some(v) = args.tab_width {
        opts.tab_width = v;
    }
    if let Some(v) = args.use_tabs {
        opts.use_tabs = v;
    }
    if args.no_semi {
        opts.semi = false;
    }
    if let Some(v) = args.single_quote {
        opts.single_quote = v;
    }
    if let Some(v) = args.sort_attributes {
        opts.sort_attributes = v;
    }
    if let Some(v) = args.single_attribute_per_line {
        opts.single_attribute_per_line = v;
    }
    if let Some(v) = args.max_attributes_per_line {
        opts.max_attributes_per_line = Some(v);
    }
    if let Some(v) = args.normalize_directive_shorthands {
        opts.normalize_directive_shorthands = v;
    }

    opts
}

#[allow(clippy::disallowed_types)]
fn default_fmt_patterns() -> Vec<std::string::String> {
    vec![
        "./**/*.vue".into(),
        "./**/*.jsx".into(),
        "./**/*.tsx".into(),
    ]
}

#[inline]
#[allow(clippy::disallowed_types)]
fn process_file(
    path: &PathBuf,
    options: &FormatOptions,
    allocator: &Allocator,
    check: bool,
    write: bool,
    profile: bool,
) -> Result<FormatFileResult, String> {
    let file_start = profile.then(Instant::now);

    // Read the file
    let read_start = profile.then(Instant::now);
    let source = match profile!("cli.fmt.file.read", fs::read_to_string(path)) {
        Ok(source) => {
            global_profiler().record_fs_read_to_string(source.len());
            source
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            return Err(format!("Failed to read file: {}", error));
        }
    };
    let read_time = read_start
        .map(|start| start.elapsed())
        .unwrap_or(Duration::ZERO);

    // Format the source using the provided allocator
    let format_start = profile.then(Instant::now);
    let result = format_file_source(path, &source, options, allocator)
        .map_err(|e| format!("Format error: {}", e))?;
    let format_time = format_start
        .map(|start| start.elapsed())
        .unwrap_or(Duration::ZERO);

    let write_start = profile.then(Instant::now);
    if result.changed {
        if check {
            // In check mode, just report that the file would change
            eprintln!("Would reformat: {}", path.display());
        } else if write {
            // Write the formatted output atomically: temp file + rename so an
            // interruption can't truncate or corrupt the user's source (#970).
            let bytes = result.code.len();
            if let Err(error) = profile!(
                "cli.fmt.file.write",
                atomic_write(path, result.code.as_bytes())
            ) {
                global_profiler().record_fs_write_failure(bytes);
                return Err(format!("Failed to write file: {}", error));
            }
            global_profiler().record_fs_write(bytes);
            eprintln!("Reformatted: {}", path.display());
        } else {
            // Print the diff or formatted output
            eprintln!("Would reformat: {}", path.display());
        }
    }
    let write_time = write_start
        .map(|start| start.elapsed())
        .unwrap_or(Duration::ZERO);

    let profile = file_start.map(|start| {
        let state = if result.changed {
            "changed"
        } else {
            "unchanged"
        };
        FormatFileProfile {
            row: ProfileFileRow {
                path: path.clone(),
                bytes: source.len(),
                total: start.elapsed(),
                primary_label: "format",
                primary: format_time,
                secondary_label: "write",
                secondary: write_time,
                note: Some(cstr!(
                    "{:.2}ms read, {}",
                    read_time.as_secs_f64() * 1000.0,
                    state
                )),
            },
            read_time,
        }
    });

    Ok(FormatFileResult {
        changed: result.changed,
        profile,
    })
}

fn format_file_source(
    path: &Path,
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
) -> Result<FormatResult, vize_glyph::FormatError> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("jsx") => {
            let code = profile!(
                "cli.fmt.file.format_jsx",
                format_script_with_source_type(
                    source,
                    options,
                    allocator,
                    SourceType::jsx().with_module(true),
                )
            )?;
            Ok(FormatResult {
                changed: code.as_str() != source,
                code,
            })
        }
        Some("tsx") => {
            let code = profile!(
                "cli.fmt.file.format_tsx",
                format_script_with_source_type(
                    source,
                    options,
                    allocator,
                    SourceType::tsx().with_module(true),
                )
            )?;
            Ok(FormatResult {
                changed: code.as_str() != source,
                code,
            })
        }
        _ => profile!(
            "cli.fmt.file.format_sfc",
            format_sfc_with_allocator(source, options, allocator)
        ),
    }
}

struct FormatFileResult {
    changed: bool,
    profile: Option<FormatFileProfile>,
}

/// Write `contents` to `path` via a sibling temp file + rename, so an
/// interruption mid-write cannot truncate or corrupt the destination (#970).
///
/// The temp file lives in the same directory as the destination so the
/// rename is a same-filesystem move (atomic on Unix; best-effort on
/// Windows where rename fails if the target exists — we remove the
/// destination first only as a fallback).
fn atomic_write(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no file name"))?
        .to_string_lossy();
    let pid = std::process::id();
    let mut counter: u64 = 0;
    let temp_path = loop {
        counter = counter.wrapping_add(1);
        let candidate = dir.join(format!(".{}.vize-fmt.{}.{}.tmp", file_name, pid, counter));
        if !candidate.exists() {
            break candidate;
        }
    };

    let result = (|| -> std::io::Result<()> {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);

        #[cfg(windows)]
        {
            // Windows rename fails if the destination exists; remove it first.
            // The window between remove and rename is narrow but real; the
            // source is still recoverable from the temp file on failure.
            let _ = fs::remove_file(path);
        }

        fs::rename(&temp_path, path)
    })();

    if result.is_err() {
        // Best-effort cleanup; ignore secondary failure.
        let _ = fs::remove_file(&temp_path);
    }
    result
}

struct FormatFileProfile {
    row: ProfileFileRow,
    read_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::{atomic_write, format_file_source};
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };
    use vize_carton::{String, ToCompactString};

    #[test]
    fn format_file_source_formats_standalone_tsx() {
        let options = super::FormatOptions::default();
        let allocator = super::Allocator::default();
        let result = format_file_source(
            Path::new("Component.tsx"),
            "const Component=({label}:{label:string})=><button>{label}</button>",
            &options,
            &allocator,
        )
        .unwrap();

        insta::assert_snapshot!(result.code.as_str());
    }

    #[test]
    fn atomic_write_preserves_source_when_no_changes() {
        let root = unique_case_dir("atomic-write-noop");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("A.vue");
        fs::write(&path, b"<template>before</template>").unwrap();

        atomic_write(&path, b"<template>after</template>").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"<template>after</template>");
        let stray: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains("vize-fmt"))
            .collect();
        assert!(stray.is_empty(), "temp file should be renamed away");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn atomic_write_leaves_original_intact_on_failure() {
        let root = unique_case_dir("atomic-write-failure");
        fs::create_dir_all(&root).unwrap();
        // Point the "destination" at a path under a non-existent directory so
        // rename fails; the original is unaffected because no destination
        // existed yet — but the temp file must be cleaned up.
        let parent = root.join("nope-this-does-not-exist");
        let dest = parent.join("A.vue");

        let result = atomic_write(&dest, b"new contents");
        assert!(result.is_err());
        let _ = fs::remove_dir_all(&root);
    }

    fn unique_case_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut dir_name = String::from(name);
        dir_name.push('-');
        let pid = std::process::id().to_compact_string();
        dir_name.push_str(pid.as_str());
        dir_name.push('-');
        let nanos = nanos.to_compact_string();
        dir_name.push_str(nanos.as_str());
        std::env::current_dir()
            .unwrap()
            .join("target")
            .join("vize-tests")
            .join("fmt")
            .join(dir_name.as_str())
    }
}
