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
mod lifecycle_tests;
#[cfg(test)]
mod tests;

pub use output::FrameOutputTelemetry;

/// Terminal mode switches used during backend initialization.
#[derive(Debug, Clone, Copy)]
pub struct TerminalOptions {
    /// Enable process terminal raw mode. Defaults to `true`.
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
    alternate_screen: bool,
    cursor_hidden: bool,
    raw_mode: bool,
    mouse_capture: bool,
    bracketed_paste: bool,
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
        Ok(Self::with_writer(width, height, io::stdout()))
    }
}

impl<W: Write> Backend<W> {
    /// Create a backend with an explicit viewport and presentation writer.
    ///
    /// This constructor does not inspect or mutate terminal process state.
    /// Set `raw_mode` to `false` when initializing a memory-backed test writer.
    pub fn with_writer(width: u16, height: u16, writer: W) -> Self {
        Self {
            current: Buffer::new(width, height),
            previous: Buffer::new(width, height),
            cursor: Cursor::new(),
            alternate_screen: false,
            cursor_hidden: false,
            raw_mode: false,
            mouse_capture: false,
            bracketed_paste: false,
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
        true
    }

    /// Clear both frame buffers and the presentation screen.
    pub fn clear(&mut self) -> io::Result<()> {
        execute!(&mut self.writer, Clear(ClearType::All))?;
        self.current.clear();
        self.previous.clear();
        Ok(())
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
