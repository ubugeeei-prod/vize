use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use super::{HeadlessAnnouncement, SemanticRole, SemanticState};
use crate::{
    layout::Rect,
    render::NodeId,
    terminal::{Cursor, Style},
};

/// Exact terminal cell captured by a headless snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessCell {
    /// Displayed symbol, empty for a wide-character continuation cell.
    pub symbol: CompactString,
    /// Complete terminal style.
    pub style: Style,
    /// Whether this cell continues the preceding wide character.
    pub continuation: bool,
}

/// One semantic node in stable render-tree preorder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSnapshotNode {
    /// Render node identifier.
    pub node_id: NodeId,
    /// Nearest semantic ancestor, skipping decorative render nodes.
    pub parent: Option<NodeId>,
    /// Depth within the semantic tree.
    pub depth: usize,
    /// Platform-neutral role.
    pub role: SemanticRole,
    /// Accessible name.
    pub name: CompactString,
    /// Optional accessible description.
    pub description: Option<CompactString>,
    /// Role-specific state.
    pub state: SemanticState,
    /// Render layout clipped to the viewport.
    pub layout: Rect,
    /// Whether the clipped layout occupies at least one terminal cell.
    pub presented: bool,
}

/// Complete deterministic headless frame assertion surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessSnapshot {
    pub(super) viewport: Rect,
    pub(super) semantics: Vec<SemanticSnapshotNode>,
    pub(super) cells: Vec<HeadlessCell>,
    pub(super) cursor: Cursor,
    pub(super) focus: Option<NodeId>,
    pub(super) announcements: Vec<HeadlessAnnouncement>,
}

impl HeadlessSnapshot {
    /// Return the captured viewport.
    pub const fn viewport(&self) -> Rect {
        self.viewport
    }

    /// Return semantic nodes in stable render-tree preorder.
    pub fn semantics(&self) -> &[SemanticSnapshotNode] {
        &self.semantics
    }

    /// Return row-major terminal cells.
    pub fn cells(&self) -> &[HeadlessCell] {
        &self.cells
    }

    /// Return a cell by terminal coordinate.
    pub fn cell(&self, x: u16, y: u16) -> Option<&HeadlessCell> {
        if x >= self.viewport.width || y >= self.viewport.height {
            return None;
        }
        self.cells
            .get(usize::from(y) * usize::from(self.viewport.width) + usize::from(x))
    }

    /// Return exact cursor state.
    pub const fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Return the focused semantic node, if any.
    pub const fn focus(&self) -> Option<NodeId> {
        self.focus
    }

    /// Return announcements in emission order.
    pub fn announcements(&self) -> &[HeadlessAnnouncement] {
        &self.announcements
    }

    /// Reconstruct one visual row, omitting wide-character continuation cells.
    pub fn row_text(&self, y: u16) -> Option<CompactString> {
        if y >= self.viewport.height {
            return None;
        }
        let width = usize::from(self.viewport.width);
        let start = usize::from(y) * width;
        let mut row = CompactString::new("");
        for cell in &self.cells[start..start + width] {
            if !cell.continuation {
                row.push_str(&cell.symbol);
            }
        }
        Some(row)
    }

    /// Reconstruct all visual rows separated by `\n`, preserving trailing cells.
    pub fn screen_text(&self) -> CompactString {
        let mut screen = CompactString::new("");
        for y in 0..self.viewport.height {
            if y > 0 {
                screen.push('\n');
            }
            if let Some(row) = self.row_text(y) {
                screen.push_str(&row);
            }
        }
        screen
    }
}
