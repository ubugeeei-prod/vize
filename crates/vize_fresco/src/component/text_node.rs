//! Text component - text display.

use compact_str::CompactString;

use crate::layout::FlexStyle;
use crate::render::{Appearance, NodeKind, RenderNode, TextContent};
use crate::terminal::Color;

/// Builder for Text nodes.
#[derive(Debug, Clone, Default)]
pub struct TextNode {
    text: CompactString,
    wrap: bool,
    style: FlexStyle,
    appearance: Appearance,
}

impl TextNode {
    /// Create a new text node builder.
    pub fn new(text: impl Into<CompactString>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Enable text wrapping.
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
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

    /// Set bold style.
    pub fn bold(mut self) -> Self {
        self.appearance.bold = true;
        self
    }

    /// Set dim style.
    pub fn dim(mut self) -> Self {
        self.appearance.dim = true;
        self
    }

    /// Set italic style.
    pub fn italic(mut self) -> Self {
        self.appearance.italic = true;
        self
    }

    /// Set underline style.
    pub fn underline(mut self) -> Self {
        self.appearance.underline = true;
        self
    }

    /// Set strikethrough style.
    pub fn strikethrough(mut self) -> Self {
        self.appearance.strikethrough = true;
        self
    }

    /// Set flex grow.
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
        self
    }

    /// Build into a RenderNode.
    pub fn build(self, id: u64) -> RenderNode {
        let content = TextContent {
            text: self.text,
            wrap: self.wrap,
        };
        RenderNode::new(id, NodeKind::Text(content))
            .with_style(self.style)
            .with_appearance(self.appearance)
    }
}

/// Convenience constructors for common text styles.
impl TextNode {
    /// Create error text (red).
    pub fn error(text: impl Into<CompactString>) -> Self {
        Self::new(text).fg(Color::Red)
    }

    /// Create warning text (yellow).
    pub fn warning(text: impl Into<CompactString>) -> Self {
        Self::new(text).fg(Color::Yellow)
    }

    /// Create success text (green).
    pub fn success(text: impl Into<CompactString>) -> Self {
        Self::new(text).fg(Color::Green)
    }

    /// Create info text (blue).
    pub fn info(text: impl Into<CompactString>) -> Self {
        Self::new(text).fg(Color::Blue)
    }

    /// Create muted/dim text.
    pub fn muted(text: impl Into<CompactString>) -> Self {
        Self::new(text).dim()
    }
}

#[cfg(test)]
mod tests {
    use super::TextNode;
    use crate::render::NodeKind;
    use crate::terminal::Color;

    #[test]
    fn test_text_node_new() {
        let node = TextNode::new("Hello").build(1);
        assert_eq!(node.id, 1);
        if let NodeKind::Text(content) = &node.kind {
            assert_eq!(content.text.as_str(), "Hello");
            assert!(!content.wrap);
        } else {
            panic!("Expected Text node");
        }
    }

    #[test]
    fn test_text_node_wrap() {
        let node = TextNode::new("Hello").wrap().build(1);
        if let NodeKind::Text(content) = &node.kind {
            assert!(content.wrap);
        }
    }

    #[test]
    fn test_text_node_styles() {
        let builder = TextNode::new("Test").fg(Color::Red).bold().underline();

        assert_eq!(builder.appearance.fg, Some(Color::Red));
        assert!(builder.appearance.bold);
        assert!(builder.appearance.underline);
    }

    #[test]
    fn test_text_node_presets() {
        let error = TextNode::error("Error message");
        assert_eq!(error.appearance.fg, Some(Color::Red));

        let success = TextNode::success("Success message");
        assert_eq!(success.appearance.fg, Some(Color::Green));
    }
}
