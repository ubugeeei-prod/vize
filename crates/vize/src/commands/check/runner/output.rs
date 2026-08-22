//! Aggregates diagnostics and declarations from one or more tsconfig programs.

use std::{collections::BTreeSet, path::Path, time::Duration, time::Instant};

use vize_carton::profiler::global_profiler;
use vize_carton::{String, cstr};

use super::{
    CheckArgs, JsonOutput, ProgramExecution,
    diagnostics::{
        emit_json_output, is_reported, is_suppressed_false_positive, render_diagnostics,
        save_virtual_ts_targets, write_profile_virtual_ts,
    },
};
use crate::commands::check::path_cache::CanonicalPathCache;

#[path = "output_declarations.rs"]
mod declarations;
#[path = "output_json.rs"]
mod json;
#[path = "output_profile.rs"]
mod profile;
use declarations::emit_declarations;
use json::emit_json;
use profile::print_profile;

#[allow(clippy::disallowed_types)]
type RenderedDiagnostics =
    std::collections::BTreeMap<std::string::String, Vec<std::string::String>>;

pub(super) fn save_virtual_ts_targets_or_exit<'a, C>(
    requested_paths: &[std::path::PathBuf],
    cwd: &Path,
    candidates: impl Fn() -> C,
    quiet: bool,
) where
    C: IntoIterator<Item = (&'a Path, &'a str)>,
{
    save_virtual_ts_targets(requested_paths, cwd, candidates, quiet).unwrap_or_else(|error| {
        let style = super::text_style::TextStyle::stderr();
        eprintln!("{} {error}", style.red("Error:"));
        std::process::exit(1);
    });
}

struct DeclarationSummary {
    files: BTreeSet<std::path::PathBuf>,
    directories: BTreeSet<std::path::PathBuf>,
    elapsed: Duration,
}

pub(super) fn finish_executions(
    args: &CheckArgs,
    cwd: &Path,
    start: Instant,
    collect_time: Duration,
    executions: Vec<ProgramExecution>,
    canonical_paths: &mut CanonicalPathCache,
) {
    let exit_code = report_executions(args, cwd, start, collect_time, executions, canonical_paths)
        .unwrap_or_else(|error| {
            let style = super::text_style::TextStyle::stderr();
            eprintln!("{} {error}", style.red("Error:"));
            1
        });
    // Machine-readable profile export: written after every reporting path
    // (including diagnostics failures) so the collected spans always land.
    args.profile_export.write_or_exit("check");
    if args.profile_export.is_requested() {
        global_profiler().disable();
    }
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

pub(super) fn exit_after_execution_error(
    executions: Vec<ProgramExecution>,
    error: vize_carton::String,
) -> ! {
    let style = super::text_style::TextStyle::stderr();
    eprintln!("{} {error}", style.red("Error:"));
    drop(executions);
    std::process::exit(1);
}

fn report_executions(
    args: &CheckArgs,
    cwd: &Path,
    start: Instant,
    collect_time: Duration,
    executions: Vec<ProgramExecution>,
    canonical_paths: &mut CanonicalPathCache,
) -> Result<i32, String> {
    let virtual_files = executions
        .iter()
        .flat_map(|execution| execution.checker.virtual_files())
        .collect::<Vec<_>>();
    if virtual_files.is_empty() {
        if args.format == "json" {
            emit_json_output(JsonOutput {
                files: Vec::new(),
                programs: Vec::new(),
                error_count: 0,
                warning_count: 0,
                file_count: 0,
                declarations: None,
            })?;
        } else {
            eprintln!("No files were registered for type checking");
        }
        return Ok(0);
    }

    if args.show_virtual_ts {
        if executions
            .iter()
            .any(|execution| execution.checker.shared_helpers_preamble().is_some())
        {
            eprintln!(
                "\n=== {} ===",
                vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME
            );
            eprintln!("{}", vize_canon::virtual_ts::SHARED_PREAMBLE_DTS);
        }
        for file in &virtual_files {
            eprintln!("\n=== {} ===", file.original_path.display());
            eprintln!("{}", file.content);
        }
    }

    if !args.save_virtual_ts_for.is_empty() {
        save_virtual_ts_targets(
            &args.save_virtual_ts_for,
            cwd,
            || {
                virtual_files
                    .iter()
                    .map(|file| (file.original_path.as_path(), file.content.as_str()))
            },
            args.quiet,
        )?;
    }

    let profile_artifact_start = Instant::now();
    if args.profile {
        write_profile_virtual_ts(&virtual_files);
    }
    let profile_artifact_time = profile_artifact_start.elapsed();

    let diagnostics_render_start = Instant::now();
    let mut reported_raw = Vec::new();
    for execution in &executions {
        for diagnostic in &execution.result.diagnostics {
            if is_reported(&execution.reported_files, &diagnostic.file, canonical_paths)
                && !is_suppressed_false_positive(diagnostic)
            {
                reported_raw.push(diagnostic.clone());
            }
        }
    }
    let diagnostics = render_diagnostics(&reported_raw, args.format != "json");
    let diagnostics_render_time = diagnostics_render_start.elapsed();
    let total_errors = reported_raw
        .iter()
        .filter(|diagnostic| diagnostic.severity == 1)
        .count();
    let total_warnings = reported_raw
        .iter()
        .filter(|diagnostic| diagnostic.severity == 2)
        .count();
    let emitted = emit_declarations(args, &executions, total_errors)?;
    let total_time = start.elapsed();
    let gen_time = executions.iter().map(|execution| execution.gen_time).sum();
    let check_time = executions
        .iter()
        .map(|execution| execution.check_time)
        .sum();

    if args.profile {
        print_profile(
            &executions,
            &virtual_files,
            total_errors,
            total_time,
            collect_time,
            gen_time,
            check_time,
            profile_artifact_time,
            diagnostics_render_time,
            emitted.as_ref(),
        );
    }

    if args.format == "json" {
        emit_json(
            args,
            cwd,
            &executions,
            &diagnostics,
            total_errors,
            total_warnings,
            emitted.as_ref(),
            canonical_paths,
        )?;
        if total_errors > 0 {
            return Ok(1);
        }
        return Ok(0);
    }

    Ok(print_text(
        args,
        &virtual_files,
        &diagnostics,
        total_errors,
        total_warnings,
        total_time,
        collect_time,
        gen_time,
        check_time,
        emitted.as_ref(),
    ))
}

#[allow(clippy::too_many_arguments)]
fn print_text(
    args: &CheckArgs,
    virtual_files: &[&vize_canon::VirtualFile],
    diagnostics: &RenderedDiagnostics,
    total_errors: usize,
    total_warnings: usize,
    total_time: Duration,
    collect_time: Duration,
    gen_time: Duration,
    check_time: Duration,
    emitted: Option<&DeclarationSummary>,
) -> i32 {
    let style = super::text_style::TextStyle::stdout();
    if !args.quiet {
        for (key, file_diagnostics) in diagnostics {
            if file_diagnostics.is_empty() {
                continue;
            }
            println!("\n{}", style.underline(key));
            for diagnostic in file_diagnostics {
                let diagnostic = if diagnostic.starts_with("error") {
                    style.red(diagnostic)
                } else {
                    style.yellow(diagnostic)
                };
                println!("  {diagnostic}");
            }
        }
    }

    let status = if total_errors > 0 {
        style.red("\u{2717}")
    } else {
        style.green("\u{2713}")
    };
    if let Some(summary) = emitted {
        println!(
            "\n{} Type checked {} files in {:.2?} (collect: {:.2?}, gen: {:.2?}, corsa: {:.2?}, dts: {:.2?})",
            status,
            virtual_files.len(),
            total_time,
            collect_time,
            gen_time,
            check_time,
            summary.elapsed
        );
    } else {
        println!(
            "\n{} Type checked {} files in {:.2?} (collect: {:.2?}, gen: {:.2?}, corsa: {:.2?})",
            status,
            virtual_files.len(),
            total_time,
            collect_time,
            gen_time,
            check_time
        );
    }
    if total_errors > 0 {
        println!("  {}", style.red(cstr!("{} error(s)", total_errors)));
    } else {
        println!("  {}", style.green("No type errors found!"));
    }
    if total_warnings > 0 {
        println!("  {}", style.yellow(cstr!("{} warning(s)", total_warnings)));
    }
    if let Some(summary) = emitted {
        let destinations = summary
            .directories
            .iter()
            .map(|path| cstr!("{}", path.display()))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "  {} to {}",
            style.green(cstr!("Emitted {} declaration file(s)", summary.files.len())),
            destinations
        );
    }

    if total_errors > 0 {
        return 1;
    }
    if let Some(max_warnings) = args.max_warnings
        && total_warnings > max_warnings
    {
        eprintln!("\nToo many warnings ({total_warnings} > max {max_warnings})");
        return 1;
    }
    0
}
