//! Running the `compilerOptions` probe and turning its output into project
//! diagnostics (#3448).
//!
//! See [`crate::batch::virtual_project::option_probe`] for why the probe config
//! exists and what it contains. Here it is executed and read back: only
//! diagnostics the checker positions *on the probe config itself* are kept, and
//! only those in TypeScript's 5xxx range — the codes reserved for command-line
//! and config-file option problems. Everything else the input-less program can
//! say is noise for this purpose, most visibly `TS18002` ("the 'files' list in
//! config file is empty"), which is the price of building no program at all.
//!
//! Diagnostics are anchored on the project exactly as
//! `cli::project_diagnostics::config` anchors the main run's config errors, so an
//! option that survives sanitization and is therefore reported by *both* runs
//! collapses in `dedup_diagnostics` instead of being reported twice.

use std::path::Path;
use std::process::Command;

use vize_carton::profile;

use super::super::virtual_project::option_probe::OptionDiagnosticNarrowing;
use super::super::{Diagnostic, VirtualProject};
use super::diagnostics::dedup_diagnostics;

/// TypeScript reserves 5000-5999 for command-line and config option messages.
const OPTION_DIAGNOSTIC_CODES: std::ops::Range<u32> = 5000..6000;

/// Option diagnostics for the user's own `compilerOptions`.
///
/// Best-effort: a probe that cannot be written or run leaves the main run's
/// diagnostics untouched rather than failing the check, because the probe adds
/// diagnostics and never gates any.
pub(super) fn option_diagnostics(corsa_path: &Path, project: &VirtualProject) -> Vec<Diagnostic> {
    let Ok(Some((config_path, narrowing))) = project.write_option_probe_tsconfig() else {
        return Vec::new();
    };

    let Ok(output) = profile!("canon.corsa.cli.option_probe", {
        Command::new(corsa_path)
            .current_dir(project.virtual_root())
            .arg("--pretty")
            .arg("false")
            .arg("--project")
            .arg(&config_path)
            .output()
    }) else {
        return Vec::new();
    };

    let anchor = project.project_diagnostics_anchor();
    let mut diagnostics = Vec::new();
    #[allow(clippy::disallowed_types)]
    for stream in [&output.stdout, &output.stderr] {
        let text = std::string::String::from_utf8_lossy(stream);
        collect_option_diagnostics(
            text.as_ref(),
            &config_path,
            &anchor,
            narrowing,
            &mut diagnostics,
        );
    }
    dedup_diagnostics(diagnostics)
}

/// Parse `output` into anchored option diagnostics.
///
/// A diagnostic's message can continue on following indented lines, so the
/// parser tracks whether the last header line was kept: continuations of a
/// dropped diagnostic must be dropped with it rather than appended to the
/// previous kept one.
fn collect_option_diagnostics(
    output: &str,
    config_path: &Path,
    anchor: &Path,
    narrowing: OptionDiagnosticNarrowing,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut last_was_kept = false;
    for line in output.lines() {
        match parse_option_diagnostic_line(line, config_path, narrowing) {
            Some(Some((code, message, severity))) => {
                diagnostics.push(Diagnostic {
                    file: anchor.to_path_buf(),
                    line: 0,
                    column: 0,
                    message,
                    code: Some(code),
                    severity,
                    block_type: None,
                });
                last_was_kept = true;
            }
            // A diagnostic header this probe does not report.
            Some(None) => last_was_kept = false,
            None => {
                let continuation = line.trim();
                if last_was_kept
                    && !continuation.is_empty()
                    && let Some(last) = diagnostics.last_mut()
                {
                    last.message.push('\n');
                    last.message.push_str(continuation);
                }
            }
        }
    }
}

/// `Some(Some(..))` for an option diagnostic on the probe config, `Some(None)`
/// for any other diagnostic header on it, `None` when the line is not a
/// diagnostic header for that file at all.
#[allow(clippy::type_complexity)]
fn parse_option_diagnostic_line(
    line: &str,
    config_path: &Path,
    narrowing: OptionDiagnosticNarrowing,
) -> Option<Option<(u32, vize_carton::String, u8)>> {
    let (prefix, suffix) = line.split_once("): ")?;
    let open = prefix.rfind('(')?;
    if !names_probe_config(&prefix[..open], config_path) {
        return None;
    }

    let (severity, rest) = suffix.split_once(' ')?;
    let severity = match severity {
        "error" => 1,
        "warning" => 2,
        "info" => 3,
        _ => return None,
    };
    let (code, message) = rest.split_once(": ")?;
    let Some(code) = code
        .strip_prefix("TS")
        .and_then(|code| code.parse::<u32>().ok())
        .filter(|code| OPTION_DIAGNOSTIC_CODES.contains(code))
        .filter(|code| narrowing.keeps(*code))
    else {
        return Some(None);
    };
    Some(Some((code, message.into(), severity)))
}

/// The checker reports the probe config by whatever path the invocation used —
/// a bare file name from `current_dir`, or the absolute path it was given.
fn names_probe_config(reported: &str, config_path: &Path) -> bool {
    let reported = Path::new(reported);
    reported == config_path || reported.file_name() == config_path.file_name()
}

#[cfg(test)]
mod tests;
