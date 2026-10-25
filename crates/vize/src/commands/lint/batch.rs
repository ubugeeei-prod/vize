//! Parallel source preparation followed by one checker publication per project.

use super::{
    LintArgs, LintRunAccumulator,
    collect::{is_plain_script_path, is_standalone_html_path},
    fix::{is_storybook_csf_path, jsx_lang_for_path},
};
use rayon::prelude::*;
use std::{
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};
use vize_carton::source_io as fs;
use vize_curator::profile::ProfileFileRow;
use vize_l0::{String, ToCompactString, cstr, profile, profiler::global_profiler};
use vize_patina::{LintResult, Linter, PreparedLint};

struct File {
    path: PathBuf,
    filename: String,
    source: String,
    times: Option<(Duration, Duration, Duration)>,
}

pub(super) fn run(
    files: &[PathBuf],
    config_indices: &[usize],
    linters: &[Linter],
    args: &LintArgs,
    retain_results: bool,
    profile_rows: Option<&Mutex<Vec<ProfileFileRow>>>,
) -> LintRunAccumulator {
    let (files, plans): (Vec<_>, Vec<_>) = files
        .par_iter()
        .zip(config_indices.par_iter())
        .filter_map(|(path, config_index)| {
            let linter = linters.get(*config_index)?;
            let start = args.profile.then(Instant::now);
            let source: String = match profile!("cli.lint.file.read", fs::read_to_string(path)) {
                Ok(source) => {
                    global_profiler().record_fs_read_to_string(source.len());
                    source.into()
                }
                Err(error) => {
                    global_profiler().record_fs_read_to_string_failure();
                    eprintln!("Failed to read {}: {}", path.display(), error);
                    return None;
                }
            };
            let read = start.map(|start| start.elapsed()).unwrap_or(Duration::ZERO);
            let filename = path.to_string_lossy().to_compact_string();
            let prepare_start = args.profile.then(Instant::now);
            let prepared = profile!("cli.lint.file.prepare", {
                if is_standalone_html_path(path) {
                    PreparedLint::complete(linter.lint_standalone_html(&source, &filename))
                } else if is_plain_script_path(path) {
                    linter.prepare_script(&source, &filename)
                } else if is_storybook_csf_path(path) {
                    PreparedLint::complete(LintResult {
                        filename: filename.clone(),
                        diagnostics: Vec::new(),
                        error_count: 0,
                        warning_count: 0,
                    })
                } else if let Some(lang) = jsx_lang_for_path(path) {
                    linter.prepare_jsx(&source, &filename, lang)
                } else {
                    linter.prepare_sfc(&source, &filename)
                }
            });
            let times = start
                .zip(prepare_start)
                .map(|(start, prepare)| (start.elapsed(), read, prepare.elapsed()));
            Some((
                File {
                    path: path.clone(),
                    filename,
                    source,
                    times,
                },
                (linter, prepared),
            ))
        })
        .unzip();
    let results = profile!(
        "cli.lint.projects.evaluate",
        Linter::lint_prepared_batch(plans)
    );
    files
        .into_par_iter()
        .zip(results.into_par_iter())
        .map(|(file, result)| {
            if let (Some((total, read, prepare)), Some(rows)) = (file.times, profile_rows)
                && let Ok(mut rows) = rows.lock()
            {
                rows.push(ProfileFileRow {
                    path: file.path.clone(),
                    bytes: file.source.len(),
                    total,
                    primary_label: "read",
                    primary: read,
                    secondary_label: "prepare",
                    secondary: prepare,
                    note: Some(cstr!(
                        "{} error(s), {} warning(s); checker in project spans",
                        result.error_count,
                        result.warning_count
                    )),
                });
            }
            (file.path, file.filename, file.source, result)
        })
        .fold(
            || LintRunAccumulator::new(retain_results),
            LintRunAccumulator::push,
        )
        .reduce(
            || LintRunAccumulator::new(retain_results),
            LintRunAccumulator::merge,
        )
}
