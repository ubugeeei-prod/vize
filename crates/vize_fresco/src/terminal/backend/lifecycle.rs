//! Transactional terminal-mode initialization and restoration.

use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

use crossterm::{
    cursor::{Hide, SetCursorStyle, Show},
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

use super::{Backend, TerminalOptions};

mod lease;
mod panic_hook;
mod raw_mode;
mod signal_hook;
mod state;

#[cfg(all(test, unix))]
mod pty_test_support;

use raw_mode::{disable_raw_mode, enable_raw_mode, raw_mode_requires_restoration};

pub use lease::TerminalSessionAcquireError;
pub use panic_hook::{
    TerminalPanicHookError, TerminalPanicHookInstallation, install_terminal_panic_hook,
};
pub use signal_hook::{
    TerminalSignalHookError, TerminalSignalHookInstallation, TerminalSignalRollbackFailure,
    install_terminal_signal_hook,
};
pub use state::{TerminalMode, TerminalSessionPhase, TerminalSessionState};

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

impl<W: Write> Backend<W> {
    /// Return a copy of this backend's terminal-session ownership state.
    ///
    /// A mode remains owned after uncertain partial output or failed cleanup,
    /// allowing callers to decide whether another restoration attempt is
    /// required without inspecting Fresco-private fields.
    pub const fn session_state(&self) -> TerminalSessionState {
        self.session
    }

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
    /// error may occur after a terminal accepted a partial command. On Unix,
    /// raw mode retains the exact prior termios value and a dedicated terminal
    /// descriptor for panic-safe restoration.
    pub fn init_with_options(&mut self, options: TerminalOptions) -> io::Result<()> {
        self.prepare_session(options)?;
        let previous = self.session;
        if let Err(initialization) = self.enable_modes(options) {
            let result = match self.restore_to(previous) {
                Ok(()) => Err(initialization),
                Err(rollback) => Err(combine_errors(initialization, rollback)),
            };
            self.release_process_lease_if_inactive();
            return result;
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
        let result = self.restore_to(TerminalSessionState::new());
        self.release_process_lease_if_inactive();
        result
    }

    fn enable_modes(&mut self, options: TerminalOptions) -> io::Result<()> {
        if options.raw_mode && !self.session.owns(TerminalMode::RawMode) {
            if let Err(error) = enable_raw_mode() {
                if raw_mode_requires_restoration() {
                    self.acquire_mode(TerminalMode::RawMode);
                }
                return Err(error);
            }
            self.acquire_mode(TerminalMode::RawMode);
        }
        if options.alternate_screen && !self.session.owns(TerminalMode::AlternateScreen) {
            self.acquire_mode(TerminalMode::AlternateScreen);
            execute!(&mut self.writer, EnterAlternateScreen)?;
        }
        if options.bracketed_paste && !self.session.owns(TerminalMode::BracketedPaste) {
            self.acquire_mode(TerminalMode::BracketedPaste);
            execute!(&mut self.writer, EnableBracketedPaste)?;
        }
        if options.mouse_capture && !self.session.owns(TerminalMode::MouseCapture) {
            self.acquire_mode(TerminalMode::MouseCapture);
            execute!(&mut self.writer, EnableMouseCapture)?;
        }
        if options.hide_cursor && !self.session.owns(TerminalMode::CursorVisibility) {
            self.acquire_mode(TerminalMode::CursorVisibility);
            execute!(&mut self.writer, Hide)?;
        }
        Ok(())
    }

    fn restore_to(&mut self, target: TerminalSessionState) -> io::Result<()> {
        let mut failures = Vec::new();
        restore_writer_mode(
            &mut self.writer,
            &mut self.session,
            target,
            TerminalMode::MouseCapture,
            DisableMouseCapture,
            &mut failures,
        );
        restore_writer_mode(
            &mut self.writer,
            &mut self.session,
            target,
            TerminalMode::BracketedPaste,
            DisableBracketedPaste,
            &mut failures,
        );
        restore_writer_mode(
            &mut self.writer,
            &mut self.session,
            target,
            TerminalMode::AlternateScreen,
            LeaveAlternateScreen,
            &mut failures,
        );
        restore_writer_mode(
            &mut self.writer,
            &mut self.session,
            target,
            TerminalMode::CursorShape,
            SetCursorStyle::DefaultUserShape,
            &mut failures,
        );
        restore_writer_mode(
            &mut self.writer,
            &mut self.session,
            target,
            TerminalMode::CursorVisibility,
            Show,
            &mut failures,
        );
        if self.session.owns(TerminalMode::RawMode) && !target.owns(TerminalMode::RawMode) {
            match disable_raw_mode() {
                Ok(()) => self.session.release(TerminalMode::RawMode),
                Err(error) => failures.push(TerminalCleanupFailure {
                    mode: TerminalMode::RawMode,
                    error,
                }),
            }
        }
        self.publish_process_session_state();
        if failures.is_empty() {
            Ok(())
        } else {
            let kind = failures[0].error.kind();
            Err(io::Error::new(kind, TerminalRestorationError { failures }))
        }
    }

    fn acquire_mode(&mut self, mode: TerminalMode) {
        self.session.acquire(mode);
        self.publish_process_session_state();
    }
}

fn restore_writer_mode<W: Write, C: crossterm::Command>(
    writer: &mut W,
    session: &mut TerminalSessionState,
    target: TerminalSessionState,
    mode: TerminalMode,
    command: C,
    failures: &mut Vec<TerminalCleanupFailure>,
) {
    if !session.owns(mode) || target.owns(mode) {
        return;
    }
    match execute!(writer, command) {
        Ok(()) => session.release(mode),
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
