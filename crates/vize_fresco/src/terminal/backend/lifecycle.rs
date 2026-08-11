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
    /// later explicit call or [`Drop`] retry, and the first error is returned
    /// after all actions have been attempted.
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
        let mut first_error = None;
        restore_writer_mode(
            &mut self.writer,
            &mut self.mouse_capture,
            target.mouse_capture,
            DisableMouseCapture,
            &mut first_error,
        );
        restore_writer_mode(
            &mut self.writer,
            &mut self.bracketed_paste,
            target.bracketed_paste,
            DisableBracketedPaste,
            &mut first_error,
        );
        restore_writer_mode(
            &mut self.writer,
            &mut self.alternate_screen,
            target.alternate_screen,
            LeaveAlternateScreen,
            &mut first_error,
        );
        restore_writer_mode(
            &mut self.writer,
            &mut self.cursor_hidden,
            target.cursor_hidden,
            Show,
            &mut first_error,
        );
        if self.raw_mode && !target.raw_mode {
            match disable_raw_mode() {
                Ok(()) => self.raw_mode = false,
                Err(error) => first_error = first_error.or(Some(error)),
            }
        }
        first_error.map_or(Ok(()), Err)
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
    command: C,
    first_error: &mut Option<io::Error>,
) {
    if !*active || target {
        return;
    }
    match execute!(writer, command) {
        Ok(()) => *active = false,
        Err(error) => *first_error = first_error.take().or(Some(error)),
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
