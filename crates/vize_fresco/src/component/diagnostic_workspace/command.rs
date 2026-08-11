//! Semantic commands shared by diagnostic workspace input adapters.

use serde::{Deserialize, Serialize};

use super::{DiagnosticWorkspaceFocus, DiagnosticWorkspaceState, VirtualListNavigation};

/// A semantic diagnostic-workspace command.
///
/// Commands intentionally describe user intent instead of physical keys. A
/// terminal, GUI, accessibility adapter, or test can therefore drive the same
/// workspace state without duplicating navigation behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticWorkspaceCommand {
    /// Select the following finding without wrapping.
    NextFinding,
    /// Select the preceding finding without wrapping.
    PreviousFinding,
    /// Select a finding one viewport toward the end.
    PageDownFindings,
    /// Select a finding one viewport toward the start.
    PageUpFindings,
    /// Select the first finding.
    FirstFinding,
    /// Select the last finding.
    LastFinding,
    /// Select the following related-evidence item without wrapping.
    NextEvidence,
    /// Select the preceding related-evidence item without wrapping.
    PreviousEvidence,
    /// Scroll the detail presentation down by one row.
    ScrollDetailDown,
    /// Scroll the detail presentation up by one row.
    ScrollDetailUp,
    /// Scroll the detail presentation one viewport toward the end.
    PageDownDetail,
    /// Scroll the detail presentation one viewport toward the start.
    PageUpDetail,
    /// Move keyboard focus to the following available semantic pane.
    FocusNext,
    /// Move keyboard focus to the preceding available semantic pane.
    FocusPrevious,
    /// Request the following category-filter value.
    NextCategory,
    /// Request the preceding category-filter value.
    PreviousCategory,
    /// Request the following severity-filter value.
    NextSeverity,
    /// Request the preceding severity-filter value.
    PreviousSeverity,
    /// Request interactive search.
    Search,
    /// Request opening the selected finding's primary source location.
    OpenSource,
    /// Request the keyboard help presentation.
    Help,
    /// Request a clean interactive-session exit.
    Exit,
}

impl DiagnosticWorkspaceCommand {
    /// Return a stable machine-readable command name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NextFinding => "next-finding",
            Self::PreviousFinding => "previous-finding",
            Self::PageDownFindings => "page-down-findings",
            Self::PageUpFindings => "page-up-findings",
            Self::FirstFinding => "first-finding",
            Self::LastFinding => "last-finding",
            Self::NextEvidence => "next-evidence",
            Self::PreviousEvidence => "previous-evidence",
            Self::ScrollDetailDown => "scroll-detail-down",
            Self::ScrollDetailUp => "scroll-detail-up",
            Self::PageDownDetail => "page-down-detail",
            Self::PageUpDetail => "page-up-detail",
            Self::FocusNext => "focus-next",
            Self::FocusPrevious => "focus-previous",
            Self::NextCategory => "next-category",
            Self::PreviousCategory => "previous-category",
            Self::NextSeverity => "next-severity",
            Self::PreviousSeverity => "previous-severity",
            Self::Search => "search",
            Self::OpenSource => "open-source",
            Self::Help => "help",
            Self::Exit => "exit",
        }
    }

    /// Return a concise human-readable description for help and key hints.
    pub const fn description(self) -> &'static str {
        match self {
            Self::NextFinding => "Next finding",
            Self::PreviousFinding => "Previous finding",
            Self::PageDownFindings => "Next page of findings",
            Self::PageUpFindings => "Previous page of findings",
            Self::FirstFinding => "First finding",
            Self::LastFinding => "Last finding",
            Self::NextEvidence => "Next evidence",
            Self::PreviousEvidence => "Previous evidence",
            Self::ScrollDetailDown => "Scroll detail down",
            Self::ScrollDetailUp => "Scroll detail up",
            Self::PageDownDetail => "Scroll detail one page down",
            Self::PageUpDetail => "Scroll detail one page up",
            Self::FocusNext => "Focus next pane",
            Self::FocusPrevious => "Focus previous pane",
            Self::NextCategory => "Next category filter",
            Self::PreviousCategory => "Previous category filter",
            Self::NextSeverity => "Next severity filter",
            Self::PreviousSeverity => "Previous severity filter",
            Self::Search => "Search findings",
            Self::OpenSource => "Open source location",
            Self::Help => "Show keyboard help",
            Self::Exit => "Exit",
        }
    }

    /// Return whether holding the key may safely repeat this command.
    ///
    /// Navigation and scrolling repeat. Focus, filters, modal actions, source
    /// actions, and exit are press-only so one long key press cannot cycle state
    /// unpredictably or launch an action more than once.
    pub const fn accepts_repeat(self) -> bool {
        matches!(
            self,
            Self::NextFinding
                | Self::PreviousFinding
                | Self::PageDownFindings
                | Self::PageUpFindings
                | Self::FirstFinding
                | Self::LastFinding
                | Self::NextEvidence
                | Self::PreviousEvidence
                | Self::ScrollDetailDown
                | Self::ScrollDetailUp
                | Self::PageDownDetail
                | Self::PageUpDetail
        )
    }

    const fn action(self) -> Option<DiagnosticWorkspaceAction> {
        match self {
            Self::NextCategory => Some(DiagnosticWorkspaceAction::NextCategory),
            Self::PreviousCategory => Some(DiagnosticWorkspaceAction::PreviousCategory),
            Self::NextSeverity => Some(DiagnosticWorkspaceAction::NextSeverity),
            Self::PreviousSeverity => Some(DiagnosticWorkspaceAction::PreviousSeverity),
            Self::Search => Some(DiagnosticWorkspaceAction::Search),
            Self::OpenSource => Some(DiagnosticWorkspaceAction::OpenSource),
            Self::Help => Some(DiagnosticWorkspaceAction::Help),
            Self::Exit => Some(DiagnosticWorkspaceAction::Exit),
            _ => None,
        }
    }
}

/// An application-owned action emitted by a workspace command.
///
/// Fresco owns deterministic navigation state, but it does not own filter
/// vocabularies, an editor launcher, modal presentation, or process lifetime.
/// Those operations are returned to the application through this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticWorkspaceAction {
    /// Advance the application's category filter.
    NextCategory,
    /// Move the application's category filter backward.
    PreviousCategory,
    /// Advance the application's severity filter.
    NextSeverity,
    /// Move the application's severity filter backward.
    PreviousSeverity,
    /// Open the application's search interaction.
    Search,
    /// Open the selected finding's primary source location.
    OpenSource,
    /// Open the keyboard help presentation.
    Help,
    /// End the interactive session through its normal restoration path.
    Exit,
}

/// Result of applying a semantic command to workspace state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticWorkspaceCommandOutcome {
    /// Fresco changed selection, focus, or scroll state.
    Changed,
    /// The command was valid but the relevant list or scroll range was empty or
    /// already at its non-wrapping boundary.
    Boundary,
    /// The application must perform an operation outside Fresco-owned state.
    Dispatch(DiagnosticWorkspaceAction),
}

impl<FindingKey: Clone + Eq, EvidenceKey: Clone + Eq>
    DiagnosticWorkspaceState<FindingKey, EvidenceKey>
{
    /// Apply a semantic command to the workspace's bounded state.
    ///
    /// `finding_keys` and `evidence_keys` remain application-owned immutable
    /// report data. Navigation automatically synchronizes stale stable-key
    /// selections. Application operations are returned as [`Dispatch`](DiagnosticWorkspaceCommandOutcome::Dispatch).
    #[must_use]
    pub fn apply_command(
        &mut self,
        command: DiagnosticWorkspaceCommand,
        finding_keys: &[FindingKey],
        evidence_keys: &[EvidenceKey],
    ) -> DiagnosticWorkspaceCommandOutcome {
        if let Some(action) = command.action() {
            if action == DiagnosticWorkspaceAction::OpenSource
                && self.findings.selected_key().is_none()
            {
                return DiagnosticWorkspaceCommandOutcome::Boundary;
            }
            return DiagnosticWorkspaceCommandOutcome::Dispatch(action);
        }

        let changed = match command {
            DiagnosticWorkspaceCommand::NextFinding => {
                self.navigate_findings(finding_keys, VirtualListNavigation::Next)
            }
            DiagnosticWorkspaceCommand::PreviousFinding => {
                self.navigate_findings(finding_keys, VirtualListNavigation::Previous)
            }
            DiagnosticWorkspaceCommand::PageDownFindings => {
                self.navigate_findings(finding_keys, VirtualListNavigation::PageDown)
            }
            DiagnosticWorkspaceCommand::PageUpFindings => {
                self.navigate_findings(finding_keys, VirtualListNavigation::PageUp)
            }
            DiagnosticWorkspaceCommand::FirstFinding => {
                self.navigate_findings(finding_keys, VirtualListNavigation::First)
            }
            DiagnosticWorkspaceCommand::LastFinding => {
                self.navigate_findings(finding_keys, VirtualListNavigation::Last)
            }
            DiagnosticWorkspaceCommand::NextEvidence => {
                self.navigate_and_focus_evidence(evidence_keys, VirtualListNavigation::Next)
            }
            DiagnosticWorkspaceCommand::PreviousEvidence => {
                self.navigate_and_focus_evidence(evidence_keys, VirtualListNavigation::Previous)
            }
            DiagnosticWorkspaceCommand::ScrollDetailDown => self.scroll_detail(1),
            DiagnosticWorkspaceCommand::ScrollDetailUp => self.scroll_detail(-1),
            DiagnosticWorkspaceCommand::PageDownDetail => {
                self.scroll_detail(self.detail_page_rows() as isize)
            }
            DiagnosticWorkspaceCommand::PageUpDetail => {
                self.scroll_detail(-(self.detail_page_rows() as isize))
            }
            DiagnosticWorkspaceCommand::FocusNext => self.focus_next(),
            DiagnosticWorkspaceCommand::FocusPrevious => self.focus_previous(),
            _ => unreachable!("application commands returned before state dispatch"),
        };

        if changed {
            DiagnosticWorkspaceCommandOutcome::Changed
        } else {
            DiagnosticWorkspaceCommandOutcome::Boundary
        }
    }

    fn navigate_and_focus_evidence(
        &mut self,
        evidence_keys: &[EvidenceKey],
        navigation: VirtualListNavigation,
    ) -> bool {
        let navigated = self.navigate_evidence(evidence_keys, navigation);
        let focused = self.set_focus(DiagnosticWorkspaceFocus::Evidence);
        navigated || focused
    }

    fn detail_page_rows(&self) -> usize {
        usize::from(self.layout.content().height)
            .saturating_sub(1)
            .max(1)
    }
}
