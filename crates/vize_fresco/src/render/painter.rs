//! Painter for rendering nodes to buffer.

use compact_str::{CompactString, ToCompactString};
use unicode_segmentation::UnicodeSegmentation;

use crate::layout::{Overflow, Rect};
use crate::terminal::{Buffer, Style};
use crate::text::{TextWidth, TextWrap, WrapMode};

use super::{BorderStyle, NodeKind, RenderNode, RenderTree};

/// Painter renders nodes to a terminal buffer.
pub struct Painter<'a> {
    buffer: &'a mut Buffer,
    /// Active ancestor clip in absolute terminal coordinates.
    clip: Rect,
    /// Reusable scratch buffer for wrapped lines (cleared per use).
    wrap_scratch: Vec<CompactString>,
}

impl<'a> Painter<'a> {
    /// Create a new painter.
    pub fn new(buffer: &'a mut Buffer) -> Self {
        Self::with_wrap_scratch(buffer, Vec::new())
    }

    pub(crate) fn with_wrap_scratch(
        buffer: &'a mut Buffer,
        wrap_scratch: Vec<CompactString>,
    ) -> Self {
        let clip = buffer.area();
        Self {
            buffer,
            clip,
            wrap_scratch,
        }
    }

    pub(crate) fn into_wrap_scratch(self) -> Vec<CompactString> {
        self.wrap_scratch
    }

    /// Paint the entire tree to the buffer.
    pub fn paint_tree(&mut self, tree: &RenderTree) {
        if let Some(root_id) = tree.root() {
            self.paint_node_clipped(tree, root_id, self.buffer.area());
        }
    }

    /// Paint a single node and its children.
    pub fn paint_node(&mut self, tree: &RenderTree, id: u64) {
        self.paint_node_clipped(tree, id, self.buffer.area());
    }

    fn paint_node_clipped(&mut self, tree: &RenderTree, id: u64, ancestor_clip: Rect) {
        if let Some(node) = tree.get(id) {
            let node_clip = node
                .layout
                .map_or_else(Rect::empty, |layout| layout.intersection(&ancestor_clip));
            if let Some(layout) = node.layout
                && !node_clip.is_empty()
            {
                let previous_clip = std::mem::replace(&mut self.clip, node_clip);
                self.paint_node_content(node, layout);
                self.clip = previous_clip;
            }

            let child_clip = if node.style.overflow == Overflow::Visible {
                ancestor_clip
            } else {
                node_clip
            };
            for &child_id in &node.children {
                self.paint_node_clipped(tree, child_id, child_clip);
            }
        }
    }

    fn set_string(&mut self, x: u16, y: u16, text: &str, style: Style) {
        self.buffer.set_string_clipped(x, y, text, style, self.clip);
    }

    /// Paint a node's content.
    fn paint_node_content(&mut self, node: &RenderNode, layout: Rect) {
        let style = node.appearance.to_style();

        // Draw border if specified
        let content_area = if let Some(border_style) = node.appearance.border {
            self.paint_border(layout, border_style, style);
            layout.inner(1)
        } else {
            layout
        };

        // Fill background if specified
        if node.appearance.bg.is_some() {
            self.buffer
                .fill(content_area.intersection(&self.clip), ' ', style);
        }

        // Draw content based on node type
        match &node.kind {
            NodeKind::Box => {
                // Box nodes just provide layout, content is drawn by children
            }
            NodeKind::Text(text) => {
                self.paint_text(&text.text, content_area, style, text.wrap_mode);
            }
            NodeKind::Input(input) => {
                self.paint_input(
                    &input.value,
                    &input.placeholder,
                    input.cursor,
                    input.focused,
                    input.mask,
                    input.mask_char,
                    input.selection_start,
                    input.selection_end,
                    content_area,
                    style,
                );
            }
            NodeKind::Raw(raw) => {
                self.paint_raw(&raw.lines, content_area, style);
            }
        }
    }

    /// Paint a border around a rectangle.
    pub fn paint_border(&mut self, rect: Rect, border: BorderStyle, style: Style) {
        if border == BorderStyle::None || rect.width < 2 || rect.height < 2 {
            return;
        }

        let (h, v, tl, tr, bl, br) = border.chars();

        // Top border
        self.set_string(rect.x, rect.y, tl, style);
        for x in rect.x + 1..rect.x + rect.width - 1 {
            self.set_string(x, rect.y, h, style);
        }
        self.set_string(rect.x + rect.width - 1, rect.y, tr, style);

        // Side borders
        for y in rect.y + 1..rect.y + rect.height - 1 {
            self.set_string(rect.x, y, v, style);
            self.set_string(rect.x + rect.width - 1, y, v, style);
        }

        // Bottom border
        self.set_string(rect.x, rect.y + rect.height - 1, bl, style);
        for x in rect.x + 1..rect.x + rect.width - 1 {
            self.set_string(x, rect.y + rect.height - 1, h, style);
        }
        self.set_string(rect.x + rect.width - 1, rect.y + rect.height - 1, br, style);
    }

    /// Paint text content.
    fn paint_text(&mut self, text: &str, area: Rect, style: Style, mode: WrapMode) {
        if area.is_empty() {
            return;
        }

        // Fast path: NoWrap is a single line clipped to the buffer width, which
        // is exactly what `set_string` does. Bypass wrapping entirely so we
        // avoid allocating a Vec + a CompactString copy of the whole string.
        if mode == WrapMode::NoWrap {
            self.set_string(area.x, area.y, text, style);
            return;
        }

        TextWrap::wrap_into(text, area.width as usize, mode, &mut self.wrap_scratch);

        for (i, line) in self.wrap_scratch.iter().enumerate() {
            if i >= area.height as usize {
                break;
            }
            self.buffer
                .set_string_clipped(area.x, area.y + i as u16, line, style, self.clip);
        }
    }

    /// Paint input field with text wrapping support.
    #[expect(
        clippy::too_many_arguments,
        reason = "an input's independent display options"
    )]
    fn paint_input(
        &mut self,
        value: &str,
        placeholder: &str,
        _cursor: usize,
        focused: bool,
        mask: bool,
        mask_char: char,
        selection_start: usize,
        selection_end: usize,
        area: Rect,
        style: Style,
    ) {
        if area.is_empty() {
            return;
        }

        let display_text: CompactString = if value.is_empty() && !focused {
            placeholder.to_compact_string()
        } else if mask {
            std::iter::repeat_n(mask_char, value.graphemes(true).count()).collect()
        } else {
            value.to_compact_string()
        };

        // Wrap text to fit area width
        let area_width = area.width as usize;
        TextWrap::wrap_into(
            &display_text,
            area_width,
            WrapMode::Char,
            &mut self.wrap_scratch,
        );

        // Render each wrapped line
        for (i, line) in self.wrap_scratch.iter().enumerate() {
            if i >= area.height as usize {
                break;
            }
            self.buffer
                .set_string_clipped(area.x, area.y + i as u16, line, style, self.clip);
        }

        if focused && selection_start < selection_end {
            self.paint_input_selection(&display_text, selection_start, selection_end, area, style);
        }

        // Terminal cursor is positioned by render_tree in render.rs
    }

    fn paint_input_selection(
        &mut self,
        value: &str,
        selection_start: usize,
        selection_end: usize,
        area: Rect,
        style: Style,
    ) {
        let mut column = 0_u16;
        let mut row = 0_u16;
        let mut selected_style = style;
        selected_style.reverse = !selected_style.reverse;

        for (index, grapheme) in value.graphemes(true).enumerate() {
            if grapheme == "\n" {
                column = 0;
                row = row.saturating_add(1);
                continue;
            }
            let Ok(width) = u16::try_from(TextWidth::width(grapheme)) else {
                break;
            };
            if width == 0 {
                continue;
            }
            if width > area.width.saturating_sub(column) {
                column = 0;
                row = row.saturating_add(1);
            }
            if row >= area.height {
                break;
            }
            if (selection_start..selection_end).contains(&index) {
                self.set_string(
                    area.x.saturating_add(column),
                    area.y.saturating_add(row),
                    grapheme,
                    selected_style,
                );
            }
            column = column.saturating_add(width);
        }
    }

    /// Paint raw content.
    fn paint_raw(&mut self, lines: &[compact_str::CompactString], area: Rect, style: Style) {
        for (i, line) in lines.iter().enumerate() {
            if i >= area.height as usize {
                break;
            }
            self.set_string(area.x, area.y + i as u16, line, style);
        }
    }

    /// Clear an area.
    pub fn clear(&mut self, area: Rect) {
        self.buffer.clear_area(area.intersection(&self.clip));
    }

    /// Fill an area with a character.
    pub fn fill(&mut self, area: Rect, ch: char, style: Style) {
        self.buffer.fill(area.intersection(&self.clip), ch, style);
    }
}

#[cfg(test)]
mod input_tests {
    use super::Painter;
    use crate::{
        layout::Rect,
        terminal::{Buffer, Style},
    };

    #[test]
    fn input_selection_reverses_complete_wide_graphemes() {
        let mut buffer = Buffer::new(4, 1);
        {
            let mut painter = Painter::new(&mut buffer);
            painter.paint_input(
                "a👋b",
                "",
                2,
                true,
                false,
                '*',
                1,
                2,
                Rect::new(0, 0, 4, 1),
                Style::new(),
            );
        }

        assert!(!buffer.get(0, 0).unwrap().style.reverse);
        assert!(buffer.get(1, 0).unwrap().style.reverse);
        assert!(buffer.get(2, 0).unwrap().style.reverse);
        assert!(!buffer.get(3, 0).unwrap().style.reverse);
    }

    #[test]
    fn password_masking_counts_graphemes_instead_of_scalar_values() {
        let mut buffer = Buffer::new(3, 1);
        {
            let mut painter = Painter::new(&mut buffer);
            painter.paint_input(
                "👋🏽a",
                "",
                2,
                true,
                true,
                '*',
                0,
                0,
                Rect::new(0, 0, 3, 1),
                Style::new(),
            );
        }

        assert_eq!(buffer.get(0, 0).unwrap().symbol.as_str(), "*");
        assert_eq!(buffer.get(1, 0).unwrap().symbol.as_str(), "*");
        assert_eq!(buffer.get(2, 0).unwrap().symbol.as_str(), " ");
    }
}

#[cfg(test)]
mod tests {
    use super::{BorderStyle, Painter, RenderNode, RenderTree};
    use crate::layout::{Overflow, Rect};
    use crate::terminal::{Buffer, Style};
    use crate::text::WrapMode;

    #[test]
    fn test_paint_text() {
        let mut buffer = Buffer::new(20, 5);
        let mut painter = Painter::new(&mut buffer);

        let area = Rect::new(0, 0, 20, 5);
        let style = Style::new();
        painter.paint_text("Hello World", area, style, WrapMode::NoWrap);

        assert_eq!(buffer.get(0, 0).map(|c| c.symbol.as_str()), Some("H"));
        assert_eq!(buffer.get(5, 0).map(|c| c.symbol.as_str()), Some(" "));
    }

    #[test]
    fn hidden_overflow_clips_descendants_in_absolute_coordinates() {
        let mut tree = RenderTree::new();
        let root_id = tree.next_id();
        let mut root = RenderNode::box_node(root_id);
        root.style.overflow = Overflow::Hidden;
        root.layout = Some(Rect::new(1, 0, 3, 1));
        tree.insert_root(root);

        let child_id = tree.next_id();
        let mut child = RenderNode::text_node(child_id, "abcdef");
        child.layout = Some(Rect::new(0, 0, 6, 1));
        tree.insert(child);
        tree.add_child(root_id, child_id);

        let mut buffer = Buffer::new(8, 1);
        Painter::new(&mut buffer).paint_tree(&tree);

        let row: String = (0..8)
            .map(|x| buffer.get(x, 0).unwrap().symbol.as_str())
            .collect();
        assert_eq!(row, " bcd    ");
    }

    #[test]
    fn test_paint_border() {
        let mut buffer = Buffer::new(10, 5);
        let mut painter = Painter::new(&mut buffer);

        let rect = Rect::new(0, 0, 10, 5);
        painter.paint_border(rect, BorderStyle::Single, Style::new());

        assert_eq!(buffer.get(0, 0).map(|c| c.symbol.as_str()), Some("┌"));
        assert_eq!(buffer.get(9, 0).map(|c| c.symbol.as_str()), Some("┐"));
        assert_eq!(buffer.get(0, 4).map(|c| c.symbol.as_str()), Some("└"));
        assert_eq!(buffer.get(9, 4).map(|c| c.symbol.as_str()), Some("┘"));
    }

    #[test]
    fn test_paint_tree() {
        let mut tree = RenderTree::new();
        let id = tree.next_id();
        let mut node = RenderNode::text_node(id, "Hello");
        node.layout = Some(Rect::new(0, 0, 10, 1));
        tree.insert_root(node);

        let mut buffer = Buffer::new(20, 5);
        let mut painter = Painter::new(&mut buffer);
        painter.paint_tree(&tree);

        assert_eq!(buffer.get(0, 0).map(|c| c.symbol.as_str()), Some("H"));
    }
}
