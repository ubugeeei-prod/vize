//! Compact ownership state for one terminal session.

use std::fmt;

/// Terminal mode owned by a Fresco backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminalMode {
    /// Process-global raw input mode.
    RawMode,
    /// Alternate screen buffer.
    AlternateScreen,
    /// Bracketed paste reporting.
    BracketedPaste,
    /// Mouse event capture.
    MouseCapture,
    /// Cursor visibility changed by Fresco.
    CursorVisibility,
    /// Cursor shape changed by Fresco frame output.
    CursorShape,
}

impl TerminalMode {
    /// Return the stable human-readable mode name used in errors.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawMode => "raw mode",
            Self::AlternateScreen => "alternate screen",
            Self::BracketedPaste => "bracketed paste",
            Self::MouseCapture => "mouse capture",
            Self::CursorVisibility => "cursor visibility",
            Self::CursorShape => "cursor shape",
        }
    }

    pub(in crate::terminal::backend) const fn bit(self) -> u8 {
        match self {
            Self::RawMode => 1 << 0,
            Self::AlternateScreen => 1 << 1,
            Self::BracketedPaste => 1 << 2,
            Self::MouseCapture => 1 << 3,
            Self::CursorVisibility => 1 << 4,
            Self::CursorShape => 1 << 5,
        }
    }
}

impl fmt::Display for TerminalMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Observable phase of one backend's terminal session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminalSessionPhase {
    /// The backend owns no terminal presentation state.
    Inactive,
    /// The backend owns, or may own, at least one terminal mode.
    Active,
}

/// Immutable snapshot of terminal modes owned by one Fresco backend.
///
/// Ownership is conservative. When a writer accepts only part of an escape
/// sequence, the corresponding mode remains owned until a later restoration
/// succeeds. This prevents partial I/O failures from being mistaken for a
/// clean terminal. The snapshot contains no writer or process-global state and
/// is therefore safe to inspect in tests and lifecycle supervisors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TerminalSessionState {
    owned_modes: u8,
}

impl TerminalSessionState {
    /// Return an inactive session state.
    pub const fn new() -> Self {
        Self { owned_modes: 0 }
    }

    /// Return whether the session owns, or may own, `mode`.
    pub const fn owns(self, mode: TerminalMode) -> bool {
        self.owned_modes & mode.bit() != 0
    }

    /// Return the lifecycle phase derived from current ownership.
    pub const fn phase(self) -> TerminalSessionPhase {
        if self.owned_modes == 0 {
            TerminalSessionPhase::Inactive
        } else {
            TerminalSessionPhase::Active
        }
    }

    /// Return whether the session owns no terminal modes.
    pub const fn is_inactive(self) -> bool {
        matches!(self.phase(), TerminalSessionPhase::Inactive)
    }

    /// Return whether the session owns at least one terminal mode.
    pub const fn is_active(self) -> bool {
        matches!(self.phase(), TerminalSessionPhase::Active)
    }

    pub(in crate::terminal::backend) const fn bits(self) -> u8 {
        self.owned_modes
    }

    #[inline]
    pub(in crate::terminal::backend) fn acquire(&mut self, mode: TerminalMode) {
        self.owned_modes |= mode.bit();
    }

    /// Conservatively own cursor commands queued by one frame.
    #[inline]
    pub(in crate::terminal::backend) fn acquire_frame_cursor(&mut self, visible: bool) -> bool {
        let previous = self.owned_modes;
        self.owned_modes |= TerminalMode::CursorVisibility.bit()
            | ((visible as u8) * TerminalMode::CursorShape.bit());
        self.owned_modes != previous
    }

    pub(super) fn release(&mut self, mode: TerminalMode) {
        self.owned_modes &= !mode.bit();
    }
}
