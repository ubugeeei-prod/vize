//! Diagnostic reporting, JSON serialization, and profile artifacts for the
//! `check` runner.

use std::{
    fs,
    path::{Path, PathBuf},
};

use vize_s0::{String as CompactString, cstr, profile, profiler::global_profiler};

use crate::commands::check::reporting::JsonOutput;

mod source_context;
mod source_filter;
mod suppressions;

pub(super) use source_filter::is_reported;
pub(super) use suppressions::is_suppressed_false_positive;

pub(super) fn emit_json_output(json_output: JsonOutput) -> Result<(), CompactString> {
    let output = serde_json::to_string_pretty(&json_output)
        .map_err(|error| cstr!("Failed to serialize check output: {error}"))?;
    println!("{output}");
    Ok(())
}

#[allow(clippy::disallowed_types)]
pub(super) fn render_diagnostics(
    diagnostics: &[vize_canon::BatchDiagnostic],
    include_source_context: bool,
) -> std::collections::BTreeMap<std::string::String, Vec<std::string::String>> {
    let mut grouped = std::collections::BTreeMap::<
        std::string::String,
        Vec<(u32, u32, std::string::String)>,
    >::new();
    let mut source_context = source_context::SourceContextCache::default();

    for diagnostic in diagnostics {
        let severity = match diagnostic.severity {
            1 => "error",
            2 => "warning",
            3 => "info",
            _ => "hint",
        };
        let code = diagnostic
            .code
            .map(|code| cstr!(" [TS{}]", code))
            .unwrap_or_default();
        let message = if include_source_context {
            source_context
                .render(diagnostic)
                .map(|context| cstr!("{} (source: {context})", diagnostic.message))
                .unwrap_or_else(|| diagnostic.message.clone())
        } else {
            diagnostic.message.clone()
        };
        let rendered = cstr!(
            "{}:{}:{}{} {}",
            severity,
            diagnostic.line + 1,
            diagnostic.column + 1,
            code,
            message
        )
        .into();
        grouped
            .entry(diagnostic.file.to_string_lossy().into_owned())
            .or_default()
            .push((diagnostic.line, diagnostic.column, rendered));
    }

    grouped
        .into_iter()
        .map(|(file, mut diagnostics)| {
            diagnostics.sort_by(|left, right| {
                left.0
                    .cmp(&right.0)
                    .then_with(|| left.1.cmp(&right.1))
                    .then_with(|| left.2.cmp(&right.2))
            });
            let rendered = diagnostics
                .into_iter()
                .map(|(_, _, rendered)| rendered)
                .collect();
            (file, rendered)
        })
        .collect()
}

/// Whether a `--save-virtual-ts-for` target names the shared ambient helpers
/// file (`__vize_helpers.d.ts`) rather than a per-`.vue` virtual module.
///
/// Matched purely on the file name so the flag accepts the bare
/// `__vize_helpers.d.ts`, a relative path, or an absolute path interchangeably.
fn is_shared_helpers_target(requested_path: &Path) -> bool {
    requested_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME)
}

/// Save the generated virtual TypeScript for a single `--save-virtual-ts-for`
/// target.
///
/// A target that names the shared helpers file (`__vize_helpers.d.ts`) writes
/// the program-wide helpers preamble ([`SHARED_PREAMBLE_DTS`]) verbatim at the
/// requested location. Every other target resolves to a generated per-`.vue`
/// virtual module and is written next to its source as `<file>.virtual.ts`,
/// exactly as before.
///
/// [`SHARED_PREAMBLE_DTS`]: vize_canon::virtual_ts::SHARED_PREAMBLE_DTS
pub(super) fn save_virtual_ts_for_path<'a>(
    requested_path: &Path,
    cwd: &Path,
    candidates: impl IntoIterator<Item = (&'a Path, &'a str)>,
) -> Result<PathBuf, CompactString> {
    if is_shared_helpers_target(requested_path) {
        return save_shared_helpers_virtual_ts(requested_path, cwd);
    }

    let requested_path = normalize_requested_virtual_ts_path(cwd, requested_path);
    let Some((original_path, content)) = candidates
        .into_iter()
        .find(|(candidate_path, _)| paths_refer_to_same_file(candidate_path, &requested_path))
    else {
        return Err(cstr!(
            "Virtual TS for {} was not generated",
            requested_path.display()
        ));
    };

    let target = virtual_ts_save_path(original_path)?;
    write_virtual_ts(&target, content)
}

/// Save several `--save-virtual-ts-for` targets in one run, writing each one in
/// turn and reporting every saved path. A failure on any target aborts the run.
pub(super) fn save_virtual_ts_targets<'a, C>(
    requested_paths: &[PathBuf],
    cwd: &Path,
    candidates: impl Fn() -> C,
    quiet: bool,
) -> Result<(), CompactString>
where
    C: IntoIterator<Item = (&'a Path, &'a str)>,
{
    for requested_path in requested_paths {
        let target = save_virtual_ts_for_path(requested_path, cwd, candidates())?;
        if !quiet {
            eprintln!("Saved Virtual TS to {}", target.display());
        }
    }
    Ok(())
}

/// Write the shared ambient helpers preamble to the requested location.
fn save_shared_helpers_virtual_ts(
    requested_path: &Path,
    cwd: &Path,
) -> Result<PathBuf, CompactString> {
    let target = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        cwd.join(requested_path)
    };
    write_virtual_ts(&target, vize_canon::virtual_ts::SHARED_PREAMBLE_DTS)
}

fn write_virtual_ts(target: &Path, content: &str) -> Result<PathBuf, CompactString> {
    let bytes = content.len();
    match profile!(
        "cli.check.save_virtual_ts.write",
        fs::write(target, content)
    ) {
        Ok(()) => {
            global_profiler().record_fs_write(bytes);
            Ok(target.to_path_buf())
        }
        Err(error) => {
            global_profiler().record_fs_write_failure(bytes);
            Err(cstr!("Failed to write {}: {}", target.display(), error))
        }
    }
}

fn virtual_ts_save_path(original_path: &Path) -> Result<PathBuf, CompactString> {
    let file_name = original_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            cstr!(
                "Cannot derive Virtual TS output path for {}",
                original_path.display()
            )
        })?;
    let mut target = original_path.to_path_buf();
    target.set_file_name(cstr!("{file_name}.virtual.ts").as_str());
    Ok(target)
}

fn normalize_requested_virtual_ts_path(cwd: &Path, path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    vize_s0::path::canonicalize_non_verbatim(&absolute)
}

fn paths_refer_to_same_file(candidate_path: &Path, requested_path: &Path) -> bool {
    let candidate_path = vize_s0::path::canonicalize_non_verbatim(candidate_path);
    candidate_path == requested_path
}

pub(super) fn write_profile_virtual_ts(files: &[&vize_canon::VirtualFile]) {
    let profile_dir = PathBuf::from("node_modules/.vize/check-profile");
    if profile_dir.exists() {
        match profile!(
            "cli.check.profile_artifact.remove_dir_all",
            fs::remove_dir_all(&profile_dir)
        ) {
            Ok(()) => global_profiler().record_fs_remove_dir_all(),
            Err(error) => {
                global_profiler().record_fs_remove_dir_all_failure();
                eprintln!(
                    "Failed to clean profile directory {}: {}",
                    profile_dir.display(),
                    error
                );
                return;
            }
        }
    }

    match profile!(
        "cli.check.profile_artifact.create_dir_all",
        fs::create_dir_all(&profile_dir)
    ) {
        Ok(()) => global_profiler().record_fs_create_dir_all(),
        Err(error) => {
            global_profiler().record_fs_create_dir_all_failure();
            eprintln!("Failed to create profile directory: {}", error);
            return;
        }
    }

    for file in files {
        let file_name = file
            .original_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| cstr!("{name}.ts"))
            .unwrap_or_else(|| "unknown.ts".into());
        let target = profile_dir.join(file_name.as_str());
        let bytes = file.content.len();
        match profile!(
            "cli.check.profile_artifact.write",
            fs::write(&target, &file.content)
        ) {
            Ok(()) => global_profiler().record_fs_write(bytes),
            Err(error) => {
                global_profiler().record_fs_write_failure(bytes);
                eprintln!("Failed to write {}: {}", target.display(), error);
            }
        }
    }

    eprintln!(
        "\x1b[33mProfile:\x1b[0m Virtual TS files written to {}",
        profile_dir.display()
    );
}

#[cfg(test)]
#[path = "diagnostics/tests.rs"]
mod tests;
