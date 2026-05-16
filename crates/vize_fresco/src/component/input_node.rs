//! Input component - text input with IME support.

use compact_str::CompactString;

use crate::layout::{Dimension, FlexStyle};
use crate::render::{Appearance, InputContent, NodeKind, RenderNode};
use crate::terminal::Color;

/// Builder for Input nodes.
#[derive(Debug, Clone, Default)]
pub struct InputNode {
    value: CompactString,
    placeholder: CompactString,
    focused: bool,
    mask: bool,
    mask_char: char,
    style: FlexStyle,
    appearance: Appearance,
}

impl InputNode {
    /// Create a new input node builder.
    pub fn new() -> Self {
        Self {
            mask_char: '*',
            ..Default::default()
        }
    }

    /// Set the input value.
    pub fn value(mut self, value: impl Into<CompactString>) -> Self {
        self.value = value.into();
        self
    }

    /// Set placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<CompactString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Mark as focused.
    pub fn focused(mut self) -> Self {
        self.focused = true;
        self
    }

    /// Enable password masking.
    pub fn password(mut self) -> Self {
        self.mask = true;
        self
    }

    /// Set mask character.
    pub fn mask_char(mut self, ch: char) -> Self {
        self.mask = true;
        self.mask_char = ch;
        self
    }

    /// Set width.
    pub fn width(mut self, width: f32) -> Self {
        self.style.width = Dimension::Points(width);
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

    /// Set flex grow.
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
        self
    }

    /// Build into a RenderNode.
    pub fn build(self, id: u64) -> RenderNode {
        let content = InputContent {
            value: self.value,
            placeholder: self.placeholder,
            cursor: 0,
            focused: self.focused,
            mask: self.mask,
            mask_char: self.mask_char,
        };
        RenderNode::new(id, NodeKind::Input(content))
            .with_style(self.style)
            .with_appearance(self.appearance)
    }
}

#[cfg(test)]
mod tests {
    use super::InputNode;
    use crate::render::NodeKind;

    #[test]
    fn test_input_node_new() {
        let node = InputNode::new().build(1);
        assert_eq!(node.id, 1);
        if let NodeKind::Input(content) = &node.kind {
            assert!(content.value.is_empty());
            assert!(!content.focused);
        } else {
            panic!("Expected Input node");
        }
    }

    #[test]
    fn test_input_node_value() {
        let node = InputNode::new()
            .value("Hello")
            .placeholder("Type here...")
            .focused()
            .build(1);

        if let NodeKind::Input(content) = &node.kind {
            assert_eq!(content.value.as_str(), "Hello");
            assert_eq!(content.placeholder.as_str(), "Type here...");
            assert!(content.focused);
        }
    }

    #[test]
    fn test_input_node_password() {
        let node = InputNode::new().password().build(1);
        if let NodeKind::Input(content) = &node.kind {
            assert!(content.mask);
            assert_eq!(content.mask_char, '*');
        }
    }

    #[test]
    fn test_input_node_custom_mask() {
        let node = InputNode::new().mask_char('•').build(1);
        if let NodeKind::Input(content) = &node.kind {
            assert!(content.mask);
            assert_eq!(content.mask_char, '•');
        }
    }
}
