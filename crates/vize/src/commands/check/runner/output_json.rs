use std::path::Path;

use vize_s0::{FxHashSet, String};

use super::super::{
    CheckArgs, JsonFileResult, JsonOutput, JsonProgramResult, ProgramExecution,
    diagnostics::{emit_json_output, is_reported},
    display_path,
};
use super::{DeclarationSummary, RenderedDiagnostics};
use crate::commands::check::path_cache::CanonicalPathCache;

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_json(
    args: &CheckArgs,
    cwd: &Path,
    executions: &[ProgramExecution],
    diagnostics: &RenderedDiagnostics,
    total_errors: usize,
    total_warnings: usize,
    emitted: Option<&DeclarationSummary>,
    canonical_paths: &mut CanonicalPathCache,
) -> Result<(), vize_s0::String> {
    let mut files_json = executions
        .iter()
        .flat_map(|execution| {
            execution
                .checker
                .virtual_files()
                .into_iter()
                .filter(|file| {
                    is_reported(
                        &execution.reported_files,
                        &file.original_path,
                        canonical_paths,
                    )
                })
                .map(|file| {
                    let key = file.original_path.to_string_lossy().into_owned();
                    JsonFileResult {
                        file: display_path(cwd, &file.original_path).into(),
                        virtual_ts: args.show_virtual_ts.then(|| file.content.clone().into()),
                        diagnostics: diagnostics.get(key.as_str()).cloned().unwrap_or_default(),
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    files_json.extend(executions.iter().flat_map(|execution| {
        execution.reported_files.iter().map(|path| {
            let key = path.to_string_lossy();
            JsonFileResult {
                file: display_path(cwd, path).into(),
                virtual_ts: None,
                diagnostics: diagnostics.get(key.as_ref()).cloned().unwrap_or_default(),
            }
        })
    }));
    files_json.sort_by(|left, right| left.file.cmp(&right.file));
    files_json.dedup_by(|left, right| left.file == right.file);
    let reported_file_count = files_json.len();
    let programs_json = executions
        .iter()
        .map(|execution| {
            let mut files = execution
                .input_files
                .iter()
                .map(|path| display_path(cwd, path).into())
                .collect::<Vec<_>>();
            files.sort();
            files.dedup();
            let mut root = display_path(cwd, &execution.program_root);
            if root.is_empty() {
                root = ".".into();
            }
            Ok(JsonProgramResult {
                root: root.into(),
                tsconfig: execution
                    .tsconfig_path
                    .as_ref()
                    .map(|path| display_path(cwd, path).into()),
                compiler_options: execution
                    .tsconfig_path
                    .as_ref()
                    .map(|path| vize_canon::snapshot_tsconfig_compiler_options(cwd, path))
                    .transpose()
                    .map_err(|error| {
                        vize_s0::cstr!(
                            "Failed to snapshot compiler options for JSON output: {}",
                            error
                        )
                    })?,
                files,
            })
        })
        .collect::<Result<Vec<_>, vize_s0::String>>()?;

    let reported_keys = executions
        .iter()
        .flat_map(|execution| execution.reported_files.iter())
        .map(|path| String::from(path.to_string_lossy()))
        .collect::<FxHashSet<_>>();
    files_json.extend(
        diagnostics
            .iter()
            .filter(|(key, values)| !values.is_empty() && !reported_keys.contains(key.as_str()))
            .map(|(key, values)| JsonFileResult {
                file: display_path(cwd, Path::new(key)).into(),
                virtual_ts: None,
                diagnostics: values.clone(),
            }),
    );

    emit_json_output(JsonOutput {
        files: files_json,
        programs: programs_json,
        error_count: total_errors,
        warning_count: total_warnings,
        file_count: reported_file_count,
        declarations: emitted.map(|summary| {
            summary
                .files
                .iter()
                .map(|path| display_path(cwd, path).into())
                .collect()
        }),
    })
}
