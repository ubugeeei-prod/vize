//! The partition-fact folio page: the exported static/dynamic facts as a
//! derived, round-trippable stage dump.
//!
//! The facts are computed once during S2→S3 lowering and read by SSR without
//! S3 (P3-3), so they are an artifact of their own rather than a column of
//! the graph-only [`vize_s3::folio::S3Folio`]. Before this page the only
//! printer was a test-local helper in the TS-17 snapshot suite; the page is
//! now the one spelling every consumer (snapshots, the Spolvero feed) shares.

use alloc::vec::Vec;
use core::fmt;
use core::str::SplitWhitespace;

use vize_davinci::folio::value::FolioValue;
use vize_davinci::folio::{Folio, FolioError};
use vize_s0::{Span, cstr};

use super::{PartitionFact, PartitionFacts, PartitionKind};

/// Owned folio page for the exported partition facts, `[s3-partition-folio]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Folio)]
pub struct S3PartitionFolio {
    /// One record per canonical S3 op, in export order.
    pub ops: Vec<FolioPartitionFact>,
}

impl S3PartitionFolio {
    /// Mirror live arena facts into the owned page.
    #[must_use]
    pub fn of(facts: &PartitionFacts<'_>) -> Self {
        Self {
            ops: facts.ops.iter().map(FolioPartitionFact::from).collect(),
        }
    }
}

/// One `op=<n> kind=<static|dynamic> span=<start>:<end>` record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolioPartitionFact {
    pub op: u32,
    pub kind: PartitionKind,
    pub span: Span,
}

impl From<&PartitionFact> for FolioPartitionFact {
    fn from(fact: &PartitionFact) -> Self {
        Self {
            op: fact.op.index(),
            kind: fact.kind,
            span: fact.span,
        }
    }
}

impl FolioValue for FolioPartitionFact {
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

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        let mut fields = text.split_whitespace();
        let op = parse_u32(field(&mut fields, "op", line)?, "op", line)?;
        let kind_text = field(&mut fields, "kind", line)?;
        let kind = PartitionKind::from_str(kind_text)
            .ok_or_else(|| FolioError::new(line, cstr!("unknown partition kind `{kind_text}`")))?;
        let span_text = field(&mut fields, "span", line)?;
        let Some((start, end)) = span_text.split_once(':') else {
            return Err(FolioError::new(line, cstr!("invalid span `{span_text}`")));
        };
        let span = Span::new(
            parse_u32(start, "span.start", line)?,
            parse_u32(end, "span.end", line)?,
        );
        if let Some(extra) = fields.next() {
            return Err(FolioError::new(line, cstr!("unexpected field `{extra}`")));
        }
        Ok(Self { op, kind, span })
    }
}

fn field<'a>(
    fields: &mut SplitWhitespace<'a>,
    name: &str,
    line: usize,
) -> Result<&'a str, FolioError> {
    let Some(raw) = fields.next() else {
        return Err(FolioError::new(line, cstr!("missing `{name}` field")));
    };
    raw.strip_prefix(name)
        .and_then(|rest| rest.strip_prefix('='))
        .ok_or_else(|| FolioError::new(line, cstr!("expected `{name}=...`, got `{raw}`")))
}

fn parse_u32(text: &str, name: &str, line: usize) -> Result<u32, FolioError> {
    text.parse()
        .map_err(|_| FolioError::new(line, cstr!("invalid `{name}` integer `{text}`")))
}
