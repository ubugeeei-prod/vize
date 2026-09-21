//! The provenance page: every lowering and pass decision as a derived,
//! round-trippable folio page (`[s2-provenance-folio]`).
//!
//! [`ProvenanceRecord`]s are what answers "why does the output contain
//! this?" (`devtool.md`, Provenance): which rule fired, on which authored
//! bytes, producing which op - or producing nothing, the survival law's
//! failure half. The page mirrors them one line per record, in decision
//! order:
//!
//! ```text
//! rule=lower.element node=0 before="<p>" after="ui.element p" @3:10
//! rule=drop.comment node=- before="<!-- x -->" after="" @10:20
//! ```
//!
//! `before`/`after` use the S2 page's string escapes. Rule names are single
//! words by convention (`lower.element`, `pass.hoist-static.fact`); `parse`
//! rejects anything else rather than guessing where the name ends.

use alloc::vec::Vec;
use core::fmt;

use vize_davinci::folio::value::FolioValue;
use vize_davinci::folio::{Folio, FolioError};
use vize_s0::{Span, String, cstr};

use super::parse::{tail_span_value, take_quoted_value};
use super::print::quoted;
use crate::provenance::ProvenanceRecord;

/// Owned page of provenance records, in decision order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Folio)]
pub struct S2ProvenanceFolio {
    /// One record per lowering or pass decision.
    pub records: Vec<FolioProvenance>,
}

impl S2ProvenanceFolio {
    /// Mirror live records into the owned page.
    #[must_use]
    pub fn of(records: &[ProvenanceRecord]) -> Self {
        Self {
            records: records.iter().map(FolioProvenance::from).collect(),
        }
    }
}

/// One `rule=… node=… before="…" after="…" @s:e` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioProvenance {
    /// The deciding rule.
    pub rule: String,
    /// The produced op's dense page-order id, or `None` when nothing was
    /// produced.
    pub node: Option<u32>,
    /// The authored text the decision consumed.
    pub before: String,
    /// The produced form, empty when nothing was produced.
    pub after: String,
    /// The authored range of `before`.
    pub span: Span,
}

impl From<&ProvenanceRecord> for FolioProvenance {
    fn from(record: &ProvenanceRecord) -> Self {
        Self {
            rule: record.rule.clone(),
            node: record.node.map(|node| node.index()),
            before: record.before.clone(),
            after: record.after.clone(),
            span: record.span,
        }
    }
}

impl FolioValue for FolioProvenance {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "rule={} node=", self.rule)?;
        match self.node {
            Some(node) => write!(w, "{node}")?,
            None => w.write_char('-')?,
        }
        w.write_str(" before=")?;
        quoted(w, &self.before)?;
        w.write_str(" after=")?;
        quoted(w, &self.after)?;
        write!(w, " @{}:{}", self.span.start, self.span.end)
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        let rest = expect(text, "rule=", line)?;
        let (rule, rest) = rest
            .split_once(' ')
            .filter(|(rule, _)| !rule.is_empty())
            .ok_or_else(|| FolioError::new(line, cstr!("expected `rule=<name> `")))?;
        let rest = expect(rest, "node=", line)?;
        let (node, rest) = rest
            .split_once(' ')
            .ok_or_else(|| FolioError::new(line, cstr!("expected `node=<id|-> `")))?;
        let node = match node {
            "-" => None,
            id => Some(
                id.parse()
                    .map_err(|_| FolioError::new(line, cstr!("invalid node id `{id}`")))?,
            ),
        };
        let (before, rest) = take_quoted_value(expect(rest, "before=", line)?, line)?;
        let (after, rest) = take_quoted_value(expect(rest, " after=", line)?, line)?;
        let span = tail_span_value(rest, line)?;
        Ok(Self {
            rule: String::from(rule),
            node,
            before,
            after,
            span,
        })
    }
}

fn expect<'a>(text: &'a str, prefix: &str, line: usize) -> Result<&'a str, FolioError> {
    text.strip_prefix(prefix)
        .ok_or_else(|| FolioError::new(line, cstr!("expected `{prefix}`")))
}
