use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use super::super::{TypeCheckResult, VirtualProject};
use crate::batch::declaration_path::is_declaration_file;
use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::executor::diagnostics::DiagnosticMapper;
use vize_carton::{FxHashMap, profile};
use vize_carton::{String, cstr};

mod checkers;
mod diagnostic_paths;
mod file_diagnostics;
mod import_resolution;
mod output;
mod partition;
mod patterns;
mod project_diagnostics;
mod shard_sizing;
mod union_find;

pub(super) use checkers::checker_count;
use checkers::rejects_checkers_flag;
use diagnostic_paths::normalize_cli_path;
use file_diagnostics::{Decoded, parse_cli_diagnostic_line};

#[cfg(test)]
use output::parse_cli_diagnostics;
use output::parse_output_diagnostics;
use partition::partition_virtual_files;
use shard_sizing::shard_count;

mod run;
pub(super) use run::{auto_server_count, check_with_cli, check_with_cli_sharded};

struct ShardPlan<'a> {
    /// Virtual paths to include per shard (owned Vue files plus every shared
    /// file).
    shards: Vec<Vec<&'a Path>>,
    /// Registered-file ownership. Absent shared or real-tree diagnostics are
    /// accepted from every shard, then deduplicated in the merged result.
    owners: FxHashMap<PathBuf, usize>,
}

fn is_vue_original(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}

fn is_ambient_declaration(path: &Path) -> bool {
    is_declaration_file(path)
}

fn declares_program_wide_types(content: &str) -> bool {
    content.contains("declare module") || content.contains("declare global")
}

fn run_cli_for_config(
    corsa_path: &Path,
    project: &VirtualProject,
    config_path: &Path,
    checkers: usize,
    owns: &dyn Fn(&Path) -> bool,
) -> CorsaResult<TypeCheckResult> {
    let output = profile!("canon.corsa.cli.command", {
        let mut command = Command::new(corsa_path);
        command.current_dir(project.virtual_root());
        // The checker count decides the diagnostic set, so it is always pinned
        // explicitly (see `checker_count`) rather than left to Corsa's default.
        command.arg("--checkers").arg(cstr!("{checkers}").as_str());
        command
            .arg("--pretty")
            .arg("false")
            .arg("--project")
            .arg(config_path)
            .output()
    })?;
    let diagnostics = profile!(
        "canon.corsa.cli.parse",
        parse_output_diagnostics(&output, project, owns)
    );

    if !output.status.success() && rejects_checkers_flag(&diagnostics) {
        return Err(CorsaError::CorsaExecution {
            exit_code: output.status.code().unwrap_or(-1),
            message: cstr!(
                "corsa runtime does not support `--checkers`, which vize requires for \
                 deterministic diagnostics (#3905); upgrade the pinned corsa runtime.\n{}",
                output_message(&output)
            ),
        });
    }

    let success = (output.status.success() || patterns::only_pattern_warnings(&output, project))
        && diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != 1);

    // A non-zero exit is a runner failure only when the output carries no
    // diagnostic-shaped lines at all (bad invocation, crash, missing CLI
    // support). Recognizable diagnostics whose every entry was suppressed or
    // failed source mapping still prove the CLI ran the project; falling back
    // to the per-file project-session API there costs orders of magnitude
    // more wall time for the same answer.
    if !output.status.success()
        && diagnostics.is_empty()
        && !output_contains_diagnostic_lines(&output)
    {
        return Err(CorsaError::CorsaExecution {
            exit_code: output.status.code().unwrap_or(-1),
            message: output_message(&output),
        });
    }

    Ok(TypeCheckResult {
        exit_code: if success {
            0
        } else {
            output.status.code().unwrap_or(1)
        },
        success,
        diagnostics,
    })
}

fn output_contains_diagnostic_lines(output: &Output) -> bool {
    [&output.stdout, &output.stderr].into_iter().any(|stream| {
        #[expect(clippy::disallowed_types, reason = "from_utf8_lossy yields std Cow")]
        let text = std::string::String::from_utf8_lossy(stream);
        text.lines()
            .any(|line| is_cli_diagnostic_line(line) || is_global_diagnostic_line(line))
    })
}

/// Whether `line` is a file-less project-level diagnostic such as
/// `error TS2688: Cannot find type definition file for 'vite/client'.`
fn is_global_diagnostic_line(line: &str) -> bool {
    let Some(rest) = line
        .strip_prefix("error ")
        .or_else(|| line.strip_prefix("warning "))
        .or_else(|| line.strip_prefix("info "))
    else {
        return false;
    };
    let Some(code) = rest.strip_prefix("TS") else {
        return false;
    };
    let digits = code.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0 && code.get(digits..).is_some_and(|rest| rest.starts_with(':'))
}

fn is_cli_diagnostic_line(line: &str) -> bool {
    let Some((prefix, suffix)) = line.split_once("): ") else {
        return false;
    };
    let Some((_, position)) = prefix.rsplit_once('(') else {
        return false;
    };
    let Some((line, column)) = position.split_once(',') else {
        return false;
    };
    if line.parse::<u32>().is_err() || column.parse::<u32>().is_err() {
        return false;
    }

    matches!(
        suffix.split_once(' ').map(|(severity, _)| severity),
        Some("error" | "warning" | "info")
    )
}

fn output_message(output: &Output) -> String {
    #[expect(clippy::disallowed_types, reason = "from_utf8_lossy yields std Cow")]
    let stderr = std::string::String::from_utf8_lossy(&output.stderr);
    #[expect(clippy::disallowed_types, reason = "from_utf8_lossy yields std Cow")]
    let stdout = std::string::String::from_utf8_lossy(&output.stdout);
    let stderr = stderr.trim();
    let stdout = stdout.trim();
    if stderr.is_empty() {
        return stdout.to_owned().into();
    }
    if stdout.is_empty() {
        return stderr.to_owned().into();
    }
    cstr!("{}\n{}", stderr, stdout)
}

#[cfg(test)]
mod tests;
