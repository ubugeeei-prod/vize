//! Allocation-free differential frame output and telemetry.

use std::io::{self, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{Attribute, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
};

use super::Backend;
use crate::terminal::Style;

/// Measured presentation cost of one successfully written frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrameOutputTelemetry {
    changed_cells: u64,
    bytes_written: u64,
}

impl FrameOutputTelemetry {
    /// Return cells whose content or style differed from the previous frame.
    ///
    /// Wide-character continuation cells are included even though they do not
    /// emit a separate glyph.
    pub const fn changed_cells(self) -> u64 {
        self.changed_cells
    }

    /// Return exact bytes accepted by the presentation writer.
    ///
    /// This includes cursor, style, glyph, and reset control sequences.
    pub const fn bytes_written(self) -> u64 {
        self.bytes_written
    }
}

impl<W: Write> Backend<W> {
    /// Render the current buffer through the owned presentation writer.
    ///
    /// This compatibility method discards frame telemetry. Use
    /// [`flush_measured`](Self::flush_measured) for performance gates.
    pub fn flush(&mut self) -> io::Result<()> {
        self.flush_measured().map(|_| ())
    }

    /// Render the current buffer and return exact output telemetry.
    ///
    /// The diff is streamed without collecting changed cells. Buffers swap only
    /// after every byte is accepted and flushed; an I/O failure leaves the
    /// current frame intact for a complete retry.
    pub fn flush_measured(&mut self) -> io::Result<FrameOutputTelemetry> {
        match self.write_frame() {
            Ok(telemetry) => {
                self.style_baseline_unknown = false;
                std::mem::swap(&mut self.current, &mut self.previous);
                self.current.clear();
                self.current_frame_blank = true;
                Ok(telemetry)
            }
            Err(error) => {
                // A partially written frame may have left an arbitrary style
                // applied, so the next frame must reestablish the baseline.
                self.style_baseline_unknown = true;
                self.current_frame_blank = false;
                Err(error)
            }
        }
    }

    fn write_frame(&mut self) -> io::Result<FrameOutputTelemetry> {
        self.acquire_process_lease()?;

        // Cursor commands are part of every frame. Acquire ownership before
        // writing so a partial command is restored conservatively after an
        // I/O failure. Publish only when ownership expands, keeping steady-state
        // frames free of atomic stores.
        if self.session.acquire_frame_cursor(self.cursor.visible) {
            self.publish_process_session_state();
        }

        let mut writer = CountingWriter::new(&mut self.writer);
        let mut changed_cells = 0_u64;
        let mut last_written: Option<(u16, u16)> = None;
        let mut last_style = Style::new();

        if self.style_baseline_unknown {
            queue_style_reset(&mut writer)?;
        }

        for (x, y, cell) in self.current.diff(&self.previous) {
            changed_cells = changed_cells.saturating_add(1);
            if cell.is_continuation {
                continue;
            }

            let adjacent = matches!(
                last_written,
                Some((last_x, last_y)) if y == last_y && x == last_x.saturating_add(1)
            );
            if !adjacent {
                queue!(writer, MoveTo(x, y))?;
            }
            if cell.style != last_style {
                apply_style(&mut writer, &cell.style, &last_style)?;
                last_style = cell.style;
            }
            queue!(writer, Print(&cell.symbol))?;
            last_written = Some((x, y));
        }

        queue_style_reset(&mut writer)?;
        if self.cursor.visible {
            let cursor_style = if self.cursor.blinking {
                self.cursor.shape.to_blinking_cursor_style()
            } else {
                self.cursor.shape.to_cursor_style()
            };
            queue!(
                writer,
                MoveTo(self.cursor.x, self.cursor.y),
                cursor_style,
                Show
            )?;
        } else {
            queue!(writer, Hide)?;
        }
        writer.flush()?;
        let bytes_written = writer.bytes_written;

        Ok(FrameOutputTelemetry {
            changed_cells,
            bytes_written,
        })
    }
}

fn queue_style_reset(writer: &mut impl Write) -> io::Result<()> {
    queue!(
        writer,
        SetForegroundColor(crossterm::style::Color::Reset),
        SetBackgroundColor(crossterm::style::Color::Reset),
        SetAttribute(Attribute::Reset)
    )
}

fn apply_style(writer: &mut impl Write, new: &Style, old: &Style) -> io::Result<()> {
    if new.fg != old.fg {
        queue!(
            writer,
            SetForegroundColor(new.fg.map_or(crossterm::style::Color::Reset, Into::into))
        )?;
    }
    if new.bg != old.bg {
        queue!(
            writer,
            SetBackgroundColor(new.bg.map_or(crossterm::style::Color::Reset, Into::into))
        )?;
    }
    if new.bold != old.bold || new.dim != old.dim {
        queue!(writer, SetAttribute(Attribute::NormalIntensity))?;
        if new.bold {
            queue!(writer, SetAttribute(Attribute::Bold))?;
        }
        if new.dim {
            queue!(writer, SetAttribute(Attribute::Dim))?;
        }
    }
    queue_attribute(
        writer,
        new.italic,
        old.italic,
        Attribute::Italic,
        Attribute::NoItalic,
    )?;
    queue_attribute(
        writer,
        new.underline,
        old.underline,
        Attribute::Underlined,
        Attribute::NoUnderline,
    )?;
    queue_attribute(
        writer,
        new.blink,
        old.blink,
        Attribute::SlowBlink,
        Attribute::NoBlink,
    )?;
    queue_attribute(
        writer,
        new.strikethrough,
        old.strikethrough,
        Attribute::CrossedOut,
        Attribute::NotCrossedOut,
    )?;
    queue_attribute(
        writer,
        new.reverse,
        old.reverse,
        Attribute::Reverse,
        Attribute::NoReverse,
    )?;
    queue_attribute(
        writer,
        new.hidden,
        old.hidden,
        Attribute::Hidden,
        Attribute::NoHidden,
    )
}

fn queue_attribute(
    writer: &mut impl Write,
    new: bool,
    old: bool,
    enabled: Attribute,
    disabled: Attribute,
) -> io::Result<()> {
    if new != old {
        queue!(writer, SetAttribute(if new { enabled } else { disabled }))?;
    }
    Ok(())
}

struct CountingWriter<'a, W> {
    inner: &'a mut W,
    bytes_written: u64,
}

impl<'a, W> CountingWriter<'a, W> {
    const fn new(inner: &'a mut W) -> Self {
        Self {
            inner,
            bytes_written: 0,
        }
    }
}

impl<W: Write> Write for CountingWriter<'_, W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buffer)?;
        self.bytes_written = self
            .bytes_written
            .saturating_add(written.try_into().unwrap_or(u64::MAX));
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_vectored(&mut self, buffers: &[io::IoSlice<'_>]) -> io::Result<usize> {
        let written = self.inner.write_vectored(buffers)?;
        self.bytes_written = self
            .bytes_written
            .saturating_add(written.try_into().unwrap_or(u64::MAX));
        Ok(written)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::{queue, style::SetAttribute};

    use super::*;

    #[test]
    fn intensity_transition_reapplies_the_surviving_attribute() {
        let old = Style::new().bold().dim();
        let new = Style::new().dim();
        let mut actual = Vec::new();
        apply_style(&mut actual, &new, &old).unwrap();

        let mut expected = Vec::new();
        queue!(
            expected,
            SetAttribute(Attribute::NormalIntensity),
            SetAttribute(Attribute::Dim)
        )
        .unwrap();
        assert_eq!(actual, expected);
    }
}
