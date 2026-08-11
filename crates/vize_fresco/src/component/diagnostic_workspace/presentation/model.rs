use serde::{Deserialize, Serialize};

use crate::{
    headless::SemanticRole,
    terminal::{Color, Style, TerminalCapabilities},
};

/// Semantic purpose of one value in a diagnostic workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticPresentationKind {
    /// Overall analyzer or rule status.
    Status,
    /// Bounded health or category score.
    Score,
    /// Finding severity.
    Severity,
    /// Finding confidence.
    Confidence,
    /// Estimated or measured impact.
    Impact,
    /// Authored source-code location.
    CodeLocation,
    /// Supporting or related evidence.
    Evidence,
    /// Automatic-fix safety classification.
    FixSafety,
    /// Keyboard shortcut and its action.
    KeyHint,
}

impl DiagnosticPresentationKind {
    /// Return the stable accessible label for this presentation.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::Score => "Score",
            Self::Severity => "Severity",
            Self::Confidence => "Confidence",
            Self::Impact => "Impact",
            Self::CodeLocation => "Location",
            Self::Evidence => "Evidence",
            Self::FixSafety => "Fix safety",
            Self::KeyHint => "Key hint",
        }
    }

    pub(super) const fn role(self, tone: DiagnosticTone) -> SemanticRole {
        match self {
            Self::Score => SemanticRole::Progress,
            Self::CodeLocation | Self::KeyHint => SemanticRole::Code,
            Self::Evidence => SemanticRole::Group,
            Self::Status | Self::Severity
                if matches!(tone, DiagnosticTone::Caution | DiagnosticTone::Negative) =>
            {
                SemanticRole::Alert
            }
            Self::Status | Self::Severity | Self::Confidence | Self::Impact | Self::FixSafety => {
                SemanticRole::Status
            }
        }
    }
}

/// Non-color meaning used for icons, terminal style, and alert semantics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticTone {
    /// No success or urgency meaning.
    #[default]
    Neutral,
    /// A passing, safe, or healthy value.
    Positive,
    /// Informational context that does not require immediate action.
    Informational,
    /// A warning or review-required value.
    Caution,
    /// A failing, unsafe, or blocking value.
    Negative,
}

impl DiagnosticTone {
    /// Return a portable style. Text and markers continue to carry meaning when
    /// a terminal capability profile removes color.
    pub const fn style(self) -> Style {
        match self {
            Self::Neutral => Style::new(),
            Self::Positive => Style::new().fg(Color::Green).bold(),
            Self::Informational => Style::new().fg(Color::Blue),
            Self::Caution => Style::new().fg(Color::Yellow).bold(),
            Self::Negative => Style::new().fg(Color::Red).bold(),
        }
    }

    pub(super) const fn marker(self, unicode: bool) -> &'static str {
        match (self, unicode) {
            (Self::Neutral, true) => "•",
            (Self::Positive, true) => "✓",
            (Self::Informational, true) => "ℹ",
            (Self::Caution, true) => "▲",
            (Self::Negative, true) => "✕",
            (Self::Neutral, false) => "-",
            (Self::Positive, false) => "+",
            (Self::Informational, false) => "i",
            (Self::Caution, false) => "!",
            (Self::Negative, false) => "x",
        }
    }
}

/// Text formatting choices resolved by the embedding application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticPresentationProfile {
    pub(super) unicode: bool,
    pub(super) compact: bool,
}

impl DiagnosticPresentationProfile {
    /// Create an ordinary Unicode profile that includes accessible labels.
    pub const fn unicode() -> Self {
        Self {
            unicode: true,
            compact: false,
        }
    }

    /// Create a conservative ASCII profile that includes accessible labels.
    pub const fn ascii() -> Self {
        Self {
            unicode: false,
            compact: false,
        }
    }

    /// Include or omit the stable label in visual text.
    ///
    /// Compact text still exposes the label through `DiagnosticPresentation::semantic_node`.
    pub const fn with_compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Return whether Unicode markers are enabled.
    pub const fn uses_unicode(self) -> bool {
        self.unicode
    }

    /// Return whether visual labels are omitted for narrow layouts.
    pub const fn is_compact(self) -> bool {
        self.compact
    }
}

impl Default for DiagnosticPresentationProfile {
    fn default() -> Self {
        Self::unicode()
    }
}

impl From<TerminalCapabilities> for DiagnosticPresentationProfile {
    /// Derive text choices from Fresco's resolved terminal contract.
    ///
    /// Unicode follows the capability decision and narrow viewports omit the
    /// repeated visual label. The label remains available in the semantic
    /// node, so compact rendering does not discard accessible meaning.
    fn from(capabilities: TerminalCapabilities) -> Self {
        Self {
            unicode: capabilities.unicode().value(),
            compact: capabilities.is_narrow(),
        }
    }
}
