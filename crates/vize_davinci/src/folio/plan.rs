//! The fusion-plan page: which walk each pass of a pipeline runs in (C-3).
//!
//! A [`Pipeline`] is const data and its fusion grouping a `const fn` over it
//! ([`Pipeline::group`]), so the plan is fully decided before anything runs.
//! The timing observer records **one span per walk**, attributed to the
//! walk's lead pass, and says nothing about which other passes shared it -
//! that is recoverable from the plan alone. This page is that plan as a
//! derived, round-trippable stage dump, so a timing view can read a profile
//! without guessing:
//!
//! ```text
//! [fusion-plan-folio]
//! stage=s2
//! walks=2
//!
//! [fusion-plan-folio.passes]
//! walk=0 pass=v-slot kind=mandatory-lowering fusability=barrier
//! walk=1 pass=hoist-static kind=optional fusability=fusable
//!
//! ```
//!
//! One record per pass, in execution order; `walk` is the 0-based fusion
//! group index, so passes sharing a walk share the number.

use alloc::vec::Vec;
use core::fmt;
use core::str::SplitWhitespace;

use vize_s0::{String, cstr};

use super::value::FolioValue;
use super::{Folio, FolioError};
use crate::pass::{Fusability, PassKind, Pipeline};

/// Owned folio page for a pipeline's fusion plan, `[fusion-plan-folio]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Folio)]
pub struct FusionPlanFolio {
    /// The stage the pipeline runs over, e.g. `s2`.
    pub stage: String,
    /// What running the plan costs in walks: its fusion-group count.
    pub walks: u32,
    /// One record per pass, in execution order.
    pub passes: Vec<FolioPlanPass>,
}

impl FusionPlanFolio {
    /// The plan `pipeline` runs: every pass with the walk it lands in.
    #[must_use]
    pub fn of(pipeline: &Pipeline) -> Self {
        let walks = pipeline.group_count();
        let mut passes = Vec::with_capacity(pipeline.passes.len());
        for walk in 0..walks {
            // A group index below `group_count` always resolves.
            let Some(group) = pipeline.group(walk) else {
                continue;
            };
            let members = pipeline.passes.get(group.start..group.end());
            for desc in members.unwrap_or_default() {
                passes.push(FolioPlanPass {
                    walk: walk_number(walk),
                    pass: String::from(desc.name),
                    kind: desc.kind,
                    fusability: desc.fusability,
                });
            }
        }
        Self {
            stage: String::from(pipeline.stage),
            walks: walk_number(walks),
            passes,
        }
    }
}

/// Walk indices print as `u32`; a pipeline is const data with a handful of
/// passes, so the conversion saturates only in theory.
fn walk_number(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// One `walk=<n> pass=<name> kind=<kind> fusability=<fusability>` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioPlanPass {
    /// The 0-based fusion group (walk) the pass runs in.
    pub walk: u32,
    /// The pass name.
    pub pass: String,
    /// Mandatory or optional ([`PassKind::as_str`]).
    pub kind: PassKind,
    /// Whether it may share a walk ([`Fusability::as_str`]).
    pub fusability: Fusability,
}

impl FolioValue for FolioPlanPass {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(
            w,
            "walk={} pass={} kind={} fusability={}",
            self.walk,
            self.pass,
            self.kind.as_str(),
            self.fusability.as_str()
        )
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        let mut fields = text.split_whitespace();
        let walk_text = field(&mut fields, "walk", line)?;
        let walk = walk_text
            .parse()
            .map_err(|_| FolioError::new(line, cstr!("invalid `walk` integer `{walk_text}`")))?;
        let pass = field(&mut fields, "pass", line)?;
        if pass.is_empty() {
            return Err(FolioError::new(line, String::from("empty pass name")));
        }
        let kind_text = field(&mut fields, "kind", line)?;
        let kind = PassKind::from_str(kind_text)
            .ok_or_else(|| FolioError::new(line, cstr!("unknown pass kind `{kind_text}`")))?;
        let fusability_text = field(&mut fields, "fusability", line)?;
        let fusability = Fusability::from_str(fusability_text).ok_or_else(|| {
            FolioError::new(line, cstr!("unknown fusability `{fusability_text}`"))
        })?;
        if let Some(extra) = fields.next() {
            return Err(FolioError::new(line, cstr!("unexpected field `{extra}`")));
        }
        Ok(Self {
            walk,
            pass: String::from(pass),
            kind,
            fusability,
        })
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
