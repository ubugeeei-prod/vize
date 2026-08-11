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
    TerminalPanicHookError, TerminalPanicHookInstallation, TerminalRestorationError,
    TerminalSessionAcquireError, TerminalSessionPhase, TerminalSessionState,
    TerminalSignalHookError, TerminalSignalHookInstallation, TerminalSignalRollbackFailure,
    install_terminal_panic_hook, install_terminal_signal_hook,
};
pub use buffer::Buffer;
pub use capabilities::{
    CapabilityDecision, CapabilityReason, ColorPreference, ColorSupport, FeaturePreference,
    TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions,
};
pub use cell::{Cell, Color, Style};
pub use cursor::{Cursor, CursorShape};
