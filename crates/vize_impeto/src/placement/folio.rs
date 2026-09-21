//! The placement page paired with the graph-only `S3Folio`.
//!
//! The graph page keeps the grammar the Lean reference parses; placements are
//! an overlay, so they print on their own derived page. Parsing checks syntax
//! only; whether a record is legal for its program is the verifier's call.

use alloc::vec::Vec;
use core::fmt;
use core::str::SplitWhitespace;

use vize_davinci::folio::value::FolioValue;
use vize_davinci::folio::{Folio, FolioError};
use vize_s0::cstr;

use super::{Placement, PlacementRecord, PlacementSet};
use crate::op::{OpId, Program};

/// Placement records in program order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Folio)]
pub struct S3PlacementFolio {
    pub placements: Vec<FolioPlacement>,
}

impl S3PlacementFolio {
    /// Mirror the live placement overlay into the owned document model.
    #[must_use]
    pub fn of(program: &Program<'_>) -> Self {
        Self {
            placements: program
                .placements
                .iter()
                .map(|record| FolioPlacement(*record))
                .collect(),
        }
    }
}

/// One placement row: `op=<n> alternatives=<list> leader=<n|-> chosen=<p>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolioPlacement(pub PlacementRecord);

impl FolioValue for FolioPlacement {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        let record = self.0;
        write!(
            w,
            "op={} alternatives={} leader=",
            record.op.index(),
            record.alternatives
        )?;
        match record.leader {
            Some(leader) => write!(w, "{}", leader.index())?,
            None => w.write_str("-")?,
        }
        write!(w, " chosen={}", record.chosen)
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        let mut fields = text.split_whitespace();
        let op = parse_u32(field(&mut fields, "op", line)?, "op", line)?;
        let alternatives = parse_set(field(&mut fields, "alternatives", line)?, line)?;
        let leader = match field(&mut fields, "leader", line)? {
            "-" => None,
            value => Some(OpId::new(parse_u32(value, "leader", line)?)),
        };
        let chosen = parse_placement(field(&mut fields, "chosen", line)?, line)?;
        if let Some(extra) = fields.next() {
            return Err(FolioError::new(line, cstr!("unexpected field `{extra}`")));
        }
        Ok(Self(PlacementRecord {
            op: OpId::new(op),
            alternatives,
            leader,
            chosen,
        }))
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

fn parse_placement(text: &str, line: usize) -> Result<Placement, FolioError> {
    Placement::from_str(text)
        .ok_or_else(|| FolioError::new(line, cstr!("unknown placement `{text}`")))
}

/// `-` or a list in canonical order, so print is injective on parse.
fn parse_set(text: &str, line: usize) -> Result<PlacementSet, FolioError> {
    let mut set = PlacementSet::EMPTY;
    if text == "-" {
        return Ok(set);
    }
    let mut previous: Option<Placement> = None;
    for name in text.split(',') {
        let placement = parse_placement(name, line)?;
        if previous.is_some_and(|previous| previous >= placement) {
            return Err(FolioError::new(
                line,
                cstr!("placement alternatives `{text}` are not in canonical order"),
            ));
        }
        previous = Some(placement);
        set = set.with(placement);
    }
    Ok(set)
}
