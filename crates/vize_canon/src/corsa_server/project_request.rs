//! Whole-project socket requests backed by one retained batch checker.

use std::path::{Path, PathBuf};

use vize_carton::{String, cstr};

use super::{
    CorsaServer, JsonRpcError, JsonRpcResponse, ProjectCheckFile, ProjectCheckParams,
    ProjectCheckResult, ProjectDiagnostic,
};
use crate::batch::{BatchTypeChecker, BatchTypeCheckerOptions, TypeChecker};

pub(super) struct ProjectCheckSession {
    files: Vec<ProjectCheckFile>,
    tsconfig: Option<ProjectCheckFile>,
    pub(super) checker: BatchTypeChecker,
}

impl CorsaServer {
    pub(super) fn handle_project_check(
        &mut self,
        id: Option<u64>,
        params: serde_json::Value,
    ) -> JsonRpcResponse {
        let params: ProjectCheckParams = match serde_json::from_value(params) {
            Ok(params) => params,
            Err(error) => return rpc_error(id, -32602, cstr!("Invalid params: {error}")),
        };
        match self.check_project(params) {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(serde_json::to_value(result).unwrap_or(serde_json::Value::Null)),
                error: None,
            },
            Err(error) => rpc_error(id, -32000, error),
        }
    }

    fn check_project(&mut self, params: ProjectCheckParams) -> Result<ProjectCheckResult, String> {
        let paths = params
            .files
            .iter()
            .map(|file| request_path(&file.uri, &self.working_dir()))
            .collect::<Result<Vec<_>, _>>()?;
        if paths.is_empty() {
            return Ok(ProjectCheckResult {
                diagnostics: Vec::new(),
                error_count: 0,
                warning_count: 0,
                file_count: 0,
            });
        }

        let same_program = self.project_session.as_ref().is_some_and(|session| {
            session.tsconfig == params.tsconfig
                && session.files.len() == params.files.len()
                && session
                    .files
                    .iter()
                    .zip(&params.files)
                    .all(|(left, right)| left.uri == right.uri)
        });

        let result = if same_program {
            let Some(session) = self.project_session.as_mut() else {
                return Err("Project session disappeared before reuse".into());
            };
            let changed = session
                .files
                .iter()
                .zip(&params.files)
                .zip(&paths)
                .filter_map(|((old, new), path)| (old != new).then(|| path.clone()))
                .collect::<Vec<_>>();
            let result = if changed.is_empty() {
                session.checker.check_persistent_project()
            } else {
                session.checker.check_incremental(&changed)
            };
            if result.is_ok() {
                session.files = params.files;
            }
            result
        } else {
            let tsconfig = params
                .tsconfig
                .as_ref()
                .map(|file| PathBuf::from(file.uri.as_str()));
            let mut checker = BatchTypeChecker::with_options_and_corsa_path(
                &self.working_dir(),
                BatchTypeCheckerOptions {
                    tsconfig_path: tsconfig,
                    ..Default::default()
                },
                self.config.corsa_path.as_deref().map(Path::new),
            )
            .map_err(|error| cstr!("{error}"))?;
            checker
                .scan_paths(&paths)
                .map_err(|error| cstr!("{error}"))?;
            checker.set_diagnostic_paths(paths.iter().map(PathBuf::as_path));
            let result = checker.check_persistent_project();
            if result.is_ok() {
                self.project_session = Some(ProjectCheckSession {
                    files: params.files,
                    tsconfig: params.tsconfig,
                    checker,
                });
            }
            result
        }
        .map_err(|error| cstr!("{error}"))?;

        let error_count = result.error_count();
        let warning_count = result.warning_count();
        let diagnostics = result
            .diagnostics
            .into_iter()
            .map(|diagnostic| ProjectDiagnostic {
                file: diagnostic.file.to_string_lossy().into_owned().into(),
                message: diagnostic.message,
                severity: match diagnostic.severity {
                    1 => "error",
                    2 => "warning",
                    3 => "info",
                    _ => "hint",
                }
                .into(),
                line: diagnostic.line + 1,
                column: diagnostic.column + 1,
                code: diagnostic.code.map(|code| cstr!("TS{code}")),
            })
            .collect();
        Ok(ProjectCheckResult {
            diagnostics,
            error_count,
            warning_count,
            file_count: paths.len(),
        })
    }
}

fn request_path(uri: &str, working_dir: &Path) -> Result<PathBuf, String> {
    let path = super::sfc_semantics::uri_to_path(uri, working_dir)
        .ok_or_else(|| cstr!("Unsupported check path: {uri}"))?;
    Ok(vize_carton::path::canonicalize_non_verbatim(&path))
}

fn rpc_error(id: Option<u64>, code: i64, message: String) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data: None,
        }),
    }
}
