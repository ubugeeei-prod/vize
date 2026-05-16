//! Box component - container with flexbox layout.

use crate::layout::{AlignItems, Dimension, Edges, FlexDirection, FlexStyle, Gap, JustifyContent};
use crate::render::{Appearance, BorderStyle, NodeKind, RenderNode};
use crate::terminal::Color;

/// Builder for Box nodes.
#[derive(Debug, Clone, Default)]
pub struct BoxNode {
    style: FlexStyle,
    appearance: Appearance,
}

impl BoxNode {
    /// Create a new box node builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set flex direction to column.
    pub fn column(mut self) -> Self {
        self.style.flex_direction = FlexDirection::Column;
        self
    }

    /// Set flex direction to row.
    pub fn row(mut self) -> Self {
        self.style.flex_direction = FlexDirection::Row;
        self
    }

    /// Set width.
    pub fn width(mut self, width: f32) -> Self {
        self.style.width = Dimension::Points(width);
        self
    }

    /// Set width as percentage.
    pub fn width_percent(mut self, percent: f32) -> Self {
        self.style.width = Dimension::Percent(percent);
        self
    }

    /// Set height.
    pub fn height(mut self, height: f32) -> Self {
        self.style.height = Dimension::Points(height);
        self
    }

    /// Set height as percentage.
    pub fn height_percent(mut self, percent: f32) -> Self {
        self.style.height = Dimension::Percent(percent);
        self
    }

    /// Set padding on all sides.
    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding = Edges::all(padding);
        self
    }

    /// Set padding with different vertical and horizontal values.
    pub fn padding_xy(mut self, vertical: f32, horizontal: f32) -> Self {
        self.style.padding = Edges::symmetric(vertical, horizontal);
        self
    }

    /// Set margin on all sides.
    pub fn margin(mut self, margin: f32) -> Self {
        self.style.margin = Edges::all(margin);
        self
    }

    /// Set gap between children.
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = Gap::all(gap);
        self
    }

    /// Set flex grow.
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
        self
    }

    /// Set flex shrink.
    pub fn shrink(mut self, shrink: f32) -> Self {
        self.style.flex_shrink = shrink;
        self
    }

    /// Align items to start.
    pub fn align_start(mut self) -> Self {
        self.style.align_items = AlignItems::FlexStart;
        self
    }

    /// Align items to center.
    pub fn align_center(mut self) -> Self {
        self.style.align_items = AlignItems::Center;
        self
    }

    /// Align items to end.
    pub fn align_end(mut self) -> Self {
        self.style.align_items = AlignItems::FlexEnd;
        self
    }

    /// Justify content to start.
    pub fn justify_start(mut self) -> Self {
        self.style.justify_content = JustifyContent::FlexStart;
        self
    }

    /// Justify content to center.
    pub fn justify_center(mut self) -> Self {
        self.style.justify_content = JustifyContent::Center;
        self
    }

    /// Justify content to end.
    pub fn justify_end(mut self) -> Self {
        self.style.justify_content = JustifyContent::FlexEnd;
        self
    }

    /// Justify content with space between.
    pub fn justify_between(mut self) -> Self {
        self.style.justify_content = JustifyContent::SpaceBetween;
        self
    }

    /// Set foreground color.
    pub fn fg(mut self, color: Color) -> Self {
        self.appearance.fg = Some(color);
        self
    }

    /// Set background color.
    pub fn bg(mut self, color: Color) -> Self {
        self.appearance.bg = Some(color);
        self
    }

    /// Add single line border.
    pub fn border(mut self) -> Self {
        self.appearance.border = Some(BorderStyle::Single);
        self
    }

    /// Add rounded border.
    pub fn border_rounded(mut self) -> Self {
        self.appearance.border = Some(BorderStyle::Rounded);
        self
    }

    /// Add double line border.
    pub fn border_double(mut self) -> Self {
        self.appearance.border = Some(BorderStyle::Double);
        self
    }

    /// Add heavy border.
    pub fn border_heavy(mut self) -> Self {
        self.appearance.border = Some(BorderStyle::Heavy);
        self
    }

    /// Build into a RenderNode.
    pub fn build(self, id: u64) -> RenderNode {
        RenderNode::new(id, NodeKind::Box)
            .with_style(self.style)
            .with_appearance(self.appearance)
    }
}

#[cfg(test)]
mod tests {
    use super::BoxNode;
    use crate::layout::{Dimension, FlexDirection};
    use crate::render::{BorderStyle, NodeKind};

    #[test]
    fn test_box_node_new() {
        let node = BoxNode::new().build(1);
        assert!(matches!(node.kind, NodeKind::Box));
        assert_eq!(node.id, 1);
    }

    #[test]
    fn test_box_node_column() {
        let builder = BoxNode::new().column();
        assert_eq!(builder.style.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn test_box_node_border() {
        let builder = BoxNode::new().border();
        assert_eq!(builder.appearance.border, Some(BorderStyle::Single));
    }

    #[test]
    fn test_box_node_dimensions() {
        let builder = BoxNode::new().width(100.0).height(50.0);
        assert!(matches!(builder.style.width, Dimension::Points(100.0)));
        assert!(matches!(builder.style.height, Dimension::Points(50.0)));
    }
}
