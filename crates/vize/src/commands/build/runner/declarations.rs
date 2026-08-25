//! Declaration emit wiring for `vize build --declaration`.
//!
//! Reuses the `vize check --declaration` runner end to end: the built SFC
//! sources become explicit check inputs and the build output directory becomes
//! the declaration directory, so `vize build --declaration` emits exactly what
//! `vize check --declaration` would emit for the same files.

use std::path::PathBuf;

use super::output::PlannedInput;
use crate::commands::{build::BuildArgs, check::CheckArgs};

/// Run the check command's declaration flow for the built SFC sources.
///
/// Exits the process when type errors are reported or emit fails, matching
/// `vize check --declaration`.
pub(super) fn emit(args: &BuildArgs, inputs: &[PlannedInput]) {
    let patterns = inputs
        .iter()
        .map(|input| input.source.to_string_lossy().into_owned())
        .collect();
    let check_args = declaration_check_args(args, patterns);
    crate::commands::check::runner::run_direct(&check_args);
}

#[allow(clippy::disallowed_types)]
fn declaration_check_args(args: &BuildArgs, patterns: Vec<std::string::String>) -> CheckArgs {
    CheckArgs {
        patterns,
        config: args.config.clone(),
        no_config: args.no_config,
        #[cfg(unix)]
        socket: None,
        tsconfig: None,
        format: "text".into(),
        show_virtual_ts: false,
        save_virtual_ts_for: Vec::new(),
        max_warnings: None,
        no_check_props: false,
        no_check_emits: false,
        no_check_template_bindings: false,
        quiet: false,
        profile: false,
        corsa_path: None,
        servers: None,
        declaration: true,
        declaration_dir: Some(declaration_output_dir(args)),
        // The build command owns the machine-readable export for its own run;
        // the delegated declaration pass never writes one.
        profile_export: Default::default(),
    }
}

/// Absolute directory declarations are written to: `--declaration-dir` when
/// given, otherwise the build output directory. Resolved against the current
/// directory because both build flags are documented as cwd-relative, while
/// the check runner resolves relative declaration dirs against the project
/// root.
fn declaration_output_dir(args: &BuildArgs) -> PathBuf {
    let dir = args.declaration_dir.as_deref().unwrap_or(&args.output);
    if dir.is_absolute() {
        return dir.to_path_buf();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    vize_s0::path::canonicalize_non_verbatim(&cwd.join(dir))
}
