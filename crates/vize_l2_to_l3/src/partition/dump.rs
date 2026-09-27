//! The partition-fact folio page: the exported static/dynamic facts as a
//! derived, round-trippable stage dump.
//!
//! The facts are computed once during L2→L3 lowering and read by SSR without
//! L3 (P3-3), so they are an artifact of their own rather than a column of
//! the graph-only [`vize_l3::dump::Page`]. Before this page the only
//! printer was a test-local helper in the TS-17 snapshot suite; the page is
//! now the one spelling every consumer (snapshots, the Spolvero feed) shares.

use alloc::vec::Vec;
use core::fmt;
use core::str::SplitWhitespace;

use vize_davinci::dump::value::DumpValue;
use vize_davinci::dump::{Dump, Error as DumpError};
use vize_l0::{Span, cstr};

use super::{PartitionFact, PartitionFacts, PartitionKind};

/// Owned folio page for the exported partition facts, `[s3-partition-folio]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Dump)]
#[dump(name = "s3-partition-folio")]
pub struct Page {
    /// One record per canonical L3 op, in export order.
    pub ops: Vec<Fact>,
}

impl Page {
    /// Mirror live arena facts into the owned page.
    #[must_use]
    pub fn of(facts: &PartitionFacts<'_>) -> Self {
        Self {
            ops: facts.ops.iter().map(Fact::from).collect(),
        }
    }
}

/// One `op=<n> kind=<static|dynamic> span=<start>:<end>` record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fact {
    pub op: u32,
    pub kind: PartitionKind,
    pub span: Span,
}

impl From<&PartitionFact> for Fact {
    fn from(fact: &PartitionFact) -> Self {
        Self {
            op: fact.op.index(),
            kind: fact.kind,
            span: fact.span,
        }
    }
}

impl DumpValue for Fact {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(
            w,
            "op={} kind={} span={}:{}",
            self.op,
            self.kind.as_str(),
            self.span.start,
            self.span.end
        )
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, DumpError> {
        let mut fields = text.split_whitespace();
        let op = parse_u32(field(&mut fields, "op", line)?, "op", line)?;
        let kind_text = field(&mut fields, "kind", line)?;
        let kind = PartitionKind::from_str(kind_text)
            .ok_or_else(|| DumpError::new(line, cstr!("unknown partition kind `{kind_text}`")))?;
        let span_text = field(&mut fields, "span", line)?;
        let Some((start, end)) = span_text.split_once(':') else {
            return Err(DumpError::new(line, cstr!("invalid span `{span_text}`")));
        };
        let span = Span::new(
            parse_u32(start, "span.start", line)?,
            parse_u32(end, "span.end", line)?,
        );
        if let Some(extra) = fields.next() {
            return Err(DumpError::new(line, cstr!("unexpected field `{extra}`")));
        }
        Ok(Self { op, kind, span })
    }
}

fn field<'a>(
    fields: &mut SplitWhitespace<'a>,
    name: &str,
    line: usize,
) -> Result<&'a str, DumpError> {
    let Some(raw) = fields.next() else {
        return Err(DumpError::new(line, cstr!("missing `{name}` field")));
    };
    raw.strip_prefix(name)
        .and_then(|rest| rest.strip_prefix('='))
        .ok_or_else(|| DumpError::new(line, cstr!("expected `{name}=...`, got `{raw}`")))
}

fn parse_u32(text: &str, name: &str, line: usize) -> Result<u32, DumpError> {
    text.parse()
        .map_err(|_| DumpError::new(line, cstr!("invalid `{name}` integer `{text}`")))
}
