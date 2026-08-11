//! Terminal control module using crossterm.
//!
//! Provides cross-platform terminal manipulation including:
//! - Raw mode initialization/cleanup
//! - Double-buffered rendering
//! - Cursor management
//! - Cell-based character storage with styles

mod backend;
mod buffer;
mod capabilities;
mod cell;
mod cursor;

pub use backend::{
    Backend, FrameOutputTelemetry, TerminalCleanupFailure, TerminalMode, TerminalOptions,
    TerminalRestorationError, TerminalSessionAcquireError, TerminalSessionPhase,
    TerminalSessionState,
};
pub use buffer::Buffer;
pub use capabilities::{
    CapabilityDecision, CapabilityReason, ColorPreference, ColorSupport, FeaturePreference,
    TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions,
};
pub use cell::{Cell, Color, Style};
pub use cursor::{Cursor, CursorShape};
