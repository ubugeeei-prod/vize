use super::super::diagnostics::{
    DiagnosticMapper, should_skip_diagnostic, should_skip_original_diagnostic,
};
use super::{normalize_cli_path, project_diagnostics};
use crate::batch::{Diagnostic, VirtualProject};

pub(super) fn parse_cli_diagnostic_line(
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
    if code == Some(2322) && mapper.is_keyof_indexed_assignment(&virtual_path, line, column) {
        return None;
    }
    if let Some(diagnostic) =
        project_diagnostics::config(&virtual_path, project, message, code, severity)
    {
        return Some(diagnostic);
    }
    let original = mapper.map_to_original(&virtual_path, line, column)?;
    if should_skip_original_diagnostic(code, &original) {
        return None;
    }

    Some(Diagnostic {
        message: mapper.devirtualized_module_message(&original, message.into()),
        line: original.line,
        column: original.column,
        file: original.path,
        code,
        severity: if mapper.is_unreachable_pattern(&virtual_path, line, column, code) {
            2
        } else {
            severity
        },
        block_type: original.block_type,
    })
}
