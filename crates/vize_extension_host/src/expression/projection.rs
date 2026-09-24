//! The projection page (`projection-page@1`): a dialect's checkable
//! projection and its span links, the P4-5a `ProjectionMapping` rows
//! serialized.
//!
//! ```text
//! [projection]
//! rows=<count>
//!
//! [projection.rows]
//! row <gen-start>:<gen-end> <src-start>:<src-end> <kind> <features>
//!   sub <gen-start>:<gen-end> <src-start>:<src-end>
//!
//! [projection.text]
//! <the generated text, verbatim to the end of the page>
//! ```
//!
//! Generated ranges index the text; authored ranges are file-absolute (a
//! `ProjectionMapping` with `authored_base` 0). `<kind>` is a
//! `ProjectionSpanKind` in kebab case and `<features>` is `all`, `none` or
//! the enabled `ProjectionFeatures` names comma-joined in their bit order.
//! The text section is terminal and verbatim, so any generated text —
//! newlines included — round-trips byte for byte.

use core::fmt::{Result as FmtResult, Write};

use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_s0::{String, cstr};

/// `ProjectionSpanKind`, in row order of the enum.
pub const KINDS: &[&str] = &[
    "unknown",
    "script",
    "interpolation",
    "directive-expr",
    "directive-arg",
    "event-handler",
    "v-for-var",
    "slot-binding",
    "component-ref",
    "template-expression",
];

/// `ProjectionFeatures::NAMED`, in bit order.
pub const FEATURES: &[&str] = &[
    "hover",
    "completion",
    "signature-help",
    "definition",
    "references",
    "rename",
    "diagnostics",
    "semantic-tokens",
];

/// A half-open byte range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}

/// One span link: `VizeMapping` plus its `ProjectionMeta`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionRow {
    pub generated: Range,
    pub authored: Range,
    /// Index into [`KINDS`].
    pub kind: u8,
    /// `ProjectionFeatures` bits.
    pub features: u8,
    /// `VizeSubSpan`s: exact sub-links inside the row.
    pub sub_spans: Vec<(Range, Range)>,
}

/// One projection: the generated text and its rows in producer order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectionPage {
    pub rows: Vec<ProjectionRow>,
    pub text: String,
}

impl Folio for ProjectionPage {
    fn print<W: Write>(&self, w: &mut W, _mode: FolioMode) -> FmtResult {
        writeln!(w, "[projection]\nrows={}\n", self.rows.len())?;
        if !self.rows.is_empty() {
            writeln!(w, "[projection.rows]")?;
            for row in &self.rows {
                let pair = |r: Range| cstr!("{}:{}", r.start, r.end);
                let kind = KINDS
                    .get(usize::from(row.kind))
                    .copied()
                    .unwrap_or("unknown");
                write!(
                    w,
                    "row {} {} {kind} ",
                    pair(row.generated),
                    pair(row.authored)
                )?;
                write_features(w, row.features)?;
                writeln!(w)?;
                for (generated, authored) in &row.sub_spans {
                    writeln!(w, "  sub {} {}", pair(*generated), pair(*authored))?;
                }
            }
            writeln!(w)?;
        }
        write!(w, "[projection.text]\n{}", self.text)
    }

    fn parse(input: &str) -> Result<Self, FolioError> {
        let err = |line: usize, message: String| FolioError::new(line, message);
        let Some((head, text)) = input.split_once("[projection.text]\n") else {
            return Err(err(0, cstr!("missing section [projection.text]")));
        };
        let mut page = ProjectionPage {
            rows: Vec::new(),
            text: String::from(text),
        };
        let mut section = "";
        for (index, line) in head.split('\n').enumerate() {
            let no = index + 1;
            match (section, line) {
                (_, "") => {}
                ("", "[projection]") => section = "header",
                ("", _) => return Err(err(no, cstr!("first section must be [projection]"))),
                ("header", "[projection.rows]") => section = "rows",
                ("header", _) => {
                    let valid = line
                        .strip_prefix("rows=")
                        .is_some_and(|n| n.parse::<u32>().is_ok());
                    if !valid {
                        return Err(err(no, cstr!("expected `rows=<count>`, found `{line}`")));
                    }
                }
                (_, _) => {
                    if let Some(rest) = line.strip_prefix("  sub ") {
                        let Some(row) = page.rows.last_mut() else {
                            return Err(err(no, cstr!("`sub` before any `row`")));
                        };
                        let [generated, authored] = ranges(rest, no)?;
                        row.sub_spans.push((generated, authored));
                    } else if let Some(rest) = line.strip_prefix("row ") {
                        page.rows.push(row(rest, no)?);
                    } else {
                        return Err(err(no, cstr!("expected `row` or `  sub`, found `{line}`")));
                    }
                }
            }
        }
        Ok(page)
    }
}

fn write_features<W: Write>(w: &mut W, bits: u8) -> FmtResult {
    match bits {
        u8::MAX => w.write_str("all"),
        0 => w.write_str("none"),
        _ => {
            let names: Vec<&str> = FEATURES
                .iter()
                .enumerate()
                .filter(|(bit, _)| bits & (1 << bit) != 0)
                .map(|(_, name)| *name)
                .collect();
            w.write_str(&names.join(","))
        }
    }
}

fn range(text: &str, no: usize) -> Result<Range, FolioError> {
    let bad = || FolioError::new(no, cstr!("invalid range `{text}`"));
    let (start, end) = text.split_once(':').ok_or_else(bad)?;
    let (start, end) = (
        start.parse().map_err(|_| bad())?,
        end.parse().map_err(|_| bad())?,
    );
    if start > end {
        return Err(bad());
    }
    Ok(Range { start, end })
}

fn ranges(text: &str, no: usize) -> Result<[Range; 2], FolioError> {
    let mut parts = text.split(' ');
    let generated = range(parts.next().unwrap_or(""), no)?;
    let authored = range(parts.next().unwrap_or(""), no)?;
    if parts.next().is_some() {
        return Err(FolioError::new(no, cstr!("a sub-span is two ranges")));
    }
    Ok([generated, authored])
}

fn row(text: &str, no: usize) -> Result<ProjectionRow, FolioError> {
    let fields: Vec<&str> = text.split(' ').collect();
    let [generated, authored, kind, features] = fields.as_slice() else {
        return Err(FolioError::new(
            no,
            cstr!("a row is `gen src kind features`"),
        ));
    };
    let kind = KINDS
        .iter()
        .position(|known| known == kind)
        .ok_or_else(|| FolioError::new(no, cstr!("unknown kind `{kind}`")))?;
    let features = match *features {
        "all" => u8::MAX,
        "none" => 0,
        list => list.split(',').try_fold(0u8, |bits, name| {
            let bit = FEATURES
                .iter()
                .position(|known| *known == name)
                .ok_or_else(|| FolioError::new(no, cstr!("unknown feature `{name}`")))?;
            Ok::<u8, FolioError>(bits | 1 << bit)
        })?,
    };
    Ok(ProjectionRow {
        generated: range(generated, no)?,
        authored: range(authored, no)?,
        kind: kind as u8,
        features,
        sub_spans: Vec::new(),
    })
}
