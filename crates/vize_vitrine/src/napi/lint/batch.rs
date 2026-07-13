//! Persistent-graph batch lint implementation for NAPI.

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use glob::glob;
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use vize_atlas::Shared;
use vize_carton::append;
use vize_patina::{HelpLevel, OutputFormat, format_results, format_summary};

use super::lint_options::{
    LintOptionsNapi, LintResultNapi, configure_type_aware_lint, create_patina_linter,
    patina_preset_from_option,
};
use crate::{
    lint_artifact::PatinaLintGraph,
    napi::lint_fix::{LintFileInput, is_lintable_extension, lint_file_with_optional_fix},
};

/// Lint Vue SFC and standalone HTML files in one persistent Atlas compilation.
#[napi]
pub fn lint(patterns: Vec<String>, options: Option<LintOptionsNapi>) -> Result<LintResultNapi> {
    use ignore::Walk;

    let opts = options.unwrap_or_default();
    let start = Instant::now();
    let files: Vec<std::path::PathBuf> = patterns
        .iter()
        .flat_map(|pattern| {
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                glob(pattern)
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|result| result.ok())
                    .filter(|path| {
                        path.extension()
                            .and_then(|extension| extension.to_str())
                            .is_some_and(is_lintable_extension)
                            && !path
                                .components()
                                .any(|component| component.as_os_str() == "node_modules")
                    })
                    .collect::<Vec<_>>()
            } else {
                Walk::new(pattern)
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry
                            .path()
                            .extension()
                            .and_then(|extension| extension.to_str())
                            .is_some_and(is_lintable_extension)
                    })
                    .map(|entry| entry.path().to_path_buf())
                    .collect::<Vec<_>>()
            }
        })
        .collect();

    if files.is_empty() {
        return Ok(LintResultNapi {
            output: format!(
                "No .vue or .html files found matching patterns: {:?}",
                patterns
            ),
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
    let preset = patina_preset_from_option(opts.preset.as_deref());
    let linter = Shared::new(configure_type_aware_lint(
        create_patina_linter(preset).with_help_level(help_level),
        opts.type_aware,
        opts.corsa_path,
    ));
    let inputs: Vec<_> = files
        .iter()
        .filter_map(|path| LintFileInput::read(path))
        .collect();
    let graph = PatinaLintGraph::new(linter, inputs.iter().map(LintFileInput::graph_source))
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;
    let error_count = AtomicUsize::new(0);
    let warning_count = AtomicUsize::new(0);
    let should_fix = opts.fix.unwrap_or(false);
    let results: Vec<_> = inputs
        .par_iter()
        .enumerate()
        .filter_map(|(index, input)| {
            let item = lint_file_with_optional_fix(&graph, index, input, should_fix)?;
            error_count.fetch_add(item.2.error_count, Ordering::Relaxed);
            warning_count.fetch_add(item.2.warning_count, Ordering::Relaxed);
            Some(item)
        })
        .collect();

    let total_errors = error_count.load(Ordering::Relaxed);
    let total_warnings = warning_count.load(Ordering::Relaxed);
    let format = opts
        .format
        .as_deref()
        .and_then(OutputFormat::parse)
        .unwrap_or(OutputFormat::Text);
    let quiet = opts.quiet.unwrap_or(false);
    let mut output = vize_carton::CompactString::default();
    if format.renders_details_when_quiet() || !quiet || total_errors > 0 || total_warnings > 0 {
        let lint_results: Vec<_> = results
            .iter()
            .map(|(_, _, result)| result)
            .cloned()
            .collect();
        let sources: Vec<_> = results
            .iter()
            .map(|(filename, source, _)| (filename.clone(), source.clone()))
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
