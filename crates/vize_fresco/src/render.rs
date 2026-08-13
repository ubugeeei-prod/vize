//! Rendering module for TUI output.
//!
//! Provides efficient differential rendering:
//! - Render tree management
//! - Node definitions
//! - Diffing algorithm
//! - Paint operations

mod diff;
mod frame;
mod node;
mod painter;
mod tree;

#[cfg(test)]
mod frame_recovery_tests;
#[cfg(test)]
mod frame_test_support;
#[cfg(test)]
mod frame_tests;

pub use frame::{
    FRAME_TELEMETRY_SCHEMA_VERSION, FrameActivityTelemetry, FrameCoalescer, FrameRenderError,
    FrameRenderer, FrameRequestOutcome, FrameTelemetry,
};

pub use node::{
    Appearance, BorderStyle, InputContent, NodeId, NodeKind, RawContent, RenderNode, TextContent,
};
pub use painter::Painter;
pub use tree::RenderTree;
