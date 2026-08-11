//! Transactional terminal-mode initialization and restoration.

use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

use crossterm::{
    cursor::{Hide, Show},
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use super::{Backend, TerminalOptions};

/// Terminal mode associated with one restoration failure.
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
        }
    }
}

impl fmt::Display for TerminalMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One terminal mode that could not be restored.
#[derive(Debug)]
pub struct TerminalCleanupFailure {
    mode: TerminalMode,
    error: io::Error,
}

impl TerminalCleanupFailure {
    /// Return the mode whose cleanup failed.
    pub const fn mode(&self) -> TerminalMode {
        self.mode
    }

    /// Return the underlying terminal or writer error.
    pub const fn error(&self) -> &io::Error {
        &self.error
    }
}

impl fmt::Display for TerminalCleanupFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.mode, self.error)
    }
}

impl Error for TerminalCleanupFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

/// Complete result of a best-effort terminal restoration.
///
/// Fresco always attempts every independently owned mode. This error retains
/// every failed cleanup in deterministic restoration order, while the modes
/// that restored successfully are removed from backend ownership.
#[derive(Debug)]
pub struct TerminalRestorationError {
    failures: Vec<TerminalCleanupFailure>,
}

impl TerminalRestorationError {
    /// Return every failed cleanup in the order Fresco attempted it.
    pub fn failures(&self) -> &[TerminalCleanupFailure] {
        &self.failures
    }
}

impl fmt::Display for TerminalRestorationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("terminal restoration failed for ")?;
        for (index, failure) in self.failures.iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "{failure}")?;
        }
        Ok(())
    }
}

impl Error for TerminalRestorationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.failures
            .first()
            .map(|failure| &failure.error as &(dyn Error + 'static))
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TerminalModeState {
    alternate_screen: bool,
    cursor_hidden: bool,
    raw_mode: bool,
    mouse_capture: bool,
    bracketed_paste: bool,
}

impl<W: Write> Backend<W> {
    /// Initialize the terminal using [`TerminalOptions::default`].
    pub fn init(&mut self) -> io::Result<()> {
        self.init_with_options(TerminalOptions::default())
    }

    /// Initialize the terminal using explicit mode options.
    ///
    /// Initialization is transactional relative to modes that were already
    /// active. If any enable operation fails, every mode attempted by this call
    /// is restored while pre-existing modes remain active. Escape-sequence
    /// modes are conservatively marked active before writing because an I/O
    /// error may occur after a terminal accepted a partial command.
    pub fn init_with_options(&mut self, options: TerminalOptions) -> io::Result<()> {
        let previous = self.mode_state();
        if let Err(initialization) = self.enable_modes(options) {
            return match self.restore_to(previous) {
                Ok(()) => Err(initialization),
                Err(rollback) => Err(combine_errors(initialization, rollback)),
            };
        }
        Ok(())
    }

    /// Initialize with mouse capture enabled.
    pub fn init_with_mouse(&mut self) -> io::Result<()> {
        self.init_with_options(TerminalOptions {
            mouse_capture: true,
            ..TerminalOptions::default()
        })
    }

    /// Restore every terminal mode enabled or possibly enabled by this backend.
    ///
    /// Every independent cleanup action is attempted even when an earlier one
    /// fails, so a rejected escape sequence cannot prevent raw-mode cleanup.
    /// The operation is idempotent: a failed action remains marked active for a
    /// later explicit call or [`Drop`] retry. Every failure is retained in a
    /// [`TerminalRestorationError`] after all actions have been attempted.
    pub fn restore(&mut self) -> io::Result<()> {
        self.restore_to(TerminalModeState::default())
    }

    fn enable_modes(&mut self, options: TerminalOptions) -> io::Result<()> {
        if options.raw_mode && !self.raw_mode {
            enable_raw_mode()?;
            self.raw_mode = true;
        }
        if options.alternate_screen && !self.alternate_screen {
            self.alternate_screen = true;
            execute!(&mut self.writer, EnterAlternateScreen)?;
        }
        if options.bracketed_paste && !self.bracketed_paste {
            self.bracketed_paste = true;
            execute!(&mut self.writer, EnableBracketedPaste)?;
        }
        if options.mouse_capture && !self.mouse_capture {
            self.mouse_capture = true;
            execute!(&mut self.writer, EnableMouseCapture)?;
        }
        if options.hide_cursor && !self.cursor_hidden {
            self.cursor_hidden = true;
            execute!(&mut self.writer, Hide)?;
        }
        Ok(())
    }

    fn restore_to(&mut self, target: TerminalModeState) -> io::Result<()> {
        let mut failures = Vec::new();
        restore_writer_mode(
            &mut self.writer,
            &mut self.mouse_capture,
            target.mouse_capture,
            TerminalMode::MouseCapture,
            DisableMouseCapture,
            &mut failures,
        );
        restore_writer_mode(
            &mut self.writer,
            &mut self.bracketed_paste,
            target.bracketed_paste,
            TerminalMode::BracketedPaste,
            DisableBracketedPaste,
            &mut failures,
        );
        restore_writer_mode(
            &mut self.writer,
            &mut self.alternate_screen,
            target.alternate_screen,
            TerminalMode::AlternateScreen,
            LeaveAlternateScreen,
            &mut failures,
        );
        restore_writer_mode(
            &mut self.writer,
            &mut self.cursor_hidden,
            target.cursor_hidden,
            TerminalMode::CursorVisibility,
            Show,
            &mut failures,
        );
        if self.raw_mode && !target.raw_mode {
            match disable_raw_mode() {
                Ok(()) => self.raw_mode = false,
                Err(error) => failures.push(TerminalCleanupFailure {
                    mode: TerminalMode::RawMode,
                    error,
                }),
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            let kind = failures[0].error.kind();
            Err(io::Error::new(kind, TerminalRestorationError { failures }))
        }
    }

    const fn mode_state(&self) -> TerminalModeState {
        TerminalModeState {
            alternate_screen: self.alternate_screen,
            cursor_hidden: self.cursor_hidden,
            raw_mode: self.raw_mode,
            mouse_capture: self.mouse_capture,
            bracketed_paste: self.bracketed_paste,
        }
    }
}

fn restore_writer_mode<W: Write, C: crossterm::Command>(
    writer: &mut W,
    active: &mut bool,
    target: bool,
    mode: TerminalMode,
    command: C,
    failures: &mut Vec<TerminalCleanupFailure>,
) {
    if !*active || target {
        return;
    }
    match execute!(writer, command) {
        Ok(()) => *active = false,
        Err(error) => failures.push(TerminalCleanupFailure { mode, error }),
    }
}

fn combine_errors(initialization: io::Error, rollback: io::Error) -> io::Error {
    let kind = initialization.kind();
    io::Error::new(
        kind,
        TerminalInitializationFailure {
            initialization,
            rollback,
        },
    )
}

#[derive(Debug)]
struct TerminalInitializationFailure {
    initialization: io::Error,
    rollback: io::Error,
}

impl fmt::Display for TerminalInitializationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "terminal initialization failed: {}; rollback also failed: {}",
            self.initialization, self.rollback
        )
    }
}

impl Error for TerminalInitializationFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.initialization)
    }
}
