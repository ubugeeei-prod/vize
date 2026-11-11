//! Corsa-backed batch executor.
//!
//! This module materializes the virtual project, asks Corsa for diagnostics,
//! and maps those diagnostics back to the original source positions.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use super::declaration_path::is_declaration_file;
use super::error::{CorsaError, CorsaNotFoundError, CorsaResult};
use super::import_rewriter::ImportRewriter;
use super::materialize_lock::MaterializeLock;
use super::type_checker::{
    DeclarationEmitOptions, DeclarationEmitResult, DeclarationOutput, IncrementalCheckMetrics,
    TypeCheckResult,
};
use super::virtual_project::VirtualProject;
use oxc_span::SourceType;
use vize_carton::{
    String,
    corsa_resolver::{CorsaResolveError, CorsaResolveRequest, resolve_batch_corsa_executable},
    cstr, profile,
};

const DECLARATION_HELPERS_FILE: &str = crate::virtual_ts::SHARED_PREAMBLE_FILE_NAME;

mod cli;
mod declaration_helpers;
mod declaration_maps;
mod declaration_outputs;
use declaration_outputs::rewrite_declaration_outputs;
mod diagnostics;
mod fallback;
mod option_probe;
mod session;

use cli::{
    CheckFailure, auto_server_count, check_with_cli, check_with_cli_classified,
    check_with_cli_sharded,
};
use diagnostics::dedup_diagnostics;
use fallback::{FallbackStep, warn_fallback};
use session::IncrementalSessionState;
#[cfg(test)]
use session::collect_virtual_file_uris;

/// Batch executor backed by `corsa`'s project-session diagnostics API.
pub struct CorsaExecutor {
    /// Path to the resolved Corsa executable.
    corsa_path: PathBuf,
    checker_count: Option<usize>,
    program_concurrency: usize,
    incremental_session: Mutex<IncrementalSessionState>,
}

impl CorsaExecutor {
    pub fn new(root: &Path) -> Result<Self, CorsaNotFoundError> {
        Self::with_corsa_path(root, None)
    }

    /// Create a new executor with an optional explicit Corsa executable path.
    pub fn with_corsa_path(
        project_root: &Path,
        corsa_path: Option<&Path>,
    ) -> Result<Self, CorsaNotFoundError> {
        let request = CorsaResolveRequest {
            explicit_path: corsa_path,
            project_root: Some(project_root),
        };

        match resolve_batch_corsa_executable(request) {
            Ok(corsa_path) => Ok(Self {
                corsa_path,
                checker_count: None,
                program_concurrency: 1,
                incremental_session: Mutex::new(IncrementalSessionState::default()),
            }),
            Err(CorsaResolveError::ExplicitNotFound { path, .. }) => {
                Err(CorsaNotFoundError::new_explicit(project_root, &path))
            }
            Err(CorsaResolveError::NotFound) => Err(CorsaNotFoundError::new(project_root)),
            Err(CorsaResolveError::InvalidRuntime { package }) => {
                Err(CorsaNotFoundError::new_invalid(project_root, &package))
            }
        }
    }

    /// Override the deterministic worker count for CLI checks without process-global mutation.
    pub fn set_checker_count(&mut self, count: Option<usize>) {
        self.checker_count = count.filter(|count| *count > 0);
    }

    /// Share the machine checker budget across concurrently checked programs.
    /// Explicit CLI or environment widths remain exact overrides.
    pub fn set_program_concurrency(&mut self, count: usize) {
        self.program_concurrency = count.max(1);
    }

    fn checkers(&self) -> usize {
        self.checker_count.unwrap_or_else(cli::checker_count)
    }

    /// Width for a program that is checked unsharded (see `unsharded_checker_count`).
    fn unsharded_checkers(&self) -> usize {
        cli::unsharded_checker_count(self.checker_count, self.program_concurrency)
    }

    pub fn corsa_path(&self) -> &Path {
        &self.corsa_path
    }

    pub(crate) fn incremental_metrics(&self) -> IncrementalCheckMetrics {
        self.incremental_session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .metrics
    }

    pub fn check(&self, project: &VirtualProject) -> CorsaResult<TypeCheckResult> {
        self.check_with_servers(project, None)
    }

    /// Run type checking on the virtual project. `servers` is the number of
    /// parallel Corsa CLI processes the project is partitioned across
    /// (`None` auto-tunes from the machine width and project size).
    pub fn check_with_servers(
        &self,
        project: &VirtualProject,
        servers: Option<usize>,
    ) -> CorsaResult<TypeCheckResult> {
        let _materialize_lock = MaterializeLock::acquire(project.virtual_root())?;
        profile!("canon.executor.materialize", project.materialize())?;

        // The generated config is sanitized, so it cannot carry the user's own
        // `compilerOptions` diagnostics; a separate input-less program does
        // (#3448). Errors it finds must fail the check exactly as `tsc`'s do.
        // It reads only the materialized config, so it runs beside the check
        // instead of after it.
        let (result, options) = std::thread::scope(|scope| {
            let probe = scope.spawn(|| option_probe::option_diagnostics(&self.corsa_path, project));
            let result = self.check_program(project, servers);
            (result, probe.join())
        });
        let mut result = result?;
        let options = options.unwrap_or_else(|panic| std::panic::resume_unwind(panic));
        if options.iter().any(|diagnostic| diagnostic.severity == 1) {
            result.success = false;
            result.exit_code = result.exit_code.max(1);
        }
        result.diagnostics.extend(options);
        // A skipped `extends` is an error like `tsc`'s TS6053, but the rest
        // of the program was still checked with the remaining options.
        let unresolved = cli::project_diagnostics::unresolved_extends(project);
        if !unresolved.is_empty() {
            result.success = false;
            result.exit_code = result.exit_code.max(1);
        }
        result.diagnostics.extend(unresolved);
        result.diagnostics = dedup_diagnostics(result.diagnostics);
        Ok(result)
    }

    /// Type-check the materialized project, walking the fallback ladder. The
    /// caller holds the materialize lock.
    fn check_program(
        &self,
        project: &VirtualProject,
        servers: Option<usize>,
    ) -> CorsaResult<TypeCheckResult> {
        if profile!(
            "canon.corsa.diagnostic_route",
            project.requires_project_session()
        ) {
            return self.check_with_project_session(project);
        }
        let servers = servers.unwrap_or_else(|| {
            auto_server_count(project, self.checkers(), self.program_concurrency)
        });
        if servers > 1 {
            match profile!(
                "canon.corsa.cli",
                check_with_cli_sharded(
                    &self.corsa_path,
                    project,
                    servers,
                    (self.checkers(), self.unsharded_checkers()),
                )
            ) {
                Ok(result) => return Ok(result),
                Err(cli_error) => {
                    // Degrade to a single CLI program before the session API:
                    // a shard-specific failure must not cost CLI support.
                    if let CheckFailure::Runtime(ref error) = cli_error {
                        warn_fallback(FallbackStep::ShardedCliToSingleCli, error);
                    }
                }
            }
        }

        let cli_error = match profile!(
            "canon.corsa.cli",
            check_with_cli_classified(&self.corsa_path, project, self.unsharded_checkers())
        ) {
            Ok(result) => return Ok(result),
            Err(cli_error) => {
                // Parser abort requires the complete same-config semantic phase;
                // only an unexpected runtime failure degrades this capability.
                if let CheckFailure::Runtime(ref error) = cli_error {
                    warn_fallback(FallbackStep::CliToSession, error);
                }
                cli_error.into_error()
            }
        };

        self.check_with_project_session(project)
            .map_err(|session_error| CorsaError::CorsaExecution {
                exit_code: 1,
                message: cstr!(
                    "{cli_error}\nSame-config native diagnostic completion failed: {session_error}"
                ),
            })
    }

    /// Emit declaration files from the materialized virtual project.
    pub fn emit_declarations(
        &self,
        project: &VirtualProject,
        options: &DeclarationEmitOptions,
    ) -> CorsaResult<DeclarationEmitResult> {
        let _materialize_lock = MaterializeLock::acquire(project.virtual_root())?;
        profile!("canon.executor.materialize_dts", project.materialize())?;
        let config_path = profile!(
            "canon.project.dts_tsconfig",
            project.write_declaration_tsconfig(
                options.out_dir.as_path(),
                options.declaration_map,
                options.mirror_layout,
            )
        )?;
        let output = profile!(
            "canon.corsa.emit_dts",
            Command::new(&self.corsa_path)
                .current_dir(project.virtual_root())
                .arg("--pretty")
                .arg("false")
                .arg("--project")
                .arg(&config_path)
                .output()
        )?;

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            #[expect(clippy::disallowed_types, reason = "from_utf8_lossy yields std Cow")]
            let stderr = std::string::String::from_utf8_lossy(&output.stderr);
            #[expect(clippy::disallowed_types, reason = "from_utf8_lossy yields std Cow")]
            let stdout = std::string::String::from_utf8_lossy(&output.stdout);
            let message = if stderr.trim().is_empty() {
                stdout.trim().to_owned().into()
            } else if stdout.trim().is_empty() {
                stderr.trim().to_owned().into()
            } else {
                cstr!("{}\n{}", stderr.trim(), stdout.trim())
            };
            return Err(CorsaError::CorsaExecution { exit_code, message });
        }

        if options.mirror_layout {
            // Another program resolves these in place of the sources: keep
            // TypeScript's own specifiers and layout.
            return Ok(DeclarationEmitResult {
                files: profile!(
                    "canon.dts.collect_outputs",
                    collect_declaration_outputs(options.out_dir.as_path(), false)
                )?,
            });
        }
        declaration_maps::rewrite_declaration_map_outputs(options.out_dir.as_path(), project)?;
        // TypeScript can widen the emit root when a retained SFC reaches an
        // inferred dependency outside the authored `rootDir`. Restore the
        // caller's final layout before spelling package-local helper imports;
        // their relative depth must describe the published path, not the
        // temporary widened path.
        project.finalize_declaration_outputs(options.out_dir.as_path(), &config_path)?;
        profile!(
            "canon.dts.rewrite_outputs",
            rewrite_declaration_outputs(options.out_dir.as_path())
        )?;

        Ok(DeclarationEmitResult {
            files: profile!(
                "canon.dts.collect_outputs",
                collect_declaration_outputs(options.out_dir.as_path(), true)
            )?,
        })
    }
}

fn collect_declaration_outputs(
    out_dir: &Path,
    rewrite_specifiers: bool,
) -> CorsaResult<Vec<DeclarationOutput>> {
    let mut files = Vec::new();
    let rewriter = ImportRewriter::new();
    if !out_dir.exists() {
        return Ok(files);
    }

    for entry in walkdir::WalkDir::new(out_dir) {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !is_declaration_file(path) {
            continue;
        }

        let content = std::fs::read_to_string(path)?;
        files.push(DeclarationOutput {
            path: path.to_path_buf(),
            content: if rewrite_specifiers {
                rewriter
                    .rewrite_declaration_specifiers(&content, SourceType::ts())
                    .code
            } else {
                content.into()
            },
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn map_corsa_error(message: String) -> CorsaError {
    CorsaError::CorsaExecution {
        exit_code: -1,
        message,
    }
}

fn should_fallback_to_cli(error: &str) -> bool {
    error.contains("expected tuple marker")
        || error.contains("expected uint8 marker")
        || error.contains("expected bin marker")
        || error.contains("process is closed: jsonrpc reader")
        || error.contains("Broken pipe")
        || error.contains("broken pipe")
}

#[cfg(test)]
mod tests;
