//! Render-tree and accessibility metadata assembly for one Doctor frame.

use vize_fresco::{
    HeadlessPresentation, HeadlessSemanticNode, RenderNode, RenderTree,
    layout::{Dimension, FlexStyle, Inset, LengthPercentageAuto, Position},
    render::{Appearance, NodeKind, RawContent},
    terminal::{Cursor, Style},
    text::WrapMode,
};

use super::StyledLine;

/// One self-contained Doctor frame shared by terminal and headless renderers.
pub(in crate::commands::doctor::tui) struct DoctorFrame {
    tree: RenderTree,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "semantic metadata is consumed by the headless conformance renderer"
        )
    )]
    presentation: HeadlessPresentation,
}

impl DoctorFrame {
    /// Borrow the production render tree for layout and painting.
    pub(in crate::commands::doctor::tui) const fn tree_mut(&mut self) -> &mut RenderTree {
        &mut self.tree
    }

    /// Borrow the non-visual state used by the headless conformance renderer.
    #[cfg(test)]
    pub(in crate::commands::doctor::tui) const fn presentation(&self) -> &HeadlessPresentation {
        &self.presentation
    }
}

/// Bounded builder for a frame whose topology is proportional to the viewport.
pub(super) struct DoctorFrameBuilder {
    tree: RenderTree,
    root: u64,
    semantics: Vec<HeadlessSemanticNode>,
    focus: Option<u64>,
}

impl DoctorFrameBuilder {
    /// Create a full-viewport root with the Doctor application semantic role.
    pub(super) fn new(root_semantic: HeadlessSemanticNode) -> Self {
        let mut tree = RenderTree::new();
        let root = tree.next_id();
        let mut root_node = RenderNode::box_node(root);
        root_node.style.width = Dimension::Percent(100.0);
        root_node.style.height = Dimension::Percent(100.0);
        tree.insert_root(root_node);

        let mut root_semantic = root_semantic;
        root_semantic.node_id = root;
        Self {
            tree,
            root,
            semantics: vec![root_semantic],
            focus: None,
        }
    }

    /// Return the root render node for top-level content.
    pub(super) const fn root(&self) -> u64 {
        self.root
    }

    /// Add an absolutely positioned semantic or decorative container.
    pub(super) fn container(
        &mut self,
        parent: u64,
        area: vize_fresco::Rect,
        semantic: Option<HeadlessSemanticNode>,
        focused: bool,
    ) -> u64 {
        let id = self.tree.next_id();
        let node = RenderNode::box_node(id).with_style(absolute_style(area));
        self.insert(parent, node, semantic, focused)
    }

    /// Add one clipped text row at a position relative to its parent.
    pub(super) fn text(
        &mut self,
        parent: u64,
        area: vize_fresco::Rect,
        text: impl Into<vize_s0::String>,
        style: Style,
        semantic: Option<HeadlessSemanticNode>,
        focused: bool,
    ) -> u64 {
        let id = self.tree.next_id();
        let mut node = RenderNode::text_node(id, text.into());
        node.style = absolute_style(area);
        if let vize_fresco::render::NodeKind::Text(content) = &mut node.kind {
            content.wrap = true;
            content.wrap_mode = WrapMode::TruncateEnd;
        }
        node.appearance = appearance(style);
        self.insert(parent, node, semantic, focused)
    }

    /// Add a one-node vertical rule without one layout node per terminal row.
    pub(super) fn vertical_rule(
        &mut self,
        parent: u64,
        area: vize_fresco::Rect,
        glyph: &str,
        style: Style,
    ) -> u64 {
        let id = self.tree.next_id();
        let node = RenderNode::new(
            id,
            NodeKind::Raw(RawContent::new(std::iter::repeat_n(
                glyph,
                usize::from(area.height),
            ))),
        )
        .with_style(absolute_style(area))
        .with_appearance(appearance(style));
        self.insert(parent, node, None, false)
    }

    /// Add the visible portion of pre-wrapped detail rows.
    pub(super) fn detail_lines(
        &mut self,
        parent: u64,
        width: u16,
        lines: &[StyledLine],
        start: usize,
        height: u16,
    ) {
        for (row, line) in lines
            .iter()
            .skip(start)
            .take(usize::from(height))
            .enumerate()
        {
            self.text(
                parent,
                vize_fresco::Rect::new(0, row as u16, width, 1),
                line.text.clone(),
                line.style,
                line.semantic.clone(),
                line.focused,
            );
        }
    }

    /// Finish semantic metadata with the exact terminal cursor for this frame.
    pub(super) fn finish(self, cursor: Cursor) -> DoctorFrame {
        let mut presentation = HeadlessPresentation::new()
            .with_semantics(self.semantics)
            .with_cursor(cursor);
        if let Some(focus) = self.focus {
            presentation = presentation.with_focus(focus);
        }
        DoctorFrame {
            tree: self.tree,
            presentation,
        }
    }

    fn insert(
        &mut self,
        parent: u64,
        node: RenderNode,
        semantic: Option<HeadlessSemanticNode>,
        focused: bool,
    ) -> u64 {
        let id = node.id;
        self.tree.insert(node);
        self.tree.add_child(parent, id);
        if let Some(mut semantic) = semantic {
            semantic.node_id = id;
            self.semantics.push(semantic);
            if focused {
                assert!(
                    self.focus.is_none(),
                    "Doctor frame has multiple focus targets"
                );
                self.focus = Some(id);
            }
        } else {
            assert!(!focused, "a focus target must expose semantic metadata");
        }
        id
    }
}

fn absolute_style(area: vize_fresco::Rect) -> FlexStyle {
    FlexStyle {
        position: Position::Absolute,
        width: Dimension::Points(f32::from(area.width)),
        height: Dimension::Points(f32::from(area.height)),
        flex_shrink: 0.0,
        inset: Inset {
            top: LengthPercentageAuto::Points(f32::from(area.y)),
            left: LengthPercentageAuto::Points(f32::from(area.x)),
            ..Inset::default()
        },
        ..FlexStyle::default()
    }
}

const fn appearance(style: Style) -> Appearance {
    Appearance {
        fg: style.fg,
        bg: style.bg,
        bold: style.bold,
        dim: style.dim,
        italic: style.italic,
        underline: style.underline,
        inverse: style.reverse,
        blink: style.blink,
        hidden: style.hidden,
        strikethrough: style.strikethrough,
        border: None,
    }
}
