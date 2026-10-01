//! The type-check projection: virtual TypeScript for one SFC.
//!
//! The checker reads the projected text and maps diagnostics back to the
//! authored file through the writer's span links (#6840, P2).

#![expect(clippy::todo, reason = "skeleton: #6840")]

use vize_l2::op::Region;

use crate::expr::ResolvedExpressions;
use crate::write::{LinkSink, Writer};

/// Inputs of one type-check projection.
#[derive(Debug, Clone, Copy)]
pub struct TsInput<'s, 'a> {
    /// The authored file text the spans index.
    pub source: &'s str,
    /// The template root.
    pub root: &'s Region<'a>,
    /// L2 identifier resolution.
    pub resolution: ResolvedExpressions<'s, 'a>,
}

/// Write the template's type-check projection for `input`.
pub fn project<L: LinkSink>(_writer: &mut Writer<L>, _input: &TsInput<'_, '_>) {
    todo!("#6840: project the template into checkable TypeScript")
}
