use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use super::super::{Diagnostic, TypeCheckResult, VirtualProject};
use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::executor::diagnostics::{
    DiagnosticMapper, relative_module_resolves_on_disk, should_skip_diagnostic,
    should_skip_original_diagnostic,
};
use vize_carton::profile;
use vize_carton::{String, cstr};

pub(super) fn check_with_cli(
    corsa_path: &Path,
    project: &VirtualProject,
) -> CorsaResult<TypeCheckResult> {
    let config_path = project.virtual_root().join("tsconfig.json");
    let output = profile!(
        "canon.corsa.cli.command",
        Command::new(corsa_path)
            .current_dir(project.virtual_root())
            .arg("--pretty")
            .arg("false")
            .arg("--project")
            .arg(&config_path)
            .output()
    )?;
    let diagnostics = profile!(
        "canon.corsa.cli.parse",
        parse_output_diagnostics(&output, project)
    );
    let success = output.status.success()
        && diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != 1);

    // A non-zero exit is a runner failure only when the output carries no
    // diagnostic-shaped lines at all (bad invocation, crash, missing CLI
    // support). Recognizable diagnostics whose every entry was suppressed or
    // failed source mapping still prove the CLI ran the project; falling back
    // to the per-file project-session API there costs orders of magnitude
    // more wall time for the same answer.
    if !output.status.success() && diagnostics.is_empty() {
        if !output_contains_diagnostic_lines(&output) {
            return Err(CorsaError::CorsaExecution {
                exit_code: output.status.code().unwrap_or(-1),
                message: output_message(&output),
            });
        }
        // Project-level diagnostics (e.g. `error TS2688: Cannot find type
        // definition file for 'x'.`) abort the runtime's semantic pass, so an
        // otherwise clean report may be incomplete. Surface them on stderr so
        // a green run is never silently built on a broken project config.
        eprintln!(
            "\x1b[33mwarning:\x1b[0m corsa reported project-level errors; type checking may be incomplete:\n{}",
            output_message(&output)
        );
    }

    Ok(TypeCheckResult {
        exit_code: output.status.code().unwrap_or(if success { 0 } else { 1 }),
        success,
        diagnostics,
    })
}

fn parse_output_diagnostics(output: &Output, project: &VirtualProject) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut mapper = DiagnosticMapper::new(project);
    #[allow(clippy::disallowed_types)]
    let stdout = std::string::String::from_utf8_lossy(&output.stdout);
    parse_cli_diagnostics(stdout.as_ref(), project, &mut mapper, &mut diagnostics);
    #[allow(clippy::disallowed_types)]
    let stderr = std::string::String::from_utf8_lossy(&output.stderr);
    parse_cli_diagnostics(stderr.as_ref(), project, &mut mapper, &mut diagnostics);
    diagnostics
}

fn parse_cli_diagnostics(
    output: &str,
    project: &VirtualProject,
    mapper: &mut DiagnosticMapper<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for line in output.lines() {
        if let Some(diagnostic) = parse_cli_diagnostic_line(line, project, mapper) {
            diagnostics.push(diagnostic);
            continue;
        }
        if is_cli_diagnostic_line(line) {
            continue;
        }

        let Some(last) = diagnostics.last_mut() else {
            continue;
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        last.message.push('\n');
        last.message.push_str(line);
    }
}

fn parse_cli_diagnostic_line(
    line: &str,
    project: &VirtualProject,
    mapper: &mut DiagnosticMapper<'_>,
) -> Option<Diagnostic> {
    let (prefix, suffix) = line.split_once("): ")?;
    let open = prefix.rfind('(')?;
    let path = &prefix[..open];
    let position = &prefix[open + 1..];
    let (line, column) = position.split_once(',')?;
    let line = line.parse::<u32>().ok()?.saturating_sub(1);
    let column = column.parse::<u32>().ok()?.saturating_sub(1);

    let (severity, rest) = suffix.split_once(' ')?;
    let severity = match severity {
        "error" => 1,
        "warning" => 2,
        "info" => 3,
        _ => return None,
    };
    let (code, message) = rest.split_once(": ")?;
    let code = code
        .strip_prefix("TS")
        .and_then(|code| code.parse::<u32>().ok());
    if should_skip_diagnostic(code, message) {
        return None;
    }
    if code == Some(6133) && !mapper.preserves_unused_diagnostics() {
        return None;
    }

    let virtual_path = normalize_cli_path(path, project.virtual_root());
    let original = mapper.map_to_original(&virtual_path, line, column)?;
    if should_skip_original_diagnostic(code, &original) {
        return None;
    }

    // Suppress the false `TS2307` raised for a relative import of a sibling that
    // exists on disk but sits outside an explicit file subset. See the matching
    // check in `diagnostics::map_lsp_diagnostic`.
    if code == Some(2307) && relative_module_resolves_on_disk(message, &original.path) {
        return None;
    }

    Some(Diagnostic {
        file: original.path,
        line: original.line,
        column: original.column,
        message: message.into(),
        code,
        severity,
        block_type: original.block_type,
    })
}

fn output_contains_diagnostic_lines(output: &Output) -> bool {
    [&output.stdout, &output.stderr].into_iter().any(|stream| {
        #[allow(clippy::disallowed_types)]
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
    digits > 0 && code[digits..].starts_with(':')
}

fn is_cli_diagnostic_line(line: &str) -> bool {
    let Some((prefix, suffix)) = line.split_once("): ") else {
        return false;
    };
    let Some(open) = prefix.rfind('(') else {
        return false;
    };
    let position = &prefix[open + 1..];
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

fn normalize_cli_path(path: &str, virtual_root: &Path) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        virtual_root.join(path)
    }
}

fn output_message(output: &Output) -> String {
    #[allow(clippy::disallowed_types)]
    let stderr = std::string::String::from_utf8_lossy(&output.stderr);
    #[allow(clippy::disallowed_types)]
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
mod tests {
    use super::{is_cli_diagnostic_line, is_global_diagnostic_line, parse_cli_diagnostics};
    use crate::batch::VirtualProject;
    use crate::batch::executor::diagnostics::DiagnosticMapper;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };
    use vize_carton::cstr;

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

        let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join("tests")
            .join(&*cstr!(
                "cli-fallback-{name}-{}-{case_id}",
                std::process::id()
            ))
    }

    #[test]
    fn recognizes_global_and_positioned_diagnostic_lines() {
        assert!(is_global_diagnostic_line(
            "error TS2688: Cannot find type definition file for 'vite/client'."
        ));
        assert!(is_global_diagnostic_line("warning TS1: w"));
        assert!(!is_global_diagnostic_line(
            "  The file is in the program because:"
        ));
        assert!(!is_global_diagnostic_line("error: missing argument"));
        assert!(!is_global_diagnostic_line("error TSX: nope"));

        assert!(is_cli_diagnostic_line(
            "src/App.vue.ts(3,7): error TS2322: Type 'string' is not assignable to type 'number'."
        ));
        assert!(!is_cli_diagnostic_line(
            "error TS2688: Cannot find type definition file for 'vite/client'."
        ));
    }

    #[test]
    fn parses_cli_diagnostics_back_to_original_files() {
        let case_dir = unique_case_dir("diagnostics");
        let _ = fs::remove_dir_all(&case_dir);
        let source = case_dir.join("src").join("main.ts");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "const value: number = 'x';\n").unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_path(&source).unwrap();
        project.materialize().unwrap();

        let output = cstr!(
            "{}(1,7): error TS2322: Type 'string' is not assignable to type 'number'.",
            project.virtual_root().join("src").join("main.ts").display()
        );
        let mut diagnostics = Vec::new();
        let mut mapper = DiagnosticMapper::new(&project);
        parse_cli_diagnostics(output.as_str(), &project, &mut mapper, &mut diagnostics);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].file, source);
        assert_eq!(diagnostics[0].line, 0);
        assert_eq!(diagnostics[0].column, 6);
        assert_eq!(diagnostics[0].code, Some(2322));

        let _ = fs::remove_dir_all(&case_dir);
    }
}
