//! Semantic state for large master-detail diagnostic workspaces.

mod command;
mod keymap;
mod layout;
mod presentation;

#[cfg(test)]
mod command_tests;
#[cfg(test)]
mod presentation_tests;
#[cfg(test)]
mod presentation_wire_tests;
#[cfg(test)]
mod tests;

use super::{VirtualListNavigation, VirtualListState, VirtualWindow};

pub use command::{
    DiagnosticWorkspaceAction, DiagnosticWorkspaceCommand, DiagnosticWorkspaceCommandOutcome,
};
pub use keymap::{
    DiagnosticKeyBinding, DiagnosticKeyChord, DiagnosticKeymapError, DiagnosticWorkspaceKeymap,
};
pub use layout::{
    DiagnosticWorkspaceLayout, DiagnosticWorkspaceMode, DiagnosticWorkspaceOptions,
    DiagnosticWorkspacePane,
};
pub use presentation::{
    DiagnosticPresentation, DiagnosticPresentationError, DiagnosticPresentationKind,
    DiagnosticPresentationProfile, DiagnosticTone,
};

/// Semantic keyboard focus within a diagnostic master-detail workspace.
///
/// Focus is independent from the responsive pane currently visible on screen.
/// In split mode, findings and detail can both be presented while exactly one
/// semantic target receives keyboard commands. In stacked mode,
/// [`DiagnosticWorkspaceState::active_stacked_pane`] derives the visible pane
/// from this value. Findings and detail remain focusable in empty or zero-row
/// viewports; evidence is focusable only when a related-evidence item is
/// selected.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiagnosticWorkspaceFocus {
    /// The virtualized finding list.
    #[default]
    Findings,
    /// The selected finding's explanation, location, and fix presentation.
    Detail,
    /// The selected finding's related evidence list.
    Evidence,
}

/// Bounded interaction state for a semantic diagnostic workspace.
///
/// The state owns only stable selected keys and constant-size viewport data. It
/// never owns finding or evidence rows. Callers retain immutable report data and
/// pass stable-key slices when reconciling or navigating. This keeps retained
/// state O(1) while [`VirtualListState`] bounds visible row materialization.
///
/// The default workspace uses [`DiagnosticWorkspaceOptions::default`]: split
/// layout begins at 80 columns, the finding list receives 40% of split width,
/// 3 rows are reserved for chrome, and 2 rows of overscan are retained before
/// and after each visible virtualized list. Construction and mutation perform
/// no terminal I/O, so the same state contract can be asserted through
/// headless tests and reused by non-terminal renderers.
#[derive(Debug, Clone)]
pub struct DiagnosticWorkspaceState<FindingKey, EvidenceKey> {
    findings: VirtualListState<FindingKey>,
    evidence: VirtualListState<EvidenceKey>,
    focus: DiagnosticWorkspaceFocus,
    layout: DiagnosticWorkspaceLayout,
    options: DiagnosticWorkspaceOptions,
    detail_scroll: usize,
    detail_content_rows: usize,
}

impl<FindingKey: Clone + Eq, EvidenceKey: Clone + Eq>
    DiagnosticWorkspaceState<FindingKey, EvidenceKey>
{
    /// Create an empty workspace for an explicit terminal viewport.
    ///
    /// Defaults come from [`DiagnosticWorkspaceOptions::default`]. Construction
    /// performs no terminal I/O and is suitable for headless tests.
    pub fn new(width: u16, height: u16) -> Self {
        Self::with_options(width, height, DiagnosticWorkspaceOptions::default())
    }

    /// Create an empty workspace with explicit layout and overscan options.
    pub fn with_options(width: u16, height: u16, options: DiagnosticWorkspaceOptions) -> Self {
        let options = options.normalized();
        let layout = DiagnosticWorkspaceLayout::new(width, height, options);
        let viewport_rows = usize::from(layout.content().height);
        Self {
            findings: VirtualListState::with_overscan(viewport_rows, options.overscan()),
            evidence: VirtualListState::with_overscan(viewport_rows, options.overscan()),
            focus: DiagnosticWorkspaceFocus::Findings,
            layout,
            options,
            detail_scroll: 0,
            detail_content_rows: 0,
        }
    }

    /// Return the normalized options used for every resize.
    pub const fn options(&self) -> DiagnosticWorkspaceOptions {
        self.options
    }

    /// Return the current split or stacked pane geometry.
    pub const fn layout(&self) -> DiagnosticWorkspaceLayout {
        self.layout
    }

    /// Return the current semantic focus.
    pub const fn focus(&self) -> DiagnosticWorkspaceFocus {
        self.focus
    }

    /// Return the full-width pane selected in stacked mode.
    pub const fn active_stacked_pane(&self) -> DiagnosticWorkspacePane {
        match self.focus {
            DiagnosticWorkspaceFocus::Findings => DiagnosticWorkspacePane::Findings,
            DiagnosticWorkspaceFocus::Detail | DiagnosticWorkspaceFocus::Evidence => {
                DiagnosticWorkspacePane::Detail
            }
        }
    }

    /// Return the virtualized finding-list state.
    pub const fn findings(&self) -> &VirtualListState<FindingKey> {
        &self.findings
    }

    /// Return the virtualized related-evidence state.
    pub const fn evidence(&self) -> &VirtualListState<EvidenceKey> {
        &self.evidence
    }

    /// Return the finding rows that may be materialized for this frame.
    pub fn finding_window(&self) -> VirtualWindow {
        self.findings.window()
    }

    /// Return the evidence rows that may be materialized for this frame.
    pub fn evidence_window(&self) -> VirtualWindow {
        self.evidence.window()
    }

    /// Reconcile a filtered or reordered finding sequence.
    ///
    /// A changed stable selection resets detail and evidence navigation. The
    /// caller should then reconcile evidence for the newly selected finding.
    #[must_use]
    pub fn reconcile_findings(&mut self, keys: &[FindingKey]) -> bool {
        let changed = self.findings.reconcile(keys);
        if changed {
            self.reset_selected_finding_state();
        }
        changed
    }

    /// Select a finding ordinal and reveal it in the virtual viewport.
    #[must_use]
    pub fn select_finding(&mut self, keys: &[FindingKey], index: usize) -> bool {
        let changed = self.findings.select_index(keys, index);
        if changed {
            self.reset_selected_finding_state();
        }
        changed
    }

    /// Apply bounded finding navigation without wrapping.
    #[must_use]
    pub fn navigate_findings(
        &mut self,
        keys: &[FindingKey],
        navigation: VirtualListNavigation,
    ) -> bool {
        let changed = self.findings.navigate(keys, navigation);
        if changed {
            self.reset_selected_finding_state();
        }
        changed
    }

    /// Scroll finding rows independently of selection, for wheel input.
    pub fn scroll_findings(&mut self, rows: isize) -> bool {
        self.findings.scroll_by(rows)
    }

    /// Reconcile stable evidence keys for the selected finding.
    #[must_use]
    pub fn reconcile_evidence(&mut self, keys: &[EvidenceKey]) -> bool {
        let changed = self.evidence.reconcile(keys);
        if keys.is_empty() && self.focus == DiagnosticWorkspaceFocus::Evidence {
            self.focus = DiagnosticWorkspaceFocus::Detail;
        }
        changed
    }

    /// Apply bounded related-evidence navigation without wrapping.
    #[must_use]
    pub fn navigate_evidence(
        &mut self,
        keys: &[EvidenceKey],
        navigation: VirtualListNavigation,
    ) -> bool {
        self.evidence.navigate(keys, navigation)
    }

    /// Set focus when the requested pane is currently available.
    ///
    /// Evidence focus is rejected when no evidence item is selected. Findings
    /// and detail remain focusable even in a zero-sized viewport.
    pub fn set_focus(&mut self, focus: DiagnosticWorkspaceFocus) -> bool {
        if focus == DiagnosticWorkspaceFocus::Evidence && self.evidence.selected_key().is_none() {
            return false;
        }
        let changed = self.focus != focus;
        self.focus = focus;
        changed
    }

    /// Move focus forward through available semantic panes.
    pub fn focus_next(&mut self) -> bool {
        let next = match (self.focus, self.evidence.selected_key().is_some()) {
            (DiagnosticWorkspaceFocus::Findings, _) => DiagnosticWorkspaceFocus::Detail,
            (DiagnosticWorkspaceFocus::Detail, true) => DiagnosticWorkspaceFocus::Evidence,
            (DiagnosticWorkspaceFocus::Detail | DiagnosticWorkspaceFocus::Evidence, _) => {
                DiagnosticWorkspaceFocus::Findings
            }
        };
        self.set_focus(next)
    }

    /// Move focus backward through available semantic panes.
    pub fn focus_previous(&mut self) -> bool {
        let next = match (self.focus, self.evidence.selected_key().is_some()) {
            (DiagnosticWorkspaceFocus::Findings, true) => DiagnosticWorkspaceFocus::Evidence,
            (DiagnosticWorkspaceFocus::Findings, false) => DiagnosticWorkspaceFocus::Detail,
            (DiagnosticWorkspaceFocus::Detail, _) => DiagnosticWorkspaceFocus::Findings,
            (DiagnosticWorkspaceFocus::Evidence, _) => DiagnosticWorkspaceFocus::Detail,
        };
        self.set_focus(next)
    }

    /// Resize pane geometry while preserving stable finding and evidence keys.
    pub fn resize(&mut self, width: u16, height: u16) -> bool {
        let next = DiagnosticWorkspaceLayout::new(width, height, self.options);
        if next == self.layout {
            return false;
        }
        self.layout = next;
        let rows = usize::from(next.content().height);
        self.findings.set_viewport_len(rows);
        self.evidence.set_viewport_len(rows);
        self.clamp_detail_scroll();
        true
    }

    /// Set the selected detail's full row count and clamp overflow.
    pub fn set_detail_content_rows(&mut self, rows: usize) -> bool {
        if self.detail_content_rows == rows {
            return false;
        }
        self.detail_content_rows = rows;
        self.clamp_detail_scroll();
        true
    }

    /// Return the first visible detail row.
    pub const fn detail_scroll(&self) -> usize {
        self.detail_scroll
    }

    /// Return detail rows that remain below the viewport.
    pub fn detail_rows_below(&self) -> usize {
        self.detail_content_rows.saturating_sub(
            self.detail_scroll
                .saturating_add(usize::from(self.layout.content().height)),
        )
    }

    /// Scroll selected-detail rows with saturating, explicitly bounded motion.
    pub fn scroll_detail(&mut self, rows: isize) -> bool {
        let max = self.max_detail_scroll();
        let next = if rows.is_negative() {
            self.detail_scroll.saturating_sub(rows.unsigned_abs())
        } else {
            self.detail_scroll.saturating_add(rows as usize)
        }
        .min(max);
        let changed = next != self.detail_scroll;
        self.detail_scroll = next;
        changed
    }

    fn reset_selected_finding_state(&mut self) {
        let _ = self.evidence.reconcile(&[]);
        self.detail_scroll = 0;
        self.detail_content_rows = 0;
        if self.focus == DiagnosticWorkspaceFocus::Evidence {
            self.focus = DiagnosticWorkspaceFocus::Detail;
        }
    }

    fn clamp_detail_scroll(&mut self) {
        self.detail_scroll = self.detail_scroll.min(self.max_detail_scroll());
    }

    fn max_detail_scroll(&self) -> usize {
        self.detail_content_rows
            .saturating_sub(usize::from(self.layout.content().height))
    }
}
