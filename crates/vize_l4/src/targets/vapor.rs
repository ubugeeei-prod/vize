//! The Vapor target, generated directly from the L3 program.
//!
//! Ports the legacy `generate` / `generators` to read the L3 program,
//! keeping the current emission order. The L3 → legacy IR adapter in
//! `vize_atelier_vapor` stays as the test oracle until byte parity, then it
//! is deleted with the legacy IR (#6840, after #6839).

#![expect(clippy::todo, reason = "skeleton: #6840")]

use vize_l3::op::Program;

use crate::expr::ResolutionTable;
use crate::write::{LinkSink, Writer};

/// Inputs of one Vapor emission.
#[derive(Debug, Clone, Copy)]
pub struct VaporInput<'s, 'a> {
    /// The authored file text the spans index.
    pub source: &'s str,
    /// The scheduled L3 program.
    pub program: &'s Program<'a>,
    /// L2 identifier resolution.
    pub resolution: ResolutionTable<'s>,
}

/// Write the Vapor render function for `input`.
pub fn emit<L: LinkSink>(_writer: &mut Writer<L>, _input: &VaporInput<'_, '_>) {
    todo!("#6840: generate Vapor JS from the L3 program")
}
