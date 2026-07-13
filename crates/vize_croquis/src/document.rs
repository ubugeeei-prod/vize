//! Complete owned semantic document shared by production tool recipes.

use vize_atlas::Shared;
use vize_carton::{CompactString, String, source_anchor::SourceAnchor};

use crate::{Croquis, CroquisSemanticSnapshot};

/// One source-backed semantic input retained by a Croquis document.
///
/// `role` is intentionally open (`script`, `script-setup`, `template`, or a
/// future frontend-defined role). Atlas and Croquis do not enumerate source
/// container kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CroquisSourceSegment {
    role: CompactString,
    language: Option<CompactString>,
    text: String,
    anchor: SourceAnchor,
}

impl CroquisSourceSegment {
    pub fn new(
        role: impl Into<CompactString>,
        text: impl Into<String>,
        anchor: SourceAnchor,
    ) -> Self {
        Self {
            role: role.into(),
            language: None,
            text: text.into(),
            anchor,
        }
    }

    /// Attach the frontend-declared language without coupling Croquis to a
    /// particular source container. Consumers such as effect-graph builders
    /// can then parse embedded JSX/TSX correctly.
    pub fn with_language(mut self, language: impl Into<CompactString>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    pub const fn anchor(&self) -> SourceAnchor {
        self.anchor
    }
}

/// Full frontend-neutral Croquis analysis and its source provenance.
///
/// Unlike [`crate::CroquisSemanticSnapshot`], this retains the complete model
/// required by production lint, typecheck, editor, and compiler consumers.
/// Lightweight or serialization-oriented recipes can request the snapshot
/// projection as a separate product without rerunning frontend analysis.
#[derive(Debug, Default)]
pub struct CroquisDocument {
    analysis: Shared<Croquis>,
    semantic_snapshot: CroquisSemanticSnapshot,
    source_anchor: Option<SourceAnchor>,
    sources: Vec<CroquisSourceSegment>,
}

impl CroquisDocument {
    pub fn new(analysis: Croquis) -> Self {
        Self::from_shared(Shared::new(analysis))
    }

    pub fn from_shared(analysis: Shared<Croquis>) -> Self {
        let semantic_snapshot = analysis.semantic_snapshot();
        Self {
            analysis,
            semantic_snapshot,
            source_anchor: None,
            sources: Vec::new(),
        }
    }

    pub fn with_source_anchor(mut self, anchor: SourceAnchor) -> Self {
        self.source_anchor = Some(anchor);
        self.semantic_snapshot.source_anchor = Some(anchor);
        self
    }

    /// Replace the compact projection when a frontend can retain semantic
    /// facts that are intentionally absent from the complete legacy model.
    pub fn with_semantic_snapshot(mut self, mut snapshot: CroquisSemanticSnapshot) -> Self {
        snapshot.source_anchor = self.source_anchor.or(snapshot.source_anchor);
        self.semantic_snapshot = snapshot;
        self
    }

    pub fn with_source(mut self, source: CroquisSourceSegment) -> Self {
        self.sources.push(source);
        self
    }

    pub fn analysis(&self) -> &Croquis {
        &self.analysis
    }

    pub fn shared_analysis(&self) -> Shared<Croquis> {
        Shared::clone(&self.analysis)
    }

    pub const fn source_anchor(&self) -> Option<SourceAnchor> {
        self.source_anchor
    }

    pub fn sources(&self) -> &[CroquisSourceSegment] {
        &self.sources
    }

    pub fn source_by_role(&self, role: &str) -> Option<&CroquisSourceSegment> {
        self.sources.iter().find(|source| source.role == role)
    }

    pub fn semantic_snapshot(&self) -> CroquisSemanticSnapshot {
        self.semantic_snapshot.clone()
    }
}
