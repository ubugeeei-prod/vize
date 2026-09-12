//! The Impeto folio page.
//!
//! Unlike S2, S3 is already flat: op ids, region ids, state edges, and effect
//! scopes fit the derived-page grammar. The custom line values below keep the
//! page typed while staying line-atomic.

use alloc::vec::Vec;
use core::fmt;
use core::str::SplitWhitespace;

use vize_davinci::folio::value::FolioValue;
use vize_davinci::folio::{Folio, FolioError};
use vize_s0::{Span, String, cstr};

use crate::op::{EdgeKind, EffectId, OpId, OpKind, Program, RegionId};

/// Flat S3 document model.
#[doc(alias = "ImpetoFolio")]
#[derive(Debug, Clone, Default, PartialEq, Eq, Folio)]
pub struct S3Folio {
    /// `built`, `partitioned`, or `scheduled`.
    pub phase: String,
    /// Region records in page order.
    pub regions: Vec<FolioRegion>,
    /// Operation records in page order.
    pub ops: Vec<FolioOp>,
    /// State-edge records in page order.
    pub edges: Vec<FolioEdge>,
    /// Effect-scope records in page order.
    pub effects: Vec<FolioEffect>,
}

impl S3Folio {
    /// Mirror a live arena program into the owned document model.
    #[must_use]
    pub fn of(program: &Program<'_>) -> Self {
        Self {
            phase: String::from(program.phase.as_str()),
            regions: program.regions.iter().map(FolioRegion::from).collect(),
            ops: program.ops.iter().map(FolioOp::from).collect(),
            edges: program.edges.iter().map(FolioEdge::from).collect(),
            effects: program.effects.iter().map(FolioEffect::from).collect(),
        }
    }
}

/// Compatibility alias for the codename spelling.
pub type ImpetoFolio = S3Folio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolioRegion {
    pub id: u32,
    pub parent: Option<u32>,
    pub owner: Option<u32>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolioOp {
    pub id: u32,
    pub kind: OpKind,
    pub region: u32,
    pub effect: Option<u32>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolioEdge {
    pub from: u32,
    pub to: u32,
    pub kind: EdgeKind,
    pub effect: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolioEffect {
    pub id: u32,
    pub owner: u32,
    pub region: u32,
    pub span: Span,
}

impl From<&crate::op::Region> for FolioRegion {
    fn from(region: &crate::op::Region) -> Self {
        Self {
            id: region.id.index(),
            parent: region.parent.map(RegionId::index),
            owner: region.owner.map(OpId::index),
            span: region.span,
        }
    }
}

impl From<&crate::op::Op> for FolioOp {
    fn from(op: &crate::op::Op) -> Self {
        Self {
            id: op.id.index(),
            kind: op.kind,
            region: op.region.index(),
            effect: op.effect.map(EffectId::index),
            span: op.span,
        }
    }
}

impl From<&crate::op::StateEdge> for FolioEdge {
    fn from(edge: &crate::op::StateEdge) -> Self {
        Self {
            from: edge.from.index(),
            to: edge.to.index(),
            kind: edge.kind,
            effect: edge.effect.map(EffectId::index),
        }
    }
}

impl From<&crate::op::EffectScope> for FolioEffect {
    fn from(effect: &crate::op::EffectScope) -> Self {
        Self {
            id: effect.id.index(),
            owner: effect.owner.index(),
            region: effect.region.index(),
            span: effect.span,
        }
    }
}

impl FolioValue for FolioRegion {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(
            w,
            "id={} parent={} owner={} span={}:{}",
            self.id,
            format_option(self.parent),
            format_option(self.owner),
            self.span.start,
            self.span.end
        )
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        let mut fields = text.split_whitespace();
        let id = parse_u32(next(&mut fields, "id", line)?, "id", line)?;
        let parent = parse_option(next(&mut fields, "parent", line)?, "parent", line)?;
        let owner = parse_option(next(&mut fields, "owner", line)?, "owner", line)?;
        let span = parse_span(next(&mut fields, "span", line)?, line)?;
        expect_end(fields, line)?;
        Ok(Self {
            id,
            parent,
            owner,
            span,
        })
    }
}

impl FolioValue for FolioOp {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(
            w,
            "id={} kind={} region={} effect={} span={}:{}",
            self.id,
            self.kind.mnemonic(),
            self.region,
            format_option(self.effect),
            self.span.start,
            self.span.end
        )
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        let mut fields = text.split_whitespace();
        let id = parse_u32(next(&mut fields, "id", line)?, "id", line)?;
        let kind = parse_op_kind(next(&mut fields, "kind", line)?, line)?;
        let region = parse_u32(next(&mut fields, "region", line)?, "region", line)?;
        let effect = parse_option(next(&mut fields, "effect", line)?, "effect", line)?;
        let span = parse_span(next(&mut fields, "span", line)?, line)?;
        expect_end(fields, line)?;
        Ok(Self {
            id,
            kind,
            region,
            effect,
            span,
        })
    }
}

impl FolioValue for FolioEdge {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(
            w,
            "from={} to={} kind={} effect={}",
            self.from,
            self.to,
            self.kind.as_str(),
            format_option(self.effect)
        )
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        let mut fields = text.split_whitespace();
        let from = parse_u32(next(&mut fields, "from", line)?, "from", line)?;
        let to = parse_u32(next(&mut fields, "to", line)?, "to", line)?;
        let kind = parse_edge_kind(next(&mut fields, "kind", line)?, line)?;
        let effect = parse_option(next(&mut fields, "effect", line)?, "effect", line)?;
        expect_end(fields, line)?;
        Ok(Self {
            from,
            to,
            kind,
            effect,
        })
    }
}

impl FolioValue for FolioEffect {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(
            w,
            "id={} owner={} region={} span={}:{}",
            self.id, self.owner, self.region, self.span.start, self.span.end
        )
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        let mut fields = text.split_whitespace();
        let id = parse_u32(next(&mut fields, "id", line)?, "id", line)?;
        let owner = parse_u32(next(&mut fields, "owner", line)?, "owner", line)?;
        let region = parse_u32(next(&mut fields, "region", line)?, "region", line)?;
        let span = parse_span(next(&mut fields, "span", line)?, line)?;
        expect_end(fields, line)?;
        Ok(Self {
            id,
            owner,
            region,
            span,
        })
    }
}

fn format_option(value: Option<u32>) -> String {
    match value {
        Some(value) => cstr!("{value}"),
        None => String::from("-"),
    }
}

fn next<'a>(
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

fn expect_end(mut fields: SplitWhitespace<'_>, line: usize) -> Result<(), FolioError> {
    match fields.next() {
        Some(extra) => Err(FolioError::new(line, cstr!("unexpected field `{extra}`"))),
        None => Ok(()),
    }
}

fn parse_u32(text: &str, name: &str, line: usize) -> Result<u32, FolioError> {
    text.parse()
        .map_err(|_| FolioError::new(line, cstr!("invalid `{name}` integer `{text}`")))
}

fn parse_option(text: &str, name: &str, line: usize) -> Result<Option<u32>, FolioError> {
    if text == "-" {
        return Ok(None);
    }
    parse_u32(text, name, line).map(Some)
}

fn parse_span(text: &str, line: usize) -> Result<Span, FolioError> {
    let Some((start, end)) = text.split_once(':') else {
        return Err(FolioError::new(line, cstr!("invalid span `{text}`")));
    };
    Ok(Span::new(
        parse_u32(start, "span.start", line)?,
        parse_u32(end, "span.end", line)?,
    ))
}

fn parse_op_kind(text: &str, line: usize) -> Result<OpKind, FolioError> {
    OpKind::from_mnemonic(text)
        .ok_or_else(|| FolioError::new(line, cstr!("unknown Impeto op kind `{text}`")))
}

fn parse_edge_kind(text: &str, line: usize) -> Result<EdgeKind, FolioError> {
    EdgeKind::from_str(text)
        .ok_or_else(|| FolioError::new(line, cstr!("unknown Impeto edge kind `{text}`")))
}

#[cfg(test)]
mod tests {
    use super::S3Folio;
    use crate::op::Phase;
    use vize_davinci::folio::{Folio, FolioError, FolioMode};
    use vize_s0::{String, cstr};

    #[test]
    fn phase_spellings_are_parseable() {
        assert_eq!(Phase::from_str("built"), Some(Phase::Built));
        assert_eq!(Phase::from_str("partitioned"), Some(Phase::Partitioned));
        assert_eq!(Phase::from_str("scheduled"), Some(Phase::Scheduled));
        assert_eq!(Phase::from_str("unknown"), None);
    }

    #[test]
    fn derived_page_rejects_an_unknown_record_kind() {
        let input = "\
[s3-folio]
phase=built

[s3-folio.ops]
id=0 kind=impeto.missing region=0 effect=- span=0:1

";
        assert_eq!(
            S3Folio::parse(input).unwrap_err(),
            FolioError::new(5, cstr!("unknown Impeto op kind `impeto.missing`"))
        );
    }

    #[test]
    fn display_mode_has_the_derived_full_text_law() {
        let folio = S3Folio {
            phase: String::from("built"),
            ..S3Folio::default()
        };
        assert_eq!(
            folio.print_to_string(FolioMode::Display),
            folio.print_to_string(FolioMode::Full)
        );
    }
}
