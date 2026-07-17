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
    DeclarationEmitOptions, DeclarationEmitResult, DeclarationOutput, TypeCheckResult,
};
use super::virtual_project::VirtualProject;
use oxc_span::SourceType;
use vize_carton::{
    String,
    corsa_resolver::{CorsaResolveError, CorsaResolveRequest, resolve_corsa_executable},
    cstr, profile,
};

const DECLARATION_HELPERS_FILE: &str = crate::virtual_ts::SHARED_PREAMBLE_FILE_NAME;

mod cli;
mod declaration_maps;
mod diagnostics;
mod session;

use cli::{auto_server_count, check_with_cli, check_with_cli_sharded};
use session::IncrementalSessionState;
#[cfg(test)]
use session::collect_virtual_file_uris;

/// Batch executor backed by `corsa`'s project-session diagnostics API.
pub struct CorsaExecutor {
    /// Path to the resolved Corsa executable.
    corsa_path: PathBuf,
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

        match resolve_corsa_executable(request) {
            Ok(corsa_path) => Ok(Self {
                corsa_path,
                incremental_session: Mutex::new(IncrementalSessionState::default()),
            }),
            Err(CorsaResolveError::ExplicitNotFound { path, .. }) => {
                Err(CorsaNotFoundError::new_explicit(project_root, &path))
            }
            Err(CorsaResolveError::NotFound) => Err(CorsaNotFoundError::new(project_root)),
        }
    }

    pub fn corsa_path(&self) -> &Path {
        &self.corsa_path
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

        let servers = servers.unwrap_or_else(|| auto_server_count(project));
        if servers > 1 {
            match profile!(
                "canon.corsa.cli",
                check_with_cli_sharded(&self.corsa_path, project, servers)
            ) {
                Ok(result) => return Ok(result),
                Err(cli_error) => {
                    // Degrade to a single CLI program before the session API:
                    // a shard-specific failure must not cost CLI support.
                    warn_fallback(FallbackStep::ShardedCliToSingleCli, &cli_error);
                }
            }
        }

        match profile!("canon.corsa.cli", check_with_cli(&self.corsa_path, project)) {
            Ok(result) => return Ok(result),
            Err(cli_error) => {
                // Fall through to the project-session API. This keeps the batch
                // runner usable with runtimes whose CLI diagnostics are not
                // available or not parseable.
                warn_fallback(FallbackStep::CliToSession, &cli_error);
            }
        }

        self.check_with_project_session(project)
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
            project.write_declaration_tsconfig(options.out_dir.as_path(), options.declaration_map)
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
            #[allow(clippy::disallowed_types)]
            let stderr = std::string::String::from_utf8_lossy(&output.stderr);
            #[allow(clippy::disallowed_types)]
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

        profile!(
            "canon.dts.rewrite_outputs",
            rewrite_declaration_outputs(options.out_dir.as_path())
        )?;
        declaration_maps::rewrite_declaration_map_outputs(options.out_dir.as_path(), project)?;

        Ok(DeclarationEmitResult {
            files: profile!(
                "canon.dts.collect_outputs",
                collect_declaration_outputs(options.out_dir.as_path())
            )?,
        })
    }
}

fn collect_declaration_outputs(out_dir: &Path) -> CorsaResult<Vec<DeclarationOutput>> {
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
            content: rewriter
                .rewrite_declaration_specifiers(&content, SourceType::ts())
                .code,
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn rewrite_declaration_outputs(out_dir: &Path) -> CorsaResult<()> {
    let rewriter = ImportRewriter::new();
    if !out_dir.exists() {
        return Ok(());
    }

    let mut wrote_vue_declaration = false;
    for entry in walkdir::WalkDir::new(out_dir) {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !is_declaration_file(path) {
            continue;
        }

        let content = std::fs::read_to_string(path)?;
        let mut rewritten = rewriter
            .rewrite_declaration_specifiers(&content, SourceType::ts())
            .code;
        // Generated `.vue.d.ts` outputs reference the hoisted helper type
        // aliases (`__EmitFn`, `__RuntimePropShape`, ...). Wire each one to
        // the helpers declaration shipped alongside the outputs so consumer
        // programs resolve them without including the virtual mirror.
        if name.ends_with(".vue.d.ts") {
            wrote_vue_declaration = true;
            let depth = path
                .strip_prefix(out_dir)
                .ok()
                .and_then(|relative| relative.parent())
                .map(|parent| parent.components().count())
                .unwrap_or(0);
            let mut reference = String::from("/// <reference path=\"");
            for _ in 0..depth {
                reference.push_str("../");
            }
            reference.push_str(DECLARATION_HELPERS_FILE);
            reference.push_str("\" />\n");
            reference.push_str(&rewritten);
            rewritten = reference.as_str().into();
        }
        if rewritten.as_str() != content {
            std::fs::write(path, rewritten.as_str())?;
        }
    }

    if wrote_vue_declaration {
        std::fs::write(
            out_dir.join(DECLARATION_HELPERS_FILE),
            crate::virtual_ts::DECLARATION_HELPERS_DTS,
        )?;
    }

    Ok(())
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

/// A degradation step on the Corsa fallback ladder. Each variant names the
/// faster/stronger path that failed and the slower/weaker path taken instead,
/// so an operator can tell from a single signal which capability was lost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FallbackStep {
    /// Parallel sharded CLI check failed; degraded to a single CLI program
    /// (doubles work on wide machines).
    ShardedCliToSingleCli,
    /// The CLI fast path failed; degraded to the much slower project-session
    /// API.
    CliToSession,
    /// The project-session API could not be spawned/handshaken; degraded back
    /// to a single CLI program.
    SessionToCli,
}

impl FallbackStep {
    const fn description(self) -> &'static str {
        match self {
            FallbackStep::ShardedCliToSingleCli => {
                "sharded Corsa CLI check failed; degraded to a single CLI program"
            }
            FallbackStep::CliToSession => {
                "Corsa CLI fast path unavailable; degraded to the slower project-session API"
            }
            FallbackStep::SessionToCli => {
                "Corsa project-session API unavailable; degraded to a single CLI program"
            }
        }
    }
}

/// Coarse cause of a ladder degradation, so the signal distinguishes a dead
/// process from an unparseable runtime from a real type-check failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FallbackCause {
    /// The Corsa process could not be spawned or died (IO error, broken pipe,
    /// closed reader, panic).
    Spawn,
    /// The Corsa process ran but its output could not be decoded/parsed.
    Parse,
    /// The Corsa process produced a usable error that is neither a spawn nor a
    /// parse problem.
    Check,
}

impl FallbackCause {
    const fn label(self) -> &'static str {
        match self {
            FallbackCause::Spawn => "spawn",
            FallbackCause::Parse => "parse",
            FallbackCause::Check => "check",
        }
    }
}

/// Classify a ladder error as a spawn, parse, or check failure from its kind
/// and message, without depending on any single error variant.
fn classify_fallback_cause(error: &CorsaError) -> FallbackCause {
    match error {
        CorsaError::Io(_) => FallbackCause::Spawn,
        CorsaError::JsonParse(_) => FallbackCause::Parse,
        _ => {
            let message = cstr!("{error}");
            if message.contains("Broken pipe")
                || message.contains("broken pipe")
                || message.contains("process is closed")
                || message.contains("jsonrpc reader")
                || message.contains("worker panicked")
                || message.contains("No such file")
                || message.contains("not found")
            {
                FallbackCause::Spawn
            } else if message.contains("marker")
                || message.contains("decode")
                || message.contains("parse")
                || message.contains("unexpected")
            {
                FallbackCause::Parse
            } else {
                FallbackCause::Check
            }
        }
    }
}

/// Whether the once-per-run fallback signal has already been emitted to stderr.
/// The structured `tracing` event fires on every degradation (subscribers can
/// dedup/aggregate), but the human-facing stderr line is emitted at most once
/// per process so a large project's repeated fallbacks stay quiet.
static FALLBACK_NOTICE_EMITTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Emit an observable signal that the Corsa integration degraded to a
/// slower/weaker path. Always records a structured `tracing::warn!` event
/// (consumed by the LSP and CI which install a subscriber); additionally prints
/// a single human-readable stderr warning per process for the `vize check` CLI,
/// which installs no subscriber. The happy path never reaches this function, so
/// a fully fast-path run stays silent.
fn warn_fallback(step: FallbackStep, error: &CorsaError) {
    let cause = classify_fallback_cause(error);
    tracing::warn!(
        target: "vize_canon::corsa::fallback",
        fallback = ?step,
        cause = cause.label(),
        error = %error,
        "{}",
        step.description(),
    );

    if let Some(notice) = fallback_stderr_notice(step, cause) {
        eprintln!("{notice}");
    }
}

/// Build the human-facing stderr warning for a ladder degradation, claiming the
/// once-per-process slot as a side effect. Returns `None` when the notice is
/// suppressed (`VIZE_SILENCE_CORSA_FALLBACK`) or already emitted this run, so a
/// large project's repeated fallbacks stay quiet. Separated from `eprintln!` so
/// the once/suppression policy is unit-testable without capturing real stderr.
fn fallback_stderr_notice(step: FallbackStep, cause: FallbackCause) -> Option<String> {
    // Noise-sensitive embedders can suppress only the stderr line; the
    // structured `tracing` event still fires for observability.
    if std::env::var_os("VIZE_SILENCE_CORSA_FALLBACK").is_some() {
        return None;
    }
    if FALLBACK_NOTICE_EMITTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    Some(cstr!(
        "\x1b[33mwarning:\x1b[0m corsa: {} ({} failure). Type checking continues on a slower path.",
        step.description(),
        cause.label(),
    ))
}

#[cfg(test)]
mod tests;
