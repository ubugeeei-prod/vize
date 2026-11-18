//! `vize check` text output: template diagnostics the hint table knows
//! ([`crate::render::hints`]) print as one hinted block per kind instead of
//! one line each. `--verbose`, `--format json` and every other diagnostic keep
//! the per-file listing, and the totals and exit code count every diagnostic.

use std::{fs, path::Path};

use vize_canon::{BatchDiagnostic, SfcBlockType};
use vize_l0::String;
use vize_l0::diag::Severity;
use vize_l0::line_index::LineIndex;

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
        // The first file's path and text, and its first finding's start.
        let mut first: Option<(String, String, usize)> = None;
        for (index, file_diagnostics) in digest
            .diagnostics
            .chunk_by(|left, right| left.file == right.file)
            .enumerate()
        {
            let Some(&head) = file_diagnostics.first() else {
                continue;
            };
            let path = display_path(cwd, &head.file);
            let text = fs::read_to_string(&head.file).ok();
            let positions = text.as_deref().and_then(Positions::new);
            let mut starts = Vec::with_capacity(file_diagnostics.len());
            for diagnostic in file_diagnostics {
                let start = positions
                    .as_ref()
                    .and_then(|positions| positions.offset(diagnostic.line, diagnostic.column));
                let subject = start.zip(text.as_deref()).and_then(|(start, text)| {
                    text.get(start..attribute_end(text, start))
                        .filter(|subject| !subject.is_empty())
                });
                starts.push(start);
                findings.push(Finding {
                    path: path.clone(),
                    line: diagnostic.line + 1,
                    column: diagnostic.column + 1,
                    severity: severity(diagnostic.severity),
                    subject: subject.map(String::from),
                });
            }
            if index == 0
                && let (Some(text), Some(Some(start))) = (text, starts.first().copied())
            {
                first = Some((path, String::from(text), start));
            }
        }
        // Diagnostics are sorted by absolute path and findings print by
        // display path; both orders agree within one project, and the frame
        // shows the first file the locations list.
        let file = first
            .as_ref()
            .map(|(path, text, _)| SourceFile::new(path, text));
        let representative = file
            .as_ref()
            .zip(first.as_ref())
            .map(|(file, (.., start))| Representative {
                file,
                start: *start,
                end: attribute_end(file.text(), *start),
                subject: None,
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

/// Byte offsets of checker positions in one file, available only where every
/// line-break convention in play agrees on them: TypeScript counts LS, PS and
/// a lone CR as breaks, the LSP a lone CR, and the frame renderer LF alone
/// (CRLF is one break in all three). A file with any of the others keeps its
/// listed `line:column` but shows no subject or frame rather than one from
/// another line. Built once per file.
struct Positions<'a> {
    index: LineIndex<'a>,
}

impl<'a> Positions<'a> {
    fn new(text: &'a str) -> Option<Self> {
        let lone_cr = text
            .match_indices('\r')
            .any(|(at, _)| text.as_bytes().get(at + 1) != Some(&b'\n'));
        if lone_cr || text.contains(['\u{2028}', '\u{2029}']) {
            return None;
        }
        Some(Self {
            index: LineIndex::new(text),
        })
    }

    /// The offset of a zero-based line and UTF-16 column.
    fn offset(&self, line: u32, column: u32) -> Option<usize> {
        self.index.line_col_to_offset(line, column)
    }
}

fn severity(severity: u8) -> Severity {
    match severity {
        1 => Severity::Error,
        2 => Severity::Warning,
        3 => Severity::Info,
        _ => Severity::Hint,
    }
}
