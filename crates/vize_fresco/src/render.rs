//! Rendering module for TUI output.
//!
//! Provides efficient differential rendering:
//! - Render tree management
//! - Node definitions
//! - Diffing algorithm
//! - Paint operations

mod diff;
mod node;
mod painter;
mod tree;

pub use node::{
    Appearance, BorderStyle, InputContent, NodeId, NodeKind, RawContent, RenderNode, TextContent,
};
pub use painter::Painter;
pub use tree::RenderTree;
