//! The diagnostic record.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use alloc::boxed::Box;

use super::witness::WitnessChain;
use crate::level::Level;
use crate::span::Span;

/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// A hint.
    Hint,
    /// A warning.
    Warning,
    /// An error.
    Error,
}

/// One diagnostic, keyed by a span and optionally proven by a witness chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Severity.
    pub severity: Severity,
    /// The level that reported it.
    pub level: Level,
    /// Primary span.
    pub span: Span,
    /// Primary message.
    pub message: Box<str>,
    /// The facts that prove it, when it is proven.
    pub witness: Option<WitnessChain>,
}

impl Diagnostic {
    /// An unproven diagnostic.
    #[must_use]
    pub fn new(severity: Severity, level: Level, span: Span, message: Box<str>) -> Self {
        let _ = (severity, level, span, message);
        todo!()
    }

    /// Attach the witness chain that proves this diagnostic.
    #[must_use]
    pub fn with_witness(self, witness: WitnessChain) -> Self {
        let _ = witness;
        todo!()
    }
}
