//! The virtual-DOM target: `render(_ctx, _cache, …)` from L2 and L3 decisions.
//!
//! This replaces `vize_l1_to_l2::emit` (moved here, #6840, after #6838) and
//! gives DOM structural source maps from the writer.

#![expect(clippy::todo, reason = "skeleton: #6840")]

use vize_l2::op::Region;
use vize_l3::decision::DecisionTables;

use crate::expr::{AccessStyle, ResolvedExpressions};
use crate::write::{LinkSink, Writer};

/// Inputs of one DOM render emission.
#[derive(Debug, Clone, Copy)]
pub struct DomInput<'s, 'a> {
    /// The authored file text the L2 spans index.
    pub source: &'s str,
    /// The template root.
    pub root: &'s Region<'a>,
    /// Shared L3 decisions under the DOM policy.
    pub decisions: &'s DecisionTables,
    /// L2 identifier resolution.
    pub resolution: ResolvedExpressions<'s, 'a>,
    /// Accessor spelling.
    pub style: AccessStyle,
}

/// Write the DOM render function body for `input`.
pub fn emit<L: LinkSink>(_writer: &mut Writer<L>, _input: &DomInput<'_, '_>) {
    todo!("#6840: move the DOM emitter onto the L4 writer")
}
