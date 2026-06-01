//! Primitive component nodes.
//!
//! Provides the core building blocks for TUI:
//! - BoxNode: Container with flexbox layout
//! - TextNode: Text display
//! - InputNode: Text input with IME support

mod box_node;
mod input_node;
mod text_node;

pub use box_node::BoxNode;
pub use input_node::InputNode;
pub use text_node::TextNode;
