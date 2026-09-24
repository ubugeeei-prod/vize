//! The `[remarks-corpus]` page: TS-32's committed baseline.
//!
//! One document per corpus: the files swept (so a file silently dropping
//! out of the sweep is a diff, not an absence), every remark keyed by its
//! file, and the **explained regressions** — the `applied → missed`
//! transitions a reviewer accepted, each with its reason.
//!
//! ```text
//! [remarks-corpus]
//!
//! [remarks-corpus.files]
//! "tests/_fixtures/a.vue"
//!
//! [remarks-corpus.entries]
//! "tests/_fixtures/a.vue" s2.hoist-static applied static-subtree @27:59 tag="h1"
//!
//! [remarks-corpus.explained]
//! "tests/_fixtures/a.vue" s2.hoist-static missed static-props @60:99 reason="..."
//!
//! ```
//!
//! Paths are JSON string literals; an entry is a quoted path, a space, and
//! a `[remarks]` entry line. An explained line has the shape of the missed
//! entry it accepts with exactly one argument, `reason`. Sections print in
//! this order, empty sections are omitted, and lists keep their order (the
//! producer sorts paths and emits entries in canonical per-file order).

use alloc::vec::Vec;
use core::fmt;

use vize_s0::{Span, String, cstr};

use super::super::feed::push_json_string;
use super::super::page::{LineEvent, ParseState};
use super::super::{Folio, FolioError, FolioMode};
use super::{entry_line, parse::parse_string, parse_entry_line};
use crate::pass::observer::{RecordedArg, RecordedRemark, RemarkArgValue, RemarkKind};

/// One remark in its file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusRemark {
    /// Corpus-relative path, `/`-separated.
    pub path: String,
    /// The remark, span in file byte offsets.
    pub remark: RecordedRemark,
}

/// An accepted `applied → missed` transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainedRegression {
    /// The file.
    pub path: String,
    /// Emitting stage.
    pub stage: String,
    /// Emitting pass.
    pub pass: String,
    /// Remark name.
    pub name: String,
    /// The remark's span.
    pub span: Span,
    /// Why the lost optimization is acceptable.
    pub reason: String,
}

impl ExplainedRegression {
    /// Whether this explanation names `remark` in `path`.
    #[must_use]
    pub fn names(&self, path: &str, remark: &RecordedRemark) -> bool {
        self.path.as_str() == path
            && self.stage == remark.stage
            && self.pass == remark.pass
            && self.name == remark.name
            && self.span == remark.span
    }
}

/// TS-32's baseline document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemarkCorpus {
    /// Every file swept, in sweep order.
    pub files: Vec<String>,
    /// Every remark, by file.
    pub entries: Vec<CorpusRemark>,
    /// Accepted regressions.
    pub explained: Vec<ExplainedRegression>,
}

fn quoted(path: &str) -> String {
    let mut out = String::default();
    push_json_string(&mut out, path);
    out
}

impl Folio for RemarkCorpus {
    /// Both modes print the canonical text: a baseline is never elided.
    fn print<W: fmt::Write>(&self, w: &mut W, _mode: FolioMode) -> fmt::Result {
        w.write_str("[remarks-corpus]\n\n")?;
        if !self.files.is_empty() {
            w.write_str("[remarks-corpus.files]\n")?;
            for path in &self.files {
                writeln!(w, "{}", quoted(path))?;
            }
            w.write_char('\n')?;
        }
        if !self.entries.is_empty() {
            w.write_str("[remarks-corpus.entries]\n")?;
            for entry in &self.entries {
                writeln!(w, "{} {}", quoted(&entry.path), entry_line(&entry.remark))?;
            }
            w.write_char('\n')?;
        }
        if !self.explained.is_empty() {
            w.write_str("[remarks-corpus.explained]\n")?;
            for explained in &self.explained {
                let remark = RecordedRemark {
                    stage: explained.stage.clone(),
                    pass: explained.pass.clone(),
                    kind: RemarkKind::Missed,
                    name: explained.name.clone(),
                    span: explained.span,
                    args: alloc::vec![RecordedArg {
                        key: String::from("reason"),
                        value: RemarkArgValue::Str(explained.reason.clone()),
                    }],
                };
                writeln!(w, "{} {}", quoted(&explained.path), entry_line(&remark))?;
            }
            w.write_char('\n')?;
        }
        Ok(())
    }

    fn parse(input: &str) -> Result<Self, FolioError> {
        const FILES: usize = 0;
        const ENTRIES: usize = 1;
        const EXPLAINED: usize = 2;
        let mut state = ParseState::new("remarks-corpus");
        let mut corpus = Self::default();
        for (index, line) in input.lines().enumerate() {
            let line_no = index + 1;
            match state.classify(line, line_no)? {
                LineEvent::Skip => {}
                LineEvent::Field => {
                    let name = line.split_once('=').map_or(line, |(name, _)| name);
                    return Err(super::super::page::unknown_field(name, line_no));
                }
                LineEvent::Section(name) => {
                    let index = match name {
                        "files" => FILES,
                        "entries" => ENTRIES,
                        "explained" => EXPLAINED,
                        other => return Err(state.unknown_section(other, line_no)),
                    };
                    state.enter_section(index, name, line_no)?;
                }
                LineEvent::Entry(FILES) => {
                    let (path, rest) = path_prefix(line, line_no)?;
                    if !rest.is_empty() {
                        return Err(FolioError::new(
                            line_no,
                            cstr!("unexpected `{rest}` after a file path"),
                        ));
                    }
                    corpus.files.push(path);
                }
                LineEvent::Entry(ENTRIES) => {
                    let (path, rest) = path_prefix(line, line_no)?;
                    let remark = parse_entry_line(rest, line_no)?;
                    corpus.entries.push(CorpusRemark { path, remark });
                }
                LineEvent::Entry(_) => {
                    let (path, rest) = path_prefix(line, line_no)?;
                    corpus.explained.push(explained(path, rest, line_no)?);
                }
            }
        }
        state.require_header()?;
        Ok(corpus)
    }
}

/// Split a line into its quoted path and the text after one space.
fn path_prefix(line: &str, line_no: usize) -> Result<(String, &str), FolioError> {
    if !line.starts_with('"') {
        return Err(FolioError::new(
            line_no,
            cstr!("corpus line `{line}` does not start with a quoted path"),
        ));
    }
    let (path, consumed) = parse_string(line, line_no)?;
    let rest = line.get(consumed..).unwrap_or_default();
    Ok((path, rest.strip_prefix(' ').unwrap_or(rest)))
}

fn explained(path: String, rest: &str, line_no: usize) -> Result<ExplainedRegression, FolioError> {
    let remark = parse_entry_line(rest, line_no)?;
    let reason = match remark.args.as_slice() {
        [
            RecordedArg {
                key,
                value: RemarkArgValue::Str(reason),
            },
        ] if key.as_str() == "reason" && remark.kind == RemarkKind::Missed => reason.clone(),
        _ => {
            return Err(FolioError::new(
                line_no,
                cstr!("an explained regression is a missed entry with exactly `reason=\"...\"`"),
            ));
        }
    };
    Ok(ExplainedRegression {
        path,
        stage: remark.stage,
        pass: remark.pass,
        name: remark.name,
        span: remark.span,
        reason,
    })
}
