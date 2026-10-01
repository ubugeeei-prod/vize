//! The server rendering target: `ssrRender(_ctx, _push, …)` from L2 and L3.
//!
//! Native: facts come from L2 and L3, with no legacy codegen and no Croquis.
//! Replaces the `vize_atelier_ssr` L4 lane (#6840, after #6838 and #6839).

#![expect(clippy::todo, reason = "skeleton: #6840")]

use vize_l2::op::Region;
use vize_l3::decision::DecisionTables;

use crate::expr::ResolvedExpressions;
use crate::write::{LinkSink, Writer};

/// Inputs of one server render emission.
#[derive(Debug, Clone, Copy)]
pub struct SsrInput<'s, 'a> {
    /// The authored file text the L2 spans index.
    pub source: &'s str,
    /// The template root.
    pub root: &'s Region<'a>,
    /// Shared L3 decisions under the SSR policy.
    pub decisions: &'s DecisionTables,
    /// L2 identifier resolution.
    pub resolution: ResolvedExpressions<'s, 'a>,
}

/// Write the server render function body for `input`.
pub fn emit<L: LinkSink>(_writer: &mut Writer<L>, _input: &SsrInput<'_, '_>) {
    todo!("#6840: native SSR emission on the L4 writer")
}
