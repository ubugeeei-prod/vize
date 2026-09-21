//! The `[remarks]` page and the remark JSON document (P3-13).
//!
//! Both are serializations of one [`RemarkLog`] - the canonical-order list a
//! [`RemarkCollector`](crate::pass::RemarkCollector) finishes into - so the
//! text a snapshot or a corpus baseline pins and the JSON Spolvero and agents
//! read cannot disagree. The grammar and the per-pass vocabulary are
//! `davinci-road/plan/remarks-format.md`; the JSON shape is committed as
//! `davinci-road/plan/remarks.schema.json`.
//!
//! # The page
//!
//! ```text
//! [remarks]
//!
//! [remarks.entries]
//! s2.hoist-static applied static-subtree @27:59 tag="h1"
//! s2.hoist-static missed static-props @60:99 tag="p" blocker="binding" op="ui.on"
//!
//! ```
//!
//! One remark per line: `{stage}.{pass} {kind} {name} @{start}:{end}`, then
//! one ` {key}={value}` per argument in emission order. A text value is a
//! JSON string literal (so it may hold spaces, quotes, anything); integers
//! and booleans are bare. Entry order is carried, never re-sorted (an
//! order-bearing list, folio-format.md rule 1): the collector already put
//! it in canonical order. `Display` drops the spans and carries no
//! round-trip law.

pub mod corpus;
pub mod diff;
mod parse;

use alloc::vec::Vec;
use core::fmt::{self, Write as _};

use vize_s0::String;

use super::feed::push_json_string;
use super::page::{LineEvent, ParseState};
use super::{Folio, FolioError, FolioMode};
use crate::pass::observer::{RecordedRemark, RemarkArgValue, RemarkKind};

/// The remark JSON document's format version. Incompatible shape changes
/// bump this **and** the committed schema's `const` together.
pub const REMARKS_SCHEMA_VERSION: u32 = 1;

/// A consumer-side remark schema negotiation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemarksSchemaMismatch {
    /// The schema this crate knows how to consume.
    pub expected: u32,
    /// The schema the payload presented.
    pub found: u32,
}

/// A run's remarks, in canonical order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemarkLog {
    /// The remarks.
    pub remarks: Vec<RecordedRemark>,
}

impl RemarkLog {
    /// A log over already-ordered remarks.
    #[must_use]
    pub const fn new(remarks: Vec<RecordedRemark>) -> Self {
        Self { remarks }
    }

    /// Negotiate the JSON schema before reading any shape-dependent field.
    ///
    /// # Errors
    ///
    /// Returns the mismatch when `schema_version` is not
    /// [`REMARKS_SCHEMA_VERSION`].
    pub fn negotiate_schema_version(schema_version: u32) -> Result<(), RemarksSchemaMismatch> {
        if schema_version == REMARKS_SCHEMA_VERSION {
            Ok(())
        } else {
            Err(RemarksSchemaMismatch {
                expected: REMARKS_SCHEMA_VERSION,
                found: schema_version,
            })
        }
    }

    /// How many remarks have `kind`.
    #[must_use]
    pub fn count(&self, kind: RemarkKind) -> usize {
        self.remarks
            .iter()
            .filter(|remark| remark.kind == kind)
            .count()
    }

    /// The committed JSON shape: one line, key order `schema_version`,
    /// `command`, `remarks`, trailing newline.
    #[must_use]
    pub fn to_json(&self, command: &str) -> String {
        let mut out = String::default();
        out.push_str("{\"schema_version\":");
        let _ = write!(out, "{REMARKS_SCHEMA_VERSION}");
        out.push_str(",\"command\":");
        push_json_string(&mut out, command);
        out.push_str(",\"remarks\":[");
        for (index, remark) in self.remarks.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            push_remark_json(&mut out, remark);
        }
        out.push_str("]}\n");
        out
    }
}

/// One remark as a JSON object: key order `stage`, `pass`, `kind`, `name`,
/// `span` (`start`, `end`), `args` (`key`, `value` per entry).
pub(crate) fn push_remark_json(out: &mut String, remark: &RecordedRemark) {
    out.push_str("{\"stage\":");
    push_json_string(out, remark.stage.as_str());
    out.push_str(",\"pass\":");
    push_json_string(out, remark.pass.as_str());
    out.push_str(",\"kind\":");
    push_json_string(out, remark.kind.as_str());
    out.push_str(",\"name\":");
    push_json_string(out, remark.name.as_str());
    let _ = write!(
        out,
        ",\"span\":{{\"start\":{},\"end\":{}}},\"args\":[",
        remark.span.start, remark.span.end
    );
    for (index, arg) in remark.args.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str("{\"key\":");
        push_json_string(out, arg.key.as_str());
        out.push_str(",\"value\":");
        match &arg.value {
            RemarkArgValue::Str(text) => push_json_string(out, text.as_str()),
            RemarkArgValue::Int(number) => {
                let _ = write!(out, "{number}");
            }
            RemarkArgValue::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        }
        out.push('}');
    }
    out.push_str("]}");
}

/// One remark as its canonical `[remarks]` entry line, without the newline
/// (the unit the corpus baseline and the remarks-diff report share).
#[must_use]
pub fn entry_line(remark: &RecordedRemark) -> String {
    let mut out = String::default();
    print_entry(&mut out, remark, FolioMode::Full).expect("printing into a string cannot fail");
    out
}

/// Parse one `[remarks]` entry line; `line_no` attributes the error.
///
/// # Errors
///
/// The page parser's entry rejections, verbatim.
pub fn parse_entry_line(line: &str, line_no: usize) -> Result<RecordedRemark, FolioError> {
    parse::entry(line, line_no)
}

/// Print one entry line (without its newline).
fn print_entry<W: fmt::Write>(w: &mut W, remark: &RecordedRemark, mode: FolioMode) -> fmt::Result {
    write!(
        w,
        "{}.{} {} {}",
        remark.stage,
        remark.pass,
        remark.kind.as_str(),
        remark.name
    )?;
    if mode == FolioMode::Full {
        write!(w, " @{}:{}", remark.span.start, remark.span.end)?;
    }
    for arg in &remark.args {
        write!(w, " {}=", arg.key)?;
        match &arg.value {
            RemarkArgValue::Str(text) => {
                let mut quoted = String::default();
                push_json_string(&mut quoted, text.as_str());
                w.write_str(quoted.as_str())?;
            }
            RemarkArgValue::Int(number) => write!(w, "{number}")?,
            RemarkArgValue::Bool(flag) => w.write_str(if *flag { "true" } else { "false" })?,
        }
    }
    Ok(())
}

/// The one section the page declares.
const SECTION_ENTRIES: usize = 0;

impl Folio for RemarkLog {
    fn print<W: fmt::Write>(&self, w: &mut W, mode: FolioMode) -> fmt::Result {
        w.write_str("[remarks]\n\n")?;
        if self.remarks.is_empty() {
            return Ok(());
        }
        w.write_str("[remarks.entries]\n")?;
        for remark in &self.remarks {
            print_entry(w, remark, mode)?;
            w.write_char('\n')?;
        }
        w.write_char('\n')
    }

    fn parse(input: &str) -> Result<Self, FolioError> {
        let mut state = ParseState::new("remarks");
        let mut remarks = Vec::new();
        for (index, line) in input.lines().enumerate() {
            let line_no = index + 1;
            match state.classify(line, line_no)? {
                LineEvent::Skip => {}
                LineEvent::Field => {
                    let name = line.split_once('=').map_or(line, |(name, _)| name);
                    return Err(super::page::unknown_field(name, line_no));
                }
                LineEvent::Section("entries") => {
                    state.enter_section(SECTION_ENTRIES, "entries", line_no)?;
                }
                LineEvent::Section(other) => return Err(state.unknown_section(other, line_no)),
                LineEvent::Entry(_) => remarks.push(parse::entry(line, line_no)?),
            }
        }
        state.require_header()?;
        Ok(Self { remarks })
    }
}
