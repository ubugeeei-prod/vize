use crate::layout::Rect;

/// Default terminal width at which list and detail panes become simultaneous.
const DEFAULT_SPLIT_WIDTH: u16 = 80;
/// Default percentage of split width assigned to the finding list.
const DEFAULT_LIST_PERCENT: u8 = 40;
/// Default rows reserved for title, status, and key-hint chrome.
const DEFAULT_CHROME_ROWS: u16 = 3;
/// Default virtualized rows retained above and below each viewport.
const DEFAULT_OVERSCAN: usize = 2;

/// Responsive geometry options for a diagnostic workspace.
///
/// These defaults define the reusable diagnostic workspace contract rather than
/// a Doctor-specific skin. At 80 columns or wider, master and detail panes are
/// both presented; below 80 columns, the workspace stacks panes and chooses the
/// visible pane from semantic focus. The list gets 40% of split width, 3 rows
/// are reserved for application chrome, and each virtualized list retains 2
/// off-screen rows before and after the viewport.
///
/// `split_width` is normalized to at least 3 columns and `list_percent` is
/// clamped to 10..=90 so invalid caller input cannot create empty split panes.
/// `chrome_rows` is capped by the current viewport height during layout, and a
/// zero-height content area remains valid for headless and narrow-terminal
/// assertions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticWorkspaceOptions {
    /// Width at which master and detail panes split. Defaults to 80 columns.
    pub split_width: u16,
    /// Percentage of width assigned to the list in split mode. Defaults to 40.
    pub list_percent: u8,
    /// Rows reserved above or below content. Defaults to 3.
    pub chrome_rows: u16,
    /// Off-screen virtual rows retained on each side. Defaults to 2.
    pub overscan: usize,
}

impl Default for DiagnosticWorkspaceOptions {
    fn default() -> Self {
        Self {
            split_width: DEFAULT_SPLIT_WIDTH,
            list_percent: DEFAULT_LIST_PERCENT,
            chrome_rows: DEFAULT_CHROME_ROWS,
            overscan: DEFAULT_OVERSCAN,
        }
    }
}

impl DiagnosticWorkspaceOptions {
    pub(super) fn normalized(mut self) -> Self {
        self.split_width = self.split_width.max(3);
        self.list_percent = self.list_percent.clamp(10, 90);
        self
    }

    pub(super) const fn overscan(self) -> usize {
        self.overscan
    }
}

/// Responsive pane mode selected from the current viewport width.
///
/// The mode is derived solely from viewport width and
/// [`DiagnosticWorkspaceOptions::split_width`], making resize behavior
/// deterministic and stable under repeated headless renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticWorkspaceMode {
    /// Finding and detail panes are visible side by side.
    Split,
    /// One full-width pane is visible according to semantic focus.
    Stacked,
}

/// Semantic pane selected for presentation in stacked mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticWorkspacePane {
    /// Virtualized finding list.
    Findings,
    /// Detail and related-evidence presentation.
    Detail,
}

/// Deterministic responsive pane rectangles for one terminal viewport.
///
/// The layout contains only terminal-cell geometry. It does not inspect
/// rendered findings, evidence, terminal capabilities, or process state, so
/// callers can assert narrow-terminal overflow and split-pane behavior without
/// opening a terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticWorkspaceLayout {
    width: u16,
    height: u16,
    mode: DiagnosticWorkspaceMode,
    content: Rect,
    findings: Rect,
    detail: Rect,
}

impl DiagnosticWorkspaceLayout {
    pub(super) fn new(width: u16, height: u16, options: DiagnosticWorkspaceOptions) -> Self {
        let chrome = options.chrome_rows.min(height);
        let content = Rect::new(0, chrome, width, height.saturating_sub(chrome));
        let mode = if width >= options.split_width {
            DiagnosticWorkspaceMode::Split
        } else {
            DiagnosticWorkspaceMode::Stacked
        };
        let (findings, detail) = match mode {
            DiagnosticWorkspaceMode::Split if width > 1 => {
                let list_width = (u32::from(width) * u32::from(options.list_percent) / 100)
                    .clamp(1, u32::from(width - 2)) as u16;
                let detail_x = list_width.saturating_add(1).min(width);
                (
                    Rect::new(0, chrome, list_width, content.height),
                    Rect::new(
                        detail_x,
                        chrome,
                        width.saturating_sub(detail_x),
                        content.height,
                    ),
                )
            }
            _ => (content, content),
        };
        Self {
            width,
            height,
            mode,
            content,
            findings,
            detail,
        }
    }

    /// Return the complete terminal width in cells.
    pub const fn width(self) -> u16 {
        self.width
    }

    /// Return the complete terminal height in cells.
    pub const fn height(self) -> u16 {
        self.height
    }

    /// Return the responsive split or stacked mode.
    pub const fn mode(self) -> DiagnosticWorkspaceMode {
        self.mode
    }

    /// Return the content rectangle after reserved chrome rows.
    pub const fn content(self) -> Rect {
        self.content
    }

    /// Return finding pane geometry.
    pub const fn findings(self) -> Rect {
        self.findings
    }

    /// Return detail pane geometry.
    pub const fn detail(self) -> Rect {
        self.detail
    }

    /// Return whether a pane is presented in the current responsive mode.
    pub const fn presents(
        self,
        pane: DiagnosticWorkspacePane,
        active_stacked_pane: DiagnosticWorkspacePane,
    ) -> bool {
        match self.mode {
            DiagnosticWorkspaceMode::Split => true,
            DiagnosticWorkspaceMode::Stacked => matches!(
                (pane, active_stacked_pane),
                (
                    DiagnosticWorkspacePane::Findings,
                    DiagnosticWorkspacePane::Findings
                ) | (
                    DiagnosticWorkspacePane::Detail,
                    DiagnosticWorkspacePane::Detail
                )
            ),
        }
    }
}
