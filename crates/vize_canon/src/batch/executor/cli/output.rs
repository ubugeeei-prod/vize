//! Decoding one `--pretty false` run and handing it to the assembly pass.

use std::path::Path;
use std::process::Output;

use super::file_diagnostics::Decoded;
use super::{
    is_cli_diagnostic_line, is_global_diagnostic_line, parse_cli_diagnostic_line, patterns,
    project_diagnostics,
};
use crate::batch::executor::diagnostics::{
    DiagnosticMapper, dedup_diagnostics, restore_authored_paths_in_messages,
};
use crate::batch::{Diagnostic, VirtualProject};

pub(super) fn parse_output_diagnostics(
    output: &Output,
    project: &VirtualProject,
    owns: &dyn Fn(&Path) -> bool,
) -> Vec<Diagnostic> {
    let mut decoded = Vec::new();
    let mut mapper = DiagnosticMapper::new(project);
    #[allow(clippy::disallowed_types)]
    let stdout = std::string::String::from_utf8_lossy(&output.stdout);
    decode_cli_diagnostics(stdout.as_ref(), project, &mut mapper, &mut decoded);
    #[allow(clippy::disallowed_types)]
    let stderr = std::string::String::from_utf8_lossy(&output.stderr);
    decode_cli_diagnostics(stderr.as_ref(), project, &mut mapper, &mut decoded);
    let diagnostics = assemble_decoded(decoded, &mut mapper, owns);
    // A single template error surfaces twice — the dynamic prop binding it sits
    // on is generated at two virtual positions that map back to the same source
    // attribute span (#1389). Collapse exact duplicates at the collection point.
    dedup_diagnostics(restore_authored_paths_in_messages(diagnostics, project))
}

/// Every stream of one run is decoded before the one assembly pass runs, so
/// template directives see the file's complete diagnostic set.
fn assemble_decoded(
    decoded: Vec<Decoded>,
    mapper: &mut DiagnosticMapper<'_>,
    owns: &dyn Fn(&Path) -> bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut raws = Vec::with_capacity(decoded.len());
    for entry in decoded {
        match entry {
            Decoded::Project(diagnostic) => diagnostics.push(diagnostic),
            Decoded::Raw { raw, .. } => raws.push(raw),
        }
    }
    diagnostics.extend(mapper.assemble(raws, owns));
    diagnostics
}

#[cfg(test)]
pub(super) fn parse_cli_diagnostics(
    output: &str,
    project: &VirtualProject,
    mapper: &mut DiagnosticMapper<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut decoded = Vec::new();
    decode_cli_diagnostics(output, project, mapper, &mut decoded);
    diagnostics.extend(assemble_decoded(decoded, mapper, &|_| true));
}

fn decode_cli_diagnostics(
    output: &str,
    project: &VirtualProject,
    mapper: &mut DiagnosticMapper<'_>,
    decoded: &mut Vec<Decoded>,
) {
    let mut last_was_kept = false;
    for line in output.lines() {
        // Project-level diagnostics carry no file position (`error TS2688:
        // Cannot find type definition file for 'x'.`). They are real,
        // user-actionable problems — tsc and vue-tsc report them and the
        // runtime may skip the semantic pass because of them — so they are
        // attributed to the project's tsconfig instead of being dropped.
        let entry = parse_cli_diagnostic_line(line, project, mapper).or_else(|| {
            project_diagnostics::global(line, project)
                .filter(|diagnostic| {
                    mapper.is_reportable_project_diagnostic(diagnostic.code, &diagnostic.message)
                })
                .map(Decoded::Project)
        });
        if let Some(entry) = entry {
            decoded.push(entry);
            last_was_kept = true;
            continue;
        }
        if is_cli_diagnostic_line(line) || is_global_diagnostic_line(line) {
            last_was_kept = false;
            continue;
        }
        let Some(last) = decoded.last_mut().filter(|_| last_was_kept) else {
            continue;
        };
        let reachability = match last {
            Decoded::Project(diagnostic) => {
                diagnostic.severity == 2 && diagnostic.code == Some(2322)
            }
            Decoded::Raw { reachability, .. } => *reachability,
        };
        if reachability && !line.trim().is_empty() {
            if patterns::warning_summary_count(line).is_some() {
                last_was_kept = false;
                continue;
            }
            if !patterns::is_warning_continuation(line) {
                // The assembly pass reads this elaboration line and keeps the
                // checker's error severity; a project diagnostic has no pass.
                if let Decoded::Project(diagnostic) = last {
                    diagnostic.severity = 1;
                }
                last_was_kept = false;
            }
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let message = match last {
            Decoded::Project(diagnostic) => &mut diagnostic.message,
            Decoded::Raw { raw, .. } => &mut raw.message,
        };
        message.push('\n');
        message.push_str(line);
    }
}
