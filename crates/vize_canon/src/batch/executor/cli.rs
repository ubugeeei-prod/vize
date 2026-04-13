use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use super::super::{Diagnostic, TypeCheckResult, VirtualProject};
use crate::batch::error::{CorsaError, CorsaResult};
use crate::batch::executor::diagnostics::should_skip_diagnostic;
use vize_carton::{cstr, String};

pub(super) fn check_with_cli(
    corsa_path: &Path,
    project: &VirtualProject,
) -> CorsaResult<TypeCheckResult> {
    let config_path = project.virtual_root().join("tsconfig.json");
    let output = Command::new(corsa_path)
        .current_dir(project.virtual_root())
        .arg("--pretty")
        .arg("false")
        .arg("--project")
        .arg(&config_path)
        .output()?;
    let diagnostics = parse_output_diagnostics(&output, project);
    let success = output.status.success()
        && diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != 1);

    if !output.status.success() && diagnostics.is_empty() {
        return Err(CorsaError::CorsaExecution {
            exit_code: output.status.code().unwrap_or(-1),
            message: output_message(&output),
        });
    }

    Ok(TypeCheckResult {
        exit_code: output.status.code().unwrap_or(if success { 0 } else { 1 }),
        success,
        diagnostics,
    })
}

fn parse_output_diagnostics(output: &Output, project: &VirtualProject) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    #[allow(clippy::disallowed_types)]
    let stdout = std::string::String::from_utf8_lossy(&output.stdout);
    parse_cli_diagnostics(stdout.as_ref(), project, &mut diagnostics);
    #[allow(clippy::disallowed_types)]
    let stderr = std::string::String::from_utf8_lossy(&output.stderr);
    parse_cli_diagnostics(stderr.as_ref(), project, &mut diagnostics);
    diagnostics
}

fn parse_cli_diagnostics(
    output: &str,
    project: &VirtualProject,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for line in output.lines() {
        if let Some(diagnostic) = parse_cli_diagnostic_line(line, project) {
            diagnostics.push(diagnostic);
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

fn parse_cli_diagnostic_line(line: &str, project: &VirtualProject) -> Option<Diagnostic> {
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
    if should_skip_diagnostic(code) {
        return None;
    }

    let virtual_path = normalize_cli_path(path, project.virtual_root());
    let original = project.map_to_original(&virtual_path, line, column)?;
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
    use super::parse_cli_diagnostics;
    use crate::batch::VirtualProject;
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
            .join("__agent_only")
            .join("tests")
            .join(&*cstr!(
                "cli-fallback-{name}-{}-{case_id}",
                std::process::id()
            ))
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
        parse_cli_diagnostics(output.as_str(), &project, &mut diagnostics);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].file, source);
        assert_eq!(diagnostics[0].line, 0);
        assert_eq!(diagnostics[0].column, 6);
        assert_eq!(diagnostics[0].code, Some(2322));

        let _ = fs::remove_dir_all(&case_dir);
    }
}
