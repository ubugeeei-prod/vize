use super::{
    Decoded, DiagnosticMapper, is_cli_diagnostic_line, is_global_diagnostic_line,
    parse_cli_diagnostic_line,
};
use crate::batch::VirtualProject;
use std::process::Output;

#[cfg(test)]
mod tests;

/// The TS backend emits reachability assertions as errors. Accept its nonzero
/// exit only when *every* raw diagnostic is a verified generated warning;
/// unrelated or unmapped backend errors must still fail the check.
pub(super) fn only_pattern_warnings(output: &Output, project: &VirtualProject) -> bool {
    if !matches!(output.status.code(), Some(1 | 2)) {
        return false;
    }
    let mut found = 0;
    let mut mapper = DiagnosticMapper::new(project);
    for stream in [&output.stdout, &output.stderr] {
        #[expect(clippy::disallowed_types, reason = "from_utf8_lossy yields std Cow")]
        let text = std::string::String::from_utf8_lossy(stream);
        let mut follows_warning = false;
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if is_global_diagnostic_line(line) {
                return false;
            }
            if !is_cli_diagnostic_line(line) {
                if follows_warning && is_warning_continuation(line) {
                    continue;
                }
                if found > 0 && warning_summary_count(line) == Some(found) {
                    follows_warning = false;
                    continue;
                }
                return false;
            }
            if !line.contains("): error TS2322:") {
                return false;
            }
            let Some(Decoded::Raw {
                raw,
                reachability: true,
            }) = parse_cli_diagnostic_line(line, project, &mut mapper)
            else {
                return false;
            };
            if mapper
                .map_to_original(&raw.virtual_path, raw.line, raw.column)
                .is_none()
            {
                return false;
            }
            found += 1;
            follows_warning = true;
        }
    }
    found > 0
}

// Generated reachability assertions only produce assignability diagnostics.
// Do not accept arbitrary indented output: JS/Go stack traces are indented too.
pub(super) fn is_warning_continuation(line: &str) -> bool {
    line.starts_with("  ")
        && line.trim_start().starts_with("Type '")
        && line.contains(" is not assignable to type ")
        && line.ends_with('.')
}

pub(super) fn warning_summary_count(line: &str) -> Option<usize> {
    let count = line.strip_prefix("Found ")?;
    count
        .strip_suffix(" error.")
        .or_else(|| count.strip_suffix(" errors."))?
        .parse()
        .ok()
}
