//! Atlas identity for the owned, source-faithful Relief syntax product.

use vize_atlas::{CompilationInput, Product};
use vize_carton::config::VueVersion;

use crate::{CompilerError, ReliefSnapshot};

/// Owned result of parsing one Vue template.
///
/// This is deliberately source-faithful and untransformed. Consumers such as
/// linters can inspect the same syntax and diagnostics without paying for, or
/// being coupled to, the compiler transform pass.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ReliefArtifact {
    snapshot: ReliefSnapshot,
    parse_diagnostics: Vec<CompilerError>,
}

impl ReliefArtifact {
    pub fn new(snapshot: ReliefSnapshot, parse_diagnostics: Vec<CompilerError>) -> Self {
        Self {
            snapshot,
            parse_diagnostics,
        }
    }

    pub const fn snapshot(&self) -> &ReliefSnapshot {
        &self.snapshot
    }

    pub fn parse_diagnostics(&self) -> &[CompilerError] {
        &self.parse_diagnostics
    }

    #[must_use]
    pub fn has_fatal_diagnostics(&self) -> bool {
        self.parse_diagnostics
            .iter()
            .any(|diagnostic| !diagnostic.is_recoverable())
    }
}

impl AsRef<ReliefSnapshot> for ReliefArtifact {
    fn as_ref(&self) -> &ReliefSnapshot {
        &self.snapshot
    }
}

/// Owned result of applying Relief's compiler transform pass.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TransformedReliefArtifact {
    snapshot: ReliefSnapshot,
    parse_diagnostics: Vec<CompilerError>,
    transform_diagnostics: Vec<CompilerError>,
}

impl TransformedReliefArtifact {
    pub fn new(
        snapshot: ReliefSnapshot,
        parse_diagnostics: Vec<CompilerError>,
        transform_diagnostics: Vec<CompilerError>,
    ) -> Self {
        Self {
            snapshot,
            parse_diagnostics,
            transform_diagnostics,
        }
    }

    pub const fn snapshot(&self) -> &ReliefSnapshot {
        &self.snapshot
    }

    pub fn parse_diagnostics(&self) -> &[CompilerError] {
        &self.parse_diagnostics
    }

    pub fn transform_diagnostics(&self) -> &[CompilerError] {
        &self.transform_diagnostics
    }

    #[must_use]
    pub fn has_fatal_diagnostics(&self) -> bool {
        self.parse_diagnostics
            .iter()
            .any(|diagnostic| !diagnostic.is_recoverable())
            || !self.transform_diagnostics.is_empty()
    }
}

impl AsRef<ReliefSnapshot> for TransformedReliefArtifact {
    fn as_ref(&self) -> &ReliefSnapshot {
        &self.snapshot
    }
}

/// Demandable Vue-template syntax snapshot.
///
/// Parsing and lowering providers live in frontend crates. Relief owns only
/// the value type and its open graph identity.
pub struct ReliefProduct;

impl Product for ReliefProduct {
    type Value = Option<ReliefArtifact>;

    const NAME: &'static str = "relief.syntax";
}

/// Demandable Relief syntax after compiler structural/expression transforms.
///
/// Render and semantic projections depend on this product; parse-only tools
/// should request [`ReliefProduct`] instead.
pub struct TransformedReliefProduct;

impl Product for TransformedReliefProduct {
    type Value = Option<TransformedReliefArtifact>;

    const NAME: &'static str = "relief.transformed";
}

/// Vue language line relevant to syntax and semantic providers.
pub struct VueDialectInput;

impl CompilationInput for VueDialectInput {
    type Value = VueVersion;

    const NAME: &'static str = "vue.dialect";
}
