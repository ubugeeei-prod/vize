use super::{
    DiagnosticMapper, is_cli_diagnostic_line, is_global_diagnostic_line, parse_cli_diagnostic_line,
};
use crate::batch::VirtualProject;
use std::process::Output;

/// The TS backend emits reachability assertions as errors. Accept its nonzero
/// exit only when *every* raw diagnostic is a verified generated warning;
/// unrelated or unmapped backend errors must still fail the check.
pub(super) fn only_pattern_warnings(output: &Output, project: &VirtualProject) -> bool {
    if !matches!(output.status.code(), Some(1 | 2)) {
        return false;
    }
    let mut found = false;
    let mut mapper = DiagnosticMapper::new(project);
    for stream in [&output.stdout, &output.stderr] {
        #[allow(clippy::disallowed_types)]
        let text = std::string::String::from_utf8_lossy(stream);
        for line in text.lines() {
            if is_global_diagnostic_line(line) {
                return false;
            }
            if !is_cli_diagnostic_line(line) {
                continue;
            }
            if !line.contains("): error TS2322:") {
                return false;
            }
            let Some(diagnostic) = parse_cli_diagnostic_line(line, project, &mut mapper) else {
                return false;
            };
            if diagnostic.severity != 2 {
                return false;
            }
            found = true;
        }
    }
    found
}
