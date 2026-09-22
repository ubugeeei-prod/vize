use super::super::diagnostics::{DiagnosticMapper, RawDiagnostic};
use super::{normalize_cli_path, project_diagnostics};
use crate::batch::{Diagnostic, VirtualProject};

/// One diagnostic line of `--pretty false` output, decoded but not assembled.
pub(super) enum Decoded {
    /// A project-level diagnostic (the generated tsconfig's own findings),
    /// attributed to the authored configuration.
    Project(Diagnostic),
    /// A diagnostic in a file the checker saw; `reachability` marks a
    /// `TS2322` on a generated reachability assertion, whose continuation
    /// lines the output interleaves with warning summaries.
    Raw {
        raw: RawDiagnostic,
        reachability: bool,
    },
}

pub(super) fn parse_cli_diagnostic_line(
    line: &str,
    project: &VirtualProject,
    mapper: &mut DiagnosticMapper<'_>,
) -> Option<Decoded> {
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

    let virtual_path = normalize_cli_path(path, project.virtual_root());
    if let Some(diagnostic) =
        project_diagnostics::config(&virtual_path, project, message, code, severity)
    {
        return mapper
            .is_reportable_project_diagnostic(code, message)
            .then_some(Decoded::Project(diagnostic));
    }
    let reachability = code == Some(2322)
        && (severity == 2 || mapper.is_unreachable_pattern(&virtual_path, line, column, code));
    Some(Decoded::Raw {
        raw: RawDiagnostic {
            virtual_path,
            line,
            column,
            end: None,
            code,
            severity,
            message: message.into(),
        },
        reachability,
    })
}
