//! Terminal backend with an injectable presentation writer.

use std::io::{self, Write};

use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};

use super::{buffer::Buffer, cursor::Cursor};

mod lifecycle;
mod output;

#[cfg(test)]
mod grapheme_tests;
#[cfg(test)]
mod lease_tests;
#[cfg(test)]
mod lifecycle_failure_tests;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod tests;

pub use lifecycle::{
    TerminalCleanupFailure, TerminalMode, TerminalPanicHookError, TerminalPanicHookInstallation,
    TerminalRestorationError, TerminalSessionAcquireError, TerminalSessionPhase,
    TerminalSessionState, TerminalSignalHookError, TerminalSignalHookInstallation,
    TerminalSignalRollbackFailure, install_terminal_panic_hook, install_terminal_signal_hook,
};
pub use output::FrameOutputTelemetry;

/// Terminal mode switches used during backend initialization.
#[derive(Debug, Clone, Copy)]
pub struct TerminalOptions {
    /// Enable process-terminal raw input. Defaults to `true`.
    ///
    /// This mode is process-global. Fresco preserves an already-active
    /// Crossterm raw-mode owner instead of disabling it during restoration;
    /// other owners must not change raw mode while the Fresco lease is active.
    pub raw_mode: bool,
    /// Enter the alternate screen. Defaults to `true`.
    pub alternate_screen: bool,
    /// Capture mouse events. Defaults to `false`.
    pub mouse_capture: bool,
    /// Enable bracketed paste. Defaults to `true`.
    pub bracketed_paste: bool,
    /// Hide the terminal cursor. Defaults to `true`.
    pub hide_cursor: bool,
}

impl Default for TerminalOptions {
    fn default() -> Self {
        Self {
            raw_mode: true,
            alternate_screen: true,
            mouse_capture: false,
            bracketed_paste: true,
            hide_cursor: true,
        }
    }
}

impl TerminalOptions {
    const fn requests_terminal_control(self) -> bool {
        self.raw_mode
            || self.alternate_screen
            || self.mouse_capture
            || self.bracketed_paste
            || self.hide_cursor
    }
}

/// Double-buffered terminal renderer with an owned presentation writer.
///
/// [`Backend::new`] preserves the standard-output behavior used by existing
/// applications. [`Backend::with_writer`] accepts any [`Write`] sink and an
/// explicit viewport, enabling deterministic headless tests without replacing
/// process-global standard output. Terminal mode escape sequences, clear
/// operations, differential frames, and restoration all use the same writer.
pub struct Backend<W: Write = io::Stdout> {
    pub(super) current: Buffer,
    pub(super) previous: Buffer,
    pub(super) cursor: Cursor,
    /// Whether `current` is known to contain only default empty cells.
    ///
    /// Successful flushes and resizes establish this invariant. Mutable buffer
    /// access and failed output conservatively invalidate it, allowing retained
    /// tree rendering to recover without scanning the viewport on normal frames.
    current_frame_blank: bool,
    /// Single source of truth for terminal presentation owned by this backend.
    session: TerminalSessionState,
    /// Whether this backend writes to the process terminal rather than an
    /// isolated injected sink.
    process_terminal: bool,
    /// Whether this backend currently holds the process-terminal lease.
    process_lease: bool,
    /// Set while the terminal style may not match [`Style::new`], either
    /// because no frame has been written yet or because a frame failed
    /// mid-write, requiring an explicit reset before the next frame.
    ///
    /// [`Style::new`]: crate::terminal::Style::new
    style_baseline_unknown: bool,
    width: u16,
    height: u16,
    pub(super) writer: W,
}

impl Backend<io::Stdout> {
    /// Create a standard-output backend with the current terminal size.
    pub fn new() -> io::Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        let mut backend = Self::with_writer(width, height, io::stdout());
        backend.process_terminal = true;
        Ok(backend)
    }
}

impl<W: Write> Backend<W> {
    /// Create a backend with an explicit viewport and presentation writer.
    ///
    /// This constructor does not inspect or mutate terminal process state.
    /// Raw mode is rejected during initialization because it is process-global;
    /// escape-sequence modes remain isolated to the injected writer.
    pub fn with_writer(width: u16, height: u16, writer: W) -> Self {
        Self {
            current: Buffer::new(width, height),
            previous: Buffer::new(width, height),
            cursor: Cursor::new(),
            current_frame_blank: true,
            session: TerminalSessionState::new(),
            process_terminal: false,
            process_lease: false,
            // An injected writer can point at a terminal whose inherited style
            // is unknown. The first frame establishes the same baseline used
            // after a partial-write failure.
            style_baseline_unknown: true,
            width,
            height,
            writer,
        }
    }

    /// Return the terminal width in cells.
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Return the terminal height in cells.
    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Return the current frame buffer for modification.
    #[inline]
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.current_frame_blank = false;
        &mut self.current
    }

    /// Return the current frame buffer for reading.
    #[inline]
    pub fn buffer(&self) -> &Buffer {
        &self.current
    }

    /// Return the cursor state for modification.
    #[inline]
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    /// Return the cursor state for reading.
    #[inline]
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Return the owned presentation writer for inspection.
    pub const fn writer(&self) -> &W {
        &self.writer
    }

    /// Return the owned presentation writer for configuration.
    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Check the process terminal size and resize both frame buffers if needed.
    pub fn sync_size(&mut self) -> io::Result<bool> {
        let (width, height) = crossterm::terminal::size()?;
        Ok(self.resize(width, height))
    }

    /// Resize both frame buffers to an explicit viewport.
    ///
    /// Returns `false` without reallocating when the viewport is unchanged.
    pub fn resize(&mut self, width: u16, height: u16) -> bool {
        if width == self.width && height == self.height {
            return false;
        }
        self.width = width;
        self.height = height;
        self.current.resize(width, height);
        self.previous.resize(width, height);
        self.current_frame_blank = true;
        true
    }

    /// Clear both frame buffers and the presentation screen.
    pub fn clear(&mut self) -> io::Result<()> {
        self.acquire_process_lease()?;
        execute!(&mut self.writer, Clear(ClearType::All))?;
        self.current.clear();
        self.previous.clear();
        self.current_frame_blank = true;
        Ok(())
    }

    /// Establish a blank current buffer before painting a new retained tree.
    ///
    /// The current buffer is already blank after every successful frame, so the
    /// normal path is one state check. A failed output retains its painted frame
    /// for direct [`flush`](Self::flush) retries; [`FrameRenderer`] calls this
    /// method before repainting because its tree may have changed meanwhile.
    ///
    /// [`FrameRenderer`]: crate::render::FrameRenderer
    pub(crate) fn prepare_retained_frame(&mut self) {
        if !self.current_frame_blank {
            self.current.clear();
            self.current_frame_blank = true;
        }
    }

    #[cfg(test)]
    fn with_process_writer(width: u16, height: u16, writer: W) -> Self {
        let mut backend = Self::with_writer(width, height, writer);
        backend.process_terminal = true;
        backend
    }
}

impl Default for Backend<io::Stdout> {
    fn default() -> Self {
        Self::new().expect("Failed to create backend")
    }
}

impl<W: Write> Drop for Backend<W> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
