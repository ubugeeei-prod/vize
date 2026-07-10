use std::path::Path;

use crate::batch::{Diagnostic, VirtualProject};

use super::super::diagnostics::should_skip_diagnostic;

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
    if should_skip_diagnostic(Some(code), message) {
        return None;
    }

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
    let Ok(relative) = path.strip_prefix(project.virtual_root()) else {
        return false;
    };
    if relative.components().count() != 1 {
        return false;
    }

    relative
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name == "tsconfig.json"
                || name == "tsconfig.declaration.json"
                || (name.starts_with("tsconfig.shard") && name.ends_with(".json"))
        })
}
