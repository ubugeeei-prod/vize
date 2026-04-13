//! Format command - High-performance Vue SFC formatting using vize_glyph

#![allow(clippy::disallowed_macros)]

use clap::Args;
use glob::{glob, MatchOptions, Pattern};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use vize_carton::cstr;
use vize_glyph::{format_sfc_with_allocator, Allocator, FormatOptions};

use crate::commands::profile::{
    print_profile_report, ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport,
};
use crate::config;

#[derive(Args)]
#[allow(clippy::disallowed_types)]
pub struct FmtArgs {
    /// Glob pattern(s) to match .vue files
    #[arg(default_value = "./**/*.vue")]
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

    /// Use single quotes instead of double quotes
    #[arg(long)]
    pub single_quote: Option<bool>,

    /// Print width (line length) for formatting
    #[arg(long)]
    pub print_width: Option<u32>,

    /// Number of spaces per indentation level
    #[arg(long)]
    pub tab_width: Option<u8>,

    /// Use tabs instead of spaces for indentation
    #[arg(long)]
    pub use_tabs: Option<bool>,

    /// Do not print semicolons at the ends of statements
    #[arg(long)]
    pub no_semi: bool,

    /// Sort HTML attributes in template
    #[arg(long)]
    pub sort_attributes: Option<bool>,

    /// Put each HTML attribute on its own line
    #[arg(long)]
    pub single_attribute_per_line: Option<bool>,

    /// Maximum number of attributes per line before wrapping
    #[arg(long)]
    pub max_attributes_per_line: Option<u32>,

    /// Normalize directive shorthands (v-bind: → :, v-on: → @, v-slot: → #)
    #[arg(long)]
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
    let options = build_format_options(&args);

    // Collect files to format
    let collect_start = Instant::now();
    let files: Vec<PathBuf> = collect_files(&args.patterns);
    let collect_time = collect_start.elapsed();

    if files.is_empty() {
        eprintln!("No .vue files found matching the patterns");
        return;
    }

    eprintln!("Found {} .vue file(s)", files.len());

    let has_errors = AtomicBool::new(false);
    let files_changed = AtomicUsize::new(0);
    let files_unchanged = AtomicUsize::new(0);
    let files_errored = AtomicUsize::new(0);
    let profile_rows = args.profile.then(|| Mutex::new(Vec::new()));

    // Process files in parallel, each thread gets its own allocator for maximum performance
    let process_start = Instant::now();
    files.par_iter().for_each(|path| {
        // Create per-thread allocator with estimated capacity
        let allocator = Allocator::with_capacity(64 * 1024); // 64KB initial capacity

        match process_file(
            path,
            &options,
            &allocator,
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

                if let (Some(profile), Some(profile_rows)) = (result.profile, profile_rows.as_ref())
                {
                    if let Ok(mut rows) = profile_rows.lock() {
                        rows.push(profile);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error formatting {}: {}", path.display(), err);
                files_errored.fetch_add(1, Ordering::Relaxed);
                has_errors.store(true, Ordering::Relaxed);
            }
        }
    });
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
        profiles.sort_by(|left, right| right.row.total.cmp(&left.row.total));

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
        let report = ProfileReport {
            title: "fmt",
            summary: summary.as_str(),
            total: elapsed,
            phases: &phases,
            files: &file_rows,
            slow_threshold,
            throughput_bytes: Some(total_bytes),
            operations: None,
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
    let cfg = config::load_config(args.config.as_deref());
    let mut opts = cfg.fmt;

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
fn collect_files(patterns: &[std::string::String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    for pattern in patterns {
        let normalized = normalize_fmt_pattern(pattern);
        if should_walk_with_gitignore(&normalized) {
            if let Some(pattern) = FmtPattern::new(&normalized, &cwd) {
                collect_walked_files(&pattern, &mut files);
            }
        } else if contains_glob_char(&normalized) {
            if let Ok(paths) = glob(&normalized) {
                for path in paths.flatten() {
                    if path.extension().is_some_and(|ext| ext == "vue") {
                        files.push(path);
                    }
                }
            }
        } else {
            let path = PathBuf::from(&normalized);
            if path.extension().is_some_and(|ext| ext == "vue") && path.is_file() {
                files.push(path);
            }
        }
    }

    // Remove duplicates
    files.sort();
    files.dedup();

    files
}

fn collect_walked_files(pattern: &FmtPattern, files: &mut Vec<PathBuf>) {
    // Use ignore crate to walk directories respecting .gitignore
    let walker = WalkBuilder::new(".")
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "vue") && pattern.matches(path) {
            files.push(path.to_path_buf());
        }
    }
}

#[inline]
fn should_walk_with_gitignore(pattern: &str) -> bool {
    matches!(pattern, "**/*.vue" | "./**/*.vue")
}

struct FmtPattern {
    pattern: Pattern,
    cwd: PathBuf,
    absolute: bool,
}

impl FmtPattern {
    fn new(pattern: &str, cwd: &Path) -> Option<Self> {
        let normalized = normalize_fmt_pattern(pattern);
        let absolute = Path::new(&normalized).is_absolute();
        Pattern::new(&normalized).ok().map(|pattern| Self {
            pattern,
            cwd: cwd.to_path_buf(),
            absolute,
        })
    }

    fn matches(&self, path: &Path) -> bool {
        let candidate = if self.absolute {
            let relative = path.strip_prefix(".").unwrap_or(path);
            let absolute = if relative.is_absolute() {
                relative.to_path_buf()
            } else {
                self.cwd.join(relative)
            };
            normalize_path(&absolute)
        } else {
            normalize_path(path.strip_prefix(".").unwrap_or(path))
        };

        self.pattern
            .matches_with(candidate.as_str(), fmt_glob_match_options())
    }
}

fn normalize_fmt_pattern(pattern: &str) -> vize_carton::String {
    let mut normalized: vize_carton::String = pattern.replace('\\', "/").into();
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.into();
    }

    if normalized.is_empty() || normalized == "." {
        return "**/*.vue".into();
    }

    if !contains_glob_char(&normalized) && Path::new(&normalized).is_dir() {
        if !normalized.ends_with('/') {
            normalized.push('/');
        }
        normalized.push_str("**/*.vue");
    }

    normalized
}

#[inline]
fn normalize_path(path: &Path) -> vize_carton::String {
    path.to_string_lossy().replace('\\', "/").into()
}

#[inline]
fn contains_glob_char(pattern: &str) -> bool {
    pattern.contains(['*', '?', '['])
}

#[inline]
fn fmt_glob_match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: !cfg!(windows),
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
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
    let source = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let read_time = read_start
        .map(|start| start.elapsed())
        .unwrap_or(Duration::ZERO);

    // Format the source using the provided allocator
    let format_start = profile.then(Instant::now);
    let result = format_sfc_with_allocator(&source, options, allocator)
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
            // Write the formatted output
            fs::write(path, &result.code).map_err(|e| format!("Failed to write file: {}", e))?;
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

struct FormatFileResult {
    changed: bool,
    profile: Option<FormatFileProfile>,
}

struct FormatFileProfile {
    row: ProfileFileRow,
    read_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::{collect_files, FmtPattern};
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };
    use vize_carton::{String, ToCompactString};

    #[test]
    fn absolute_glob_only_matches_requested_directory() {
        let root = unique_case_dir("absolute-glob");
        let input_dir = root.join("bench-input");
        let other_dir = root.join("other");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&other_dir).unwrap();
        fs::write(input_dir.join("A.vue"), "<template><div/></template>").unwrap();
        fs::write(other_dir.join("B.vue"), "<template><div/></template>").unwrap();

        let pattern = input_dir.join("*.vue").to_string_lossy().into_owned();
        let files = collect_files(&[pattern]);
        let _ = fs::remove_dir_all(&root);

        assert_eq!(files, vec![input_dir.join("A.vue")]);
    }

    #[test]
    fn relative_glob_does_not_match_every_vue_file() {
        let cwd = std::env::current_dir().unwrap();
        let pattern = FmtPattern::new("bench/__in__/*.vue", &cwd).unwrap();

        assert!(pattern.matches(Path::new("./bench/__in__/Component0000.vue")));
        assert!(!pattern.matches(Path::new("./examples/cli/src/App.vue")));
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
            .join("__agent_only")
            .join("tests")
            .join("fmt")
            .join(dir_name.as_str())
    }
}
