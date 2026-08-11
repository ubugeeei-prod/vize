use rustc_hash::{FxHashMap, FxHashSet};
use thiserror::Error;

use super::{
    HeadlessCell, HeadlessPresentation, HeadlessSemanticNode, HeadlessSnapshot,
    SemanticSnapshotNode,
};
use crate::{
    layout::Rect,
    render::{NodeId, Painter, RenderTree},
    terminal::Buffer,
};

/// Default maximum number of cells allocated by a headless viewport.
pub const DEFAULT_HEADLESS_CELL_BUDGET: usize = 1_000_000;

/// Invalid headless presentation or viewport.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HeadlessRenderError {
    /// Requested viewport exceeds the configured allocation budget.
    #[error("headless viewport {width}x{height} requires {required} cells; budget is {budget}")]
    CellBudgetExceeded {
        /// Requested width.
        width: u16,
        /// Requested height.
        height: u16,
        /// Required row-major cells.
        required: usize,
        /// Configured limit.
        budget: usize,
    },
    /// Multiple semantic records reference the same render node.
    #[error("semantic render node {0} is declared more than once")]
    DuplicateSemanticNode(NodeId),
    /// Semantic metadata references a render node that does not exist.
    #[error("semantic render node {0} does not exist")]
    UnknownSemanticNode(NodeId),
    /// Semantic metadata references a node detached from the render root.
    #[error("semantic render node {0} is detached from the render root")]
    DetachedSemanticNode(NodeId),
    /// A semantic node has no accessible name.
    #[error("semantic render node {0} has an empty accessible name")]
    EmptySemanticName(NodeId),
    /// A supplied description contains no accessible text.
    #[error("semantic render node {0} has an empty accessible description")]
    EmptySemanticDescription(NodeId),
    /// Heading level is outside 1 through 6.
    #[error(
        "semantic heading render node {node_id} has invalid level {level}; expected 1 through 6"
    )]
    InvalidHeadingLevel {
        /// Invalid render node.
        node_id: NodeId,
        /// Invalid heading level.
        level: u8,
    },
    /// A non-heading role supplied a heading level.
    #[error("non-heading semantic render node {0} supplies a heading level")]
    UnexpectedHeadingLevel(NodeId),
    /// Logical set metadata supplied only a position or only a size.
    #[error("semantic render node {0} must supply set position and size together")]
    IncompleteSetPosition(NodeId),
    /// Logical set metadata is zero or outside the declared set.
    #[error("semantic render node {node_id} has invalid set position {position} of {set_size}")]
    InvalidSetPosition {
        /// Invalid render node.
        node_id: NodeId,
        /// One-based position.
        position: u64,
        /// Declared set size.
        set_size: u64,
    },
    /// Focus references a node without semantic metadata.
    #[error("focused render node {0} is not semantic")]
    UnknownFocus(NodeId),
    /// Focus references a semantic node outside the presented viewport.
    #[error("focused semantic render node {0} is not presented in the viewport")]
    FocusNotPresented(NodeId),
    /// A visible cursor falls outside the viewport.
    #[error("visible cursor ({x}, {y}) is outside headless viewport {width}x{height}")]
    CursorOutsideViewport {
        /// Cursor column.
        x: u16,
        /// Cursor row.
        y: u16,
        /// Viewport width.
        width: u16,
        /// Viewport height.
        height: u16,
    },
    /// An announcement message is empty.
    #[error("announcement {0} has an empty message")]
    EmptyAnnouncement(usize),
    /// An announcement source has no semantic metadata.
    #[error("announcement {index} references non-semantic render node {node_id}")]
    UnknownAnnouncementSource {
        /// Zero-based announcement index.
        index: usize,
        /// Invalid render node.
        node_id: NodeId,
    },
}

/// Reusable terminal-free renderer for deterministic frame snapshots.
pub struct HeadlessRenderer {
    buffer: Buffer,
    cell_budget: usize,
}

impl HeadlessRenderer {
    /// Create a renderer using [`DEFAULT_HEADLESS_CELL_BUDGET`].
    pub fn new(width: u16, height: u16) -> Result<Self, HeadlessRenderError> {
        Self::with_cell_budget(width, height, DEFAULT_HEADLESS_CELL_BUDGET)
    }

    /// Create a renderer with an explicit maximum cell allocation.
    pub fn with_cell_budget(
        width: u16,
        height: u16,
        cell_budget: usize,
    ) -> Result<Self, HeadlessRenderError> {
        validate_cell_budget(width, height, cell_budget)?;
        Ok(Self {
            buffer: Buffer::new(width, height),
            cell_budget,
        })
    }

    /// Return the current viewport.
    pub fn viewport(&self) -> Rect {
        self.buffer.area()
    }

    /// Resize and clear the reusable buffer after validating its allocation.
    pub fn resize(&mut self, width: u16, height: u16) -> Result<bool, HeadlessRenderError> {
        validate_cell_budget(width, height, self.cell_budget)?;
        if self.buffer.width() == width && self.buffer.height() == height {
            return Ok(false);
        }
        self.buffer.resize(width, height);
        Ok(true)
    }

    /// Render one complete snapshot through production layout and paint paths.
    pub fn render(
        &mut self,
        tree: &mut RenderTree,
        presentation: &HeadlessPresentation,
    ) -> Result<HeadlessSnapshot, HeadlessRenderError> {
        let semantic_map = validate_presentation(tree, presentation)?;
        validate_cursor(self.viewport(), presentation)?;

        tree.compute_layout(self.buffer.width(), self.buffer.height());
        self.buffer.clear();
        Painter::new(&mut self.buffer).paint_tree(tree);

        let semantics = snapshot_semantics(tree, self.viewport(), &semantic_map)?;
        if let Some(focus) = presentation.focus
            && !semantics
                .iter()
                .any(|node| node.node_id == focus && node.presented)
        {
            return Err(HeadlessRenderError::FocusNotPresented(focus));
        }

        let cells = self
            .buffer
            .iter()
            .map(|(_, _, cell)| HeadlessCell {
                symbol: cell.symbol.clone(),
                style: cell.style,
                continuation: cell.is_continuation,
            })
            .collect();

        Ok(HeadlessSnapshot {
            viewport: self.viewport(),
            semantics,
            cells,
            cursor: presentation.cursor,
            focus: presentation.focus,
            announcements: presentation.announcements.clone(),
        })
    }
}

fn validate_cell_budget(width: u16, height: u16, budget: usize) -> Result<(), HeadlessRenderError> {
    let required = usize::from(width).saturating_mul(usize::from(height));
    if required > budget {
        return Err(HeadlessRenderError::CellBudgetExceeded {
            width,
            height,
            required,
            budget,
        });
    }
    Ok(())
}

fn validate_cursor(
    viewport: Rect,
    presentation: &HeadlessPresentation,
) -> Result<(), HeadlessRenderError> {
    let cursor = presentation.cursor;
    if cursor.visible && !viewport.contains(cursor.x, cursor.y) {
        return Err(HeadlessRenderError::CursorOutsideViewport {
            x: cursor.x,
            y: cursor.y,
            width: viewport.width,
            height: viewport.height,
        });
    }
    Ok(())
}

fn validate_presentation<'a>(
    tree: &RenderTree,
    presentation: &'a HeadlessPresentation,
) -> Result<FxHashMap<NodeId, &'a HeadlessSemanticNode>, HeadlessRenderError> {
    let mut semantics = FxHashMap::default();
    for semantic in &presentation.semantics {
        if tree.get(semantic.node_id).is_none() {
            return Err(HeadlessRenderError::UnknownSemanticNode(semantic.node_id));
        }
        if semantic.name.trim().is_empty() {
            return Err(HeadlessRenderError::EmptySemanticName(semantic.node_id));
        }
        if semantic
            .description
            .as_deref()
            .is_some_and(|description| description.trim().is_empty())
        {
            return Err(HeadlessRenderError::EmptySemanticDescription(
                semantic.node_id,
            ));
        }
        match (semantic.role, semantic.state.level) {
            (super::SemanticRole::Heading, Some(1..=6)) | (_, None) => {}
            (super::SemanticRole::Heading, Some(level)) => {
                return Err(HeadlessRenderError::InvalidHeadingLevel {
                    node_id: semantic.node_id,
                    level,
                });
            }
            (_, Some(_)) => {
                return Err(HeadlessRenderError::UnexpectedHeadingLevel(
                    semantic.node_id,
                ));
            }
        }
        match (semantic.state.position, semantic.state.set_size) {
            (None, None) => {}
            (Some(position), Some(set_size)) if position > 0 && position <= set_size => {}
            (Some(position), Some(set_size)) => {
                return Err(HeadlessRenderError::InvalidSetPosition {
                    node_id: semantic.node_id,
                    position,
                    set_size,
                });
            }
            _ => {
                return Err(HeadlessRenderError::IncompleteSetPosition(semantic.node_id));
            }
        }
        if semantics.insert(semantic.node_id, semantic).is_some() {
            return Err(HeadlessRenderError::DuplicateSemanticNode(semantic.node_id));
        }
    }
    if let Some(focus) = presentation.focus
        && !semantics.contains_key(&focus)
    {
        return Err(HeadlessRenderError::UnknownFocus(focus));
    }
    for (index, announcement) in presentation.announcements.iter().enumerate() {
        if announcement.message.trim().is_empty() {
            return Err(HeadlessRenderError::EmptyAnnouncement(index));
        }
        if let Some(node_id) = announcement.source
            && !semantics.contains_key(&node_id)
        {
            return Err(HeadlessRenderError::UnknownAnnouncementSource { index, node_id });
        }
    }
    Ok(semantics)
}

fn snapshot_semantics(
    tree: &RenderTree,
    viewport: Rect,
    semantics: &FxHashMap<NodeId, &HeadlessSemanticNode>,
) -> Result<Vec<SemanticSnapshotNode>, HeadlessRenderError> {
    let mut snapshot = Vec::with_capacity(semantics.len());
    let mut visited = FxHashSet::default();
    let mut stack = tree
        .root()
        .into_iter()
        .map(|root| (root, None, 0_usize))
        .collect::<Vec<_>>();

    while let Some((node_id, semantic_parent, semantic_depth)) = stack.pop() {
        let Some(render_node) = tree.get(node_id) else {
            continue;
        };
        let (next_parent, next_depth) = if let Some(semantic) = semantics.get(&node_id) {
            visited.insert(node_id);
            let layout = render_node
                .layout
                .unwrap_or_default()
                .intersection(&viewport);
            snapshot.push(SemanticSnapshotNode {
                node_id,
                parent: semantic_parent,
                depth: semantic_depth,
                role: semantic.role,
                name: semantic.name.clone(),
                description: semantic.description.clone(),
                state: semantic.state.clone(),
                layout,
                presented: !layout.is_empty(),
            });
            (Some(node_id), semantic_depth.saturating_add(1))
        } else {
            (semantic_parent, semantic_depth)
        };
        for child in render_node.children.iter().rev() {
            stack.push((*child, next_parent, next_depth));
        }
    }

    if let Some(detached) = semantics
        .keys()
        .filter(|node_id| !visited.contains(node_id))
        .min()
    {
        return Err(HeadlessRenderError::DetachedSemanticNode(*detached));
    }
    Ok(snapshot)
}
