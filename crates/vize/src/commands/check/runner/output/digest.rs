//! `vize check` text output: template diagnostics the hint table knows
//! ([`crate::render::hints`]) print as one hinted block per kind instead of
//! one line each. `--verbose`, `--format json` and every other diagnostic keep
//! the per-file listing, and the totals and exit code count every diagnostic.

use std::{fs, path::Path};

use vize_canon::{BatchDiagnostic, SfcBlockType};
use vize_l0::String;
use vize_l0::diag::Severity;

use crate::render::digest::{DigestOptions, Finding, Representative, attribute_end, render_digest};
use crate::render::hints::{HINTS, Hint, for_template_type_diagnostic};
use crate::render::{EnglishCatalog, SourceFile};

use super::super::display_path;

/// Diagnostics of one hinted kind.
pub(super) struct Digest<'d> {
    hint: &'static Hint,
    diagnostics: Vec<&'d BatchDiagnostic>,
}

/// Split `diagnostics` into the per-file listing and the hinted digests, in
/// hint-table order.
pub(super) fn partition(
    diagnostics: &[BatchDiagnostic],
) -> (Vec<BatchDiagnostic>, Vec<Digest<'_>>) {
    let mut listed = Vec::new();
    let mut digests: Vec<Digest<'_>> = Vec::new();
    for diagnostic in diagnostics {
        let hint = (diagnostic.block_type == Some(SfcBlockType::Template))
            .then(|| for_template_type_diagnostic(diagnostic.code, &diagnostic.message))
            .flatten();
        match hint {
            Some(hint) => match digests.iter_mut().find(|digest| digest.hint.id == hint.id) {
                Some(digest) => digest.diagnostics.push(diagnostic),
                None => digests.push(Digest {
                    hint,
                    diagnostics: vec![diagnostic],
                }),
            },
            None => listed.push(diagnostic.clone()),
        }
    }
    digests.sort_by_key(|digest| HINTS.iter().position(|entry| entry.id == digest.hint.id));
    (listed, digests)
}

/// Render every digest, each after a blank line.
pub(super) fn render(cwd: &Path, digests: &mut [Digest<'_>], color: bool) -> String {
    let mut out = String::default();
    for digest in digests {
        digest.diagnostics.sort_by(|left, right| {
            (&left.file, left.line, left.column).cmp(&(&right.file, right.line, right.column))
        });
        let mut findings = Vec::with_capacity(digest.diagnostics.len());
        let mut first_text: Option<(String, String)> = None;
        for file_diagnostics in digest
            .diagnostics
            .chunk_by(|left, right| left.file == right.file)
        {
            let Some(&head) = file_diagnostics.first() else {
                continue;
            };
            let path = display_path(cwd, &head.file);
            let text = fs::read_to_string(&head.file).ok();
            let file = text.as_deref().map(|text| SourceFile::new(&path, text));
            for diagnostic in file_diagnostics {
                let subject = file.as_ref().and_then(|file| {
                    let start = file.offset_of(diagnostic.line, diagnostic.column)?;
                    file.text()
                        .get(start..attribute_end(file.text(), start))
                        .filter(|subject| !subject.is_empty())
                });
                findings.push(Finding {
                    path: path.clone(),
                    line: diagnostic.line + 1,
                    column: diagnostic.column + 1,
                    severity: severity(diagnostic.severity),
                    subject: subject.map(String::from),
                });
            }
            if findings.len() == file_diagnostics.len()
                && let Some(text) = text
            {
                first_text = Some((path, String::from(text)));
            }
        }
        // Diagnostics are sorted by absolute path and findings print by
        // display path; both orders agree within one project, and the frame
        // shows the first file the locations list.
        let file = first_text
            .as_ref()
            .map(|(path, text)| SourceFile::new(path, text));
        let representative = file.as_ref().and_then(|file| {
            let first = digest.diagnostics.first()?;
            let start = file.offset_of(first.line, first.column)?;
            Some(Representative {
                file,
                start,
                end: attribute_end(file.text(), start),
                subject: None,
            })
        });
        out.push('\n');
        render_digest(
            &mut out,
            &EnglishCatalog,
            digest.hint,
            &findings,
            representative.as_ref(),
            DigestOptions {
                color,
                ..DigestOptions::default()
            },
        );
    }
    out
}

fn severity(severity: u8) -> Severity {
    match severity {
        1 => Severity::Error,
        2 => Severity::Warning,
        3 => Severity::Info,
        _ => Severity::Hint,
    }
}
