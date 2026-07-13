//! SSR emission from the owned, frontend-neutral Rendu HIR.

mod emit;

#[cfg(test)]
mod parity_tests;
#[cfg(test)]
mod tests;

use vize_carton::{String, source_anchor::SourceAnchor};
use vize_rendu::{RenduRoot, RenduSpan};

/// Kind of Rendu artifact represented by one generated-code mapping.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RenduSsrMappingKind {
    Node,
    Property,
    Expression,
    Binding,
    Branch,
}

/// Byte range in generated SSR code tied to an original source span.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RenduSsrMapping {
    pub generated_start: usize,
    pub generated_end: usize,
    pub source: RenduSpan,
    /// Stable compilation source identity behind the Rendu-local span.
    pub anchor: Option<SourceAnchor>,
    pub kind: RenduSsrMappingKind,
}

/// Deterministic SSR module emitted from a Rendu graph.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct RenduSsrOutput {
    pub code: String,
    pub mappings: Vec<RenduSsrMapping>,
}

/// Compile a validated Rendu graph without consulting its producer AST.
pub fn compile_rendu(root: &RenduRoot) -> RenduSsrOutput {
    emit::SsrEmitter::new(root).emit()
}
