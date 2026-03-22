//! NAPI bindings for Vue SFC linting.
//!
//! Provides the `lint` function for linting Vue SFC files
//! with native multithreading and .gitignore awareness.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use glob::glob;
use napi::bindgen_prelude::Result;
use napi_derive::napi;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};
use vize_carton::append;

/// Lint options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct LintOptionsNapi {
    /// Output format: "text" or "json"
    pub format: Option<String>,
    /// Maximum number of warnings before failing
    pub max_warnings: Option<u32>,
    /// Quiet mode - only show summary
    pub quiet: Option<bool>,
    /// Automatically fix problems (not yet implemented)
    pub fix: Option<bool>,
    /// Help display level: "full", "short", "none"
    pub help_level: Option<String>,
    /// Lint preset: "happy-path", "opinionated", "essential", or "nuxt"
    pub preset: Option<String>,
}

/// Lint result for NAPI
#[napi(object)]
pub struct LintResultNapi {
    /// Formatted output string
    pub output: String,
    /// Total number of errors
    pub error_count: u32,
    /// Total number of warnings
    pub warning_count: u32,
    /// Number of files linted
    pub file_count: u32,
    /// Time in milliseconds
    pub time_ms: f64,
}

/// Lint Vue SFC files matching patterns (native multithreading, .gitignore-aware)
#[napi]
pub fn lint(patterns: Vec<String>, options: Option<LintOptionsNapi>) -> Result<LintResultNapi> {
    use ignore::Walk;
    use std::time::Instant;
    use vize_patina::{
        format_results, format_summary, HelpLevel, LintPreset, Linter, OutputFormat,
    };

    let opts = options.unwrap_or_default();
    let start = Instant::now();

    // Collect .vue files using glob patterns or directory walking
    let files: Vec<std::path::PathBuf> = patterns
        .iter()
        .flat_map(|pattern| {
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

    if files.is_empty() {
        return Ok(LintResultNapi {
            output: format!("No .vue files found matching patterns: {:?}", patterns),
            error_count: 0,
            warning_count: 0,
            file_count: 0,
            time_ms: start.elapsed().as_secs_f64() * 1000.0,
        });
    }

    let help_level = match opts.help_level.as_deref() {
        Some("none") => HelpLevel::None,
        Some("short") => HelpLevel::Short,
        _ => HelpLevel::Full,
    };
    let preset = opts
        .preset
        .as_deref()
        .and_then(LintPreset::parse)
        .unwrap_or_default();
    let linter = Linter::with_preset(preset).with_help_level(help_level);
    let error_count = AtomicUsize::new(0);
    let warning_count = AtomicUsize::new(0);

    // Lint all files in parallel and collect results
    let results: Vec<_> = files
        .par_iter()
        .filter_map(|path| {
            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let filename = path.to_string_lossy().to_string();
            let result = linter.lint_sfc(&source, &filename);

            error_count.fetch_add(result.error_count, Ordering::Relaxed);
            warning_count.fetch_add(result.warning_count, Ordering::Relaxed);

            Some((filename, source, result))
        })
        .collect();

    let total_errors = error_count.load(Ordering::Relaxed);
    let total_warnings = warning_count.load(Ordering::Relaxed);

    let format = match opts.format.as_deref() {
        Some("json") => OutputFormat::Json,
        _ => OutputFormat::Text,
    };

    let quiet = opts.quiet.unwrap_or(false);

    // Format output
    let mut output = vize_carton::CompactString::default();
    if !quiet || total_errors > 0 || total_warnings > 0 {
        let lint_results: Vec<_> = results.iter().map(|(_, _, r)| r).cloned().collect();
        let sources: Vec<_> = results
            .iter()
            .map(|(f, s, _)| {
                (
                    vize_carton::CompactString::from(f.as_str()),
                    vize_carton::CompactString::from(s.as_str()),
                )
            })
            .collect();

        let formatted = format_results(&lint_results, &sources, format);
        if !formatted.trim().is_empty() {
            output.push_str(&formatted);
        }
    }

    let elapsed = start.elapsed();
    if format == OutputFormat::Text {
        append!(
            output,
            "\n{}\n",
            format_summary(total_errors, total_warnings, files.len())
        );
        append!(output, "Linted {} files in {:.4?}", files.len(), elapsed);
    }

    Ok(LintResultNapi {
        output: output.into(),
        error_count: total_errors as u32,
        warning_count: total_warnings as u32,
        file_count: files.len() as u32,
        time_ms: elapsed.as_secs_f64() * 1000.0,
    })
}
