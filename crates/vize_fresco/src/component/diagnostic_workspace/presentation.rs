use compact_str::{CompactString, ToCompactString};
use serde::Serialize;
use thiserror::Error;

mod model;
mod wire;

pub use model::{DiagnosticPresentationKind, DiagnosticPresentationProfile, DiagnosticTone};

use crate::{
    component::TextNode,
    headless::{HeadlessSemanticNode, SemanticState},
    render::{NodeId, RenderNode},
    terminal::{Style, TerminalCapabilities},
};

/// Invalid semantic diagnostic presentation input.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DiagnosticPresentationError {
    /// A required text field contains no visible text.
    #[error("diagnostic presentation {field} must contain visible text")]
    EmptyText {
        /// Stable field name for callers and tests.
        field: &'static str,
    },
    /// A score has no usable maximum or exceeds it.
    #[error("diagnostic score {value} is outside 0 through {maximum}")]
    InvalidScore {
        /// Supplied score.
        value: u64,
        /// Supplied inclusive maximum.
        maximum: u64,
    },
    /// Evidence position is zero or outside the declared logical set.
    #[error("diagnostic evidence position {position} is outside 1 through {set_size}")]
    InvalidEvidencePosition {
        /// Supplied one-based position.
        position: u64,
        /// Supplied logical set size.
        set_size: u64,
    },
    /// A source line or column uses the invalid zero value.
    #[error("diagnostic source locations use one-based line and column values")]
    InvalidCodeLocation,
    /// Structured metadata is absent, contradictory, or non-canonical.
    #[error("invalid {kind:?} presentation structure: {reason}")]
    InvalidStructure {
        /// Presentation kind whose invariant was violated.
        kind: DiagnosticPresentationKind,
        /// Stable explanation suitable for a wire-boundary error.
        reason: &'static str,
    },
}

/// One typed visual and accessible value in a diagnostic workspace.
///
/// Fresco deliberately owns presentation semantics rather than Doctor's domain
/// enums. An analyzer maps its stable values into this small contract, keeping
/// the TUI reusable for third-party rules and other diagnostic products.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticPresentation {
    kind: DiagnosticPresentationKind,
    tone: DiagnosticTone,
    value: CompactString,
    description: Option<CompactString>,
    score: Option<(u64, u64)>,
    set_position: Option<(u64, u64)>,
}

impl DiagnosticPresentation {
    /// Create a typed textual value.
    pub fn new(
        kind: DiagnosticPresentationKind,
        value: impl Into<CompactString>,
        tone: DiagnosticTone,
    ) -> Result<Self, DiagnosticPresentationError> {
        if matches!(
            kind,
            DiagnosticPresentationKind::Score
                | DiagnosticPresentationKind::CodeLocation
                | DiagnosticPresentationKind::Evidence
                | DiagnosticPresentationKind::KeyHint
        ) {
            return Err(DiagnosticPresentationError::InvalidStructure {
                kind,
                reason: "use the dedicated constructor for this presentation kind",
            });
        }
        Self::from_text(kind, value, tone)
    }

    fn from_text(
        kind: DiagnosticPresentationKind,
        value: impl Into<CompactString>,
        tone: DiagnosticTone,
    ) -> Result<Self, DiagnosticPresentationError> {
        Ok(Self {
            kind,
            tone,
            value: visible_text(value, "value")?,
            description: None,
            score: None,
            set_position: None,
        })
    }

    /// Create a bounded score presentation.
    pub fn score(
        value: u64,
        maximum: u64,
        tone: DiagnosticTone,
    ) -> Result<Self, DiagnosticPresentationError> {
        if maximum == 0 || value > maximum {
            return Err(DiagnosticPresentationError::InvalidScore { value, maximum });
        }
        let mut text = value.to_compact_string();
        text.push_str(" / ");
        text.push_str(&maximum.to_compact_string());
        let mut presentation = Self::from_text(DiagnosticPresentationKind::Score, text, tone)?;
        presentation.score = Some((value, maximum));
        Ok(presentation)
    }

    /// Create a one-based source location using a portable ASCII path suffix.
    pub fn code_location(
        path: impl Into<CompactString>,
        line: u64,
        column: u64,
    ) -> Result<Self, DiagnosticPresentationError> {
        if line == 0 || column == 0 {
            return Err(DiagnosticPresentationError::InvalidCodeLocation);
        }
        let mut value = visible_text(path, "path")?;
        value.push(':');
        value.push_str(&line.to_compact_string());
        value.push(':');
        value.push_str(&column.to_compact_string());
        Self::from_text(
            DiagnosticPresentationKind::CodeLocation,
            value,
            DiagnosticTone::Neutral,
        )
    }

    /// Create supporting evidence with virtualized logical-set metadata.
    pub fn evidence(
        summary: impl Into<CompactString>,
        position: u64,
        set_size: u64,
    ) -> Result<Self, DiagnosticPresentationError> {
        if position == 0 || position > set_size {
            return Err(DiagnosticPresentationError::InvalidEvidencePosition {
                position,
                set_size,
            });
        }
        let mut presentation = Self::from_text(
            DiagnosticPresentationKind::Evidence,
            summary,
            DiagnosticTone::Informational,
        )?;
        presentation.set_position = Some((position, set_size));
        Ok(presentation)
    }

    /// Create a keyboard hint while keeping the key and action independently validated.
    pub fn key_hint(
        key: impl Into<CompactString>,
        action: impl Into<CompactString>,
    ) -> Result<Self, DiagnosticPresentationError> {
        let mut value = visible_text(key, "key")?;
        value.push_str(": ");
        value.push_str(&visible_text(action, "action")?);
        Self::from_text(
            DiagnosticPresentationKind::KeyHint,
            value,
            DiagnosticTone::Neutral,
        )
    }

    /// Attach a longer accessible explanation.
    pub fn with_description(
        mut self,
        description: impl Into<CompactString>,
    ) -> Result<Self, DiagnosticPresentationError> {
        self.description = Some(visible_text(description, "description")?);
        Ok(self)
    }

    /// Return the semantic purpose.
    pub const fn kind(&self) -> DiagnosticPresentationKind {
        self.kind
    }

    /// Return the non-color tone.
    pub const fn tone(&self) -> DiagnosticTone {
        self.tone
    }

    /// Return the normalized displayed value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Return the optional accessible explanation.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Return a non-color, Unicode-aware visual row.
    pub fn text(&self, profile: DiagnosticPresentationProfile) -> CompactString {
        let mut text = CompactString::new(self.tone.marker(profile.unicode));
        text.push(' ');
        if !profile.compact {
            text.push_str(self.kind.label());
            text.push_str(": ");
        }
        text.push_str(&self.value);
        text
    }

    /// Return the portable base style before terminal-capability adaptation.
    pub const fn style(&self) -> Style {
        self.tone.style()
    }

    /// Build a visual row through Fresco's ordinary render-tree path.
    pub fn render_node(
        &self,
        node_id: NodeId,
        profile: DiagnosticPresentationProfile,
    ) -> RenderNode {
        self.render_node_with_style(node_id, profile, self.style())
    }

    /// Build a visual row adapted to one resolved terminal profile.
    ///
    /// The capability contract selects Unicode and compact text and clamps
    /// color to the supported depth. Text markers continue to carry tone in
    /// monochrome and redirected output.
    pub fn render_node_for_capabilities(
        &self,
        node_id: NodeId,
        capabilities: TerminalCapabilities,
    ) -> RenderNode {
        self.render_node_with_style(
            node_id,
            capabilities.into(),
            capabilities.adapt_style(self.style()),
        )
    }

    /// Build semantic metadata for headless and accessibility assertions.
    pub fn semantic_node(&self, node_id: NodeId) -> HeadlessSemanticNode {
        let mut state = SemanticState::default().with_value(self.value.clone());
        if let Some((position, set_size)) = self.set_position {
            state = state.with_set_position(position, set_size);
        }
        let mut semantic =
            HeadlessSemanticNode::new(node_id, self.kind.role(self.tone), self.kind.label())
                .with_state(state);
        if let Some(description) = &self.description {
            semantic = semantic.with_description(description.clone());
        }
        semantic
    }

    /// Return score bounds when this is a validated score presentation.
    pub const fn score_bounds(&self) -> Option<(u64, u64)> {
        self.score
    }

    /// Return logical evidence position when present.
    pub const fn evidence_position(&self) -> Option<(u64, u64)> {
        self.set_position
    }

    fn render_node_with_style(
        &self,
        node_id: NodeId,
        profile: DiagnosticPresentationProfile,
        style: Style,
    ) -> RenderNode {
        let mut node = TextNode::new(self.text(profile)).build(node_id);
        node.appearance.fg = style.fg;
        node.appearance.bg = style.bg;
        node.appearance.bold = style.bold;
        node.appearance.dim = style.dim;
        node.appearance.italic = style.italic;
        node.appearance.underline = style.underline;
        node
    }
}

fn visible_text(
    value: impl Into<CompactString>,
    field: &'static str,
) -> Result<CompactString, DiagnosticPresentationError> {
    let value = value.into();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DiagnosticPresentationError::EmptyText { field });
    }
    Ok(CompactString::from(trimmed))
}
