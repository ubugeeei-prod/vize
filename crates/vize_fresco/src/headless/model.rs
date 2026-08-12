use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use crate::{render::NodeId, terminal::Cursor};

/// Platform-neutral role exposed by a rendered semantic node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticRole {
    /// Root application surface.
    Application,
    /// Primary content region.
    Main,
    /// Navigation region.
    Navigation,
    /// Named region.
    Region,
    /// Related group of controls or content.
    Group,
    /// Heading with an optional level in [`SemanticState`].
    Heading,
    /// Collection of related items.
    List,
    /// One item in a collection.
    ListItem,
    /// Non-urgent status update.
    Status,
    /// Urgent error or warning.
    Alert,
    /// Progress or score indicator.
    Progress,
    /// Read-only text.
    Text,
    /// Editable text input.
    Input,
    /// Search input.
    SearchBox,
    /// Action control.
    Button,
    /// Navigable link.
    Link,
    /// Source code or machine-readable identifier.
    Code,
}

/// Optional state attached to a semantic node.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticState {
    /// Human-readable current value, such as `92 / 100`.
    pub value: Option<CompactString>,
    /// Heading level when the role is [`SemanticRole::Heading`].
    pub level: Option<u8>,
    /// One-based item position within a logical set.
    pub position: Option<u64>,
    /// Total logical set size, including virtualized off-screen items.
    pub set_size: Option<u64>,
    /// Whether this item is selected.
    pub selected: bool,
    /// Whether this item is disabled.
    pub disabled: bool,
    /// Whether expandable content is open.
    pub expanded: Option<bool>,
    /// Whether the region is waiting for an update.
    pub busy: bool,
    /// Whether an input value is read-only.
    pub read_only: bool,
}

impl SemanticState {
    /// Set a human-readable value.
    pub fn with_value(mut self, value: impl Into<CompactString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set a heading level.
    pub const fn with_level(mut self, level: u8) -> Self {
        self.level = Some(level);
        self
    }

    /// Set one-based position and total size for a logical collection.
    pub const fn with_set_position(mut self, position: u64, set_size: u64) -> Self {
        self.position = Some(position);
        self.set_size = Some(set_size);
        self
    }

    /// Mark the node selected or unselected.
    pub const fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// Semantic metadata associated with one render-tree node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessSemanticNode {
    /// Render node carrying this semantic meaning.
    pub node_id: NodeId,
    /// Platform-neutral role.
    pub role: SemanticRole,
    /// Accessible name used in snapshots and announcements.
    pub name: CompactString,
    /// Optional longer explanation.
    pub description: Option<CompactString>,
    /// Optional role state.
    pub state: SemanticState,
}

impl HeadlessSemanticNode {
    /// Create semantic metadata with a required accessible name.
    pub fn new(node_id: NodeId, role: SemanticRole, name: impl Into<CompactString>) -> Self {
        Self {
            node_id,
            role,
            name: name.into(),
            description: None,
            state: SemanticState::default(),
        }
    }

    /// Set the longer accessible description.
    pub fn with_description(mut self, description: impl Into<CompactString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set role-specific state.
    pub fn with_state(mut self, state: SemanticState) -> Self {
        self.state = state;
        self
    }
}

/// Urgency used when asserting a live announcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnnouncementPoliteness {
    /// Announce after the current interaction finishes.
    Polite,
    /// Interrupt because the message requires immediate attention.
    Assertive,
}

/// One semantic announcement emitted by the rendered frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessAnnouncement {
    /// Announcement urgency.
    pub politeness: AnnouncementPoliteness,
    /// Exact message exposed to assistive output.
    pub message: CompactString,
    /// Optional semantic node that caused the announcement.
    pub source: Option<NodeId>,
}

impl HeadlessAnnouncement {
    /// Create an announcement without a source node.
    pub fn new(politeness: AnnouncementPoliteness, message: impl Into<CompactString>) -> Self {
        Self {
            politeness,
            message: message.into(),
            source: None,
        }
    }

    /// Associate the announcement with a semantic node.
    pub const fn with_source(mut self, source: NodeId) -> Self {
        self.source = Some(source);
        self
    }
}

/// Per-frame non-visual state supplied to [`super::HeadlessRenderer`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessPresentation {
    pub(super) semantics: Vec<HeadlessSemanticNode>,
    pub(super) focus: Option<NodeId>,
    pub(super) cursor: Cursor,
    pub(super) announcements: Vec<HeadlessAnnouncement>,
}

impl HeadlessPresentation {
    /// Create a presentation with a hidden cursor and no semantic nodes.
    pub fn new() -> Self {
        let mut cursor = Cursor::new();
        cursor.hide();
        Self {
            semantics: Vec::new(),
            focus: None,
            cursor,
            announcements: Vec::new(),
        }
    }

    /// Supply semantic metadata. Input order does not affect snapshot order.
    pub fn with_semantics(
        mut self,
        semantics: impl IntoIterator<Item = HeadlessSemanticNode>,
    ) -> Self {
        self.semantics = semantics.into_iter().collect();
        self
    }

    /// Set the focused semantic render node.
    pub const fn with_focus(mut self, focus: NodeId) -> Self {
        self.focus = Some(focus);
        self
    }

    /// Set exact terminal cursor state.
    pub const fn with_cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = cursor;
        self
    }

    /// Supply announcements in emission order.
    pub fn with_announcements(
        mut self,
        announcements: impl IntoIterator<Item = HeadlessAnnouncement>,
    ) -> Self {
        self.announcements = announcements.into_iter().collect();
        self
    }
}

impl Default for HeadlessPresentation {
    fn default() -> Self {
        Self::new()
    }
}
