//! The extraction page: the TS-17 snapshot of every decision one extraction
//! made, with the plan metrics before and after it.

use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use vize_davinci::folio::value::FolioValue;
use vize_davinci::folio::{Folio, FolioError};
use vize_s0::{Span, String, cstr};

use super::report::{Decision, DecisionKind, Delta, Extraction, Reason};
use crate::op::OpId;
use crate::placement::folio::{field, parse_placement, parse_u32};

/// One extraction, printed as `[s3-extraction-folio]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Folio)]
pub struct S3ExtractionFolio {
    pub tier: String,
    pub candidate_budget: u32,
    pub budget_left: u32,
    pub emitted_size_before: u64,
    pub emitted_size_after: u64,
    pub reactive_edges_before: u64,
    pub reactive_edges_after: u64,
    pub update_path_before: u64,
    pub update_path_after: u64,
    pub decisions: Vec<FolioDecision>,
}

impl S3ExtractionFolio {
    /// Mirror one extraction into the owned document model.
    #[must_use]
    pub fn of(extraction: &Extraction) -> Self {
        let (before, after) = (extraction.before, extraction.after);
        Self {
            tier: String::from(extraction.tier.as_str()),
            candidate_budget: extraction.candidate_budget,
            budget_left: extraction.budget_left,
            emitted_size_before: before.emitted_size,
            emitted_size_after: after.emitted_size,
            reactive_edges_before: before.reactive_edges,
            reactive_edges_after: after.reactive_edges,
            update_path_before: before.update_path,
            update_path_after: after.update_path,
            decisions: extraction
                .decisions
                .iter()
                .copied()
                .map(FolioDecision)
                .collect(),
        }
    }
}

/// One decision row:
/// `op=<n> placement=<p> kind=<k> reason=<r> span=<s>:<e> size=<d> edges=<d> path=<d> budget=<n>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolioDecision(pub Decision);

impl FolioValue for FolioDecision {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        let decision = self.0;
        write!(
            w,
            "op={} placement={} kind={} reason={} span={}:{} size={} edges={} path={} budget={}",
            decision.op.index(),
            decision.placement,
            decision.kind.as_str(),
            decision.reason,
            decision.span.start,
            decision.span.end,
            decision.delta.emitted_size,
            decision.delta.reactive_edges,
            decision.delta.update_path,
            decision.budget_left
        )
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        let mut fields = text.split_whitespace();
        let op = OpId::new(parse_u32(field(&mut fields, "op", line)?, "op", line)?);
        let placement = parse_placement(field(&mut fields, "placement", line)?, line)?;
        let kind = match field(&mut fields, "kind", line)? {
            "applied" => DecisionKind::Applied,
            "missed" => DecisionKind::Missed,
            other => {
                return Err(FolioError::new(
                    line,
                    cstr!("unknown decision kind `{other}`"),
                ));
            }
        };
        let reason_text = field(&mut fields, "reason", line)?;
        let reason = Reason::parse(reason_text).ok_or_else(|| {
            FolioError::new(line, cstr!("unknown decision reason `{reason_text}`"))
        })?;
        if (kind == DecisionKind::Applied) != (reason == Reason::Committed) {
            return Err(FolioError::new(
                line,
                cstr!(
                    "decision kind `{}` contradicts reason `{reason}`",
                    kind.as_str()
                ),
            ));
        }
        let span_text = field(&mut fields, "span", line)?;
        let (start, end) = span_text
            .split_once(':')
            .ok_or_else(|| FolioError::new(line, cstr!("invalid span `{span_text}`")))?;
        let span = Span::new(
            parse_u32(start, "span.start", line)?,
            parse_u32(end, "span.end", line)?,
        );
        let delta = Delta {
            emitted_size: parse_int(field(&mut fields, "size", line)?, "size", line)?,
            reactive_edges: parse_int(field(&mut fields, "edges", line)?, "edges", line)?,
            update_path: parse_int(field(&mut fields, "path", line)?, "path", line)?,
        };
        let budget_left = parse_u32(field(&mut fields, "budget", line)?, "budget", line)?;
        if let Some(extra) = fields.next() {
            return Err(FolioError::new(line, cstr!("unexpected field `{extra}`")));
        }
        Ok(Self(Decision {
            op,
            span,
            placement,
            kind,
            reason,
            delta,
            budget_left,
        }))
    }
}

fn parse_int<T: FromStr>(text: &str, name: &str, line: usize) -> Result<T, FolioError> {
    text.parse()
        .map_err(|_| FolioError::new(line, cstr!("invalid `{name}` integer `{text}`")))
}
