use std::path::Path;

use crate::batch::{Diagnostic, VirtualProject};

pub(super) fn global(line: &str, project: &VirtualProject) -> Option<Diagnostic> {
    let (severity, rest) = line.split_once(' ')?;
    let severity = match severity {
        "error" => 1,
        "warning" => 2,
        "info" => 3,
        _ => return None,
    };
    let (code, message) = rest.split_once(": ")?;
    let code = code.strip_prefix("TS")?.parse::<u32>().ok()?;

    Some(Diagnostic {
        file: project.project_diagnostics_anchor(),
        line: 0,
        column: 0,
        message: message.into(),
        code: Some(code),
        severity,
        block_type: None,
    })
}

pub(super) fn config(
    path: &Path,
    project: &VirtualProject,
    message: &str,
    code: Option<u32>,
    severity: u8,
) -> Option<Diagnostic> {
    if !is_project_config_path(path, project) {
        return None;
    }

    Some(Diagnostic {
        file: project.project_diagnostics_anchor(),
        line: 0,
        column: 0,
        message: message.into(),
        code,
        severity,
        block_type: None,
    })
}

fn is_project_config_path(path: &Path, project: &VirtualProject) -> bool {
    if path.parent() != project.generated_tsconfig_path().parent() {
        return false;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name == "tsconfig.json"
                || name == "tsconfig.declaration.json"
                || (name.starts_with("tsconfig.shard") && name.ends_with(".json"))
        })
}

/// TS6053 for each `extends` entry the chain skipped, at the entry itself, the
/// way `tsc -p` reports it before checking the rest of the program.
pub(in crate::batch::executor) fn unresolved_extends(project: &VirtualProject) -> Vec<Diagnostic> {
    project
        .unresolved_tsconfig_extends()
        .into_iter()
        .map(|unresolved| {
            let quoted = vize_carton::cstr!("\"{}\"", unresolved.specifier);
            let (line, column) = std::fs::read_to_string(&unresolved.config)
                .ok()
                .and_then(|content| {
                    let offset = content.find(quoted.as_str())?;
                    let before = content.get(..offset)?;
                    let line = before.matches('\n').count();
                    let column = offset - before.rfind('\n').map_or(0, |newline| newline + 1);
                    Some((u32::try_from(line).ok()?, u32::try_from(column).ok()?))
                })
                .unwrap_or((0, 0));
            Diagnostic {
                file: unresolved.config.clone(),
                line,
                column,
                message: unresolved.message().as_str().into(),
                code: Some(6053),
                severity: 1,
                block_type: None,
            }
        })
        .collect()
}
