use std::io::{self, Write};

use crossterm::{
    cursor::{MoveTo, SetCursorStyle, Show},
    queue,
    style::{Attribute, SetAttribute, SetBackgroundColor, SetForegroundColor},
};

use super::{Backend, TerminalMode, TerminalOptions, TerminalSessionPhase};
use crate::terminal::{Color, CursorShape, Style};

#[test]
fn standard_output_backend_uses_detected_terminal_size_when_available() {
    if let Ok(backend) = Backend::new() {
        assert!(backend.width() > 0);
        assert!(backend.height() > 0);
    }
}

#[test]
fn terminal_options_documented_defaults_preserve_legacy_modes() {
    let options = TerminalOptions::default();

    assert!(options.alternate_screen);
    assert!(!options.mouse_capture);
    assert!(options.bracketed_paste);
    assert!(options.raw_mode);
    assert!(options.hide_cursor);
}

#[test]
fn session_state_is_a_single_byte_hot_path_snapshot() {
    assert_eq!(std::mem::size_of::<super::TerminalSessionState>(), 1);
}

#[test]
fn injected_writer_owns_lifecycle_output_and_restoration_is_idempotent() {
    let mut backend = Backend::with_writer(80, 24, Vec::new());
    backend
        .init_with_options(TerminalOptions {
            raw_mode: false,
            alternate_screen: true,
            mouse_capture: true,
            bracketed_paste: true,
            hide_cursor: true,
        })
        .unwrap();
    let initialized_bytes = backend.writer().len();
    assert!(initialized_bytes > 0);

    backend.restore().unwrap();
    let restored_bytes = backend.writer().len();
    assert!(restored_bytes > initialized_bytes);
    backend.restore().unwrap();
    assert_eq!(backend.writer().len(), restored_bytes);
}

#[test]
fn frame_cursor_output_is_owned_and_restored_as_one_session() {
    let mut backend = Backend::with_writer(4, 1, Vec::new());
    backend.cursor_mut().set_shape(CursorShape::Bar);
    backend.cursor_mut().set_blinking(false);

    backend.flush_measured().unwrap();

    let active = backend.session_state();
    assert_eq!(active.phase(), TerminalSessionPhase::Active);
    assert!(active.owns(TerminalMode::CursorVisibility));
    assert!(active.owns(TerminalMode::CursorShape));

    let frame_end = backend.writer().len();
    backend.restore().unwrap();
    assert!(backend.session_state().is_inactive());

    let restoration = &backend.writer()[frame_end..];
    let mut expected = Vec::new();
    queue!(expected, SetCursorStyle::DefaultUserShape, Show).unwrap();
    assert_eq!(restoration, expected);
}

#[test]
fn hidden_cursor_frame_does_not_claim_cursor_shape() {
    let mut backend = Backend::with_writer(4, 1, Vec::new());
    backend.cursor_mut().hide();

    backend.flush_measured().unwrap();

    let active = backend.session_state();
    assert!(active.owns(TerminalMode::CursorVisibility));
    assert!(!active.owns(TerminalMode::CursorShape));
    let frame_end = backend.writer().len();

    backend.restore().unwrap();
    let restoration = &backend.writer()[frame_end..];
    let mut expected = Vec::new();
    queue!(expected, Show).unwrap();
    assert_eq!(restoration, expected);
}

#[test]
fn failed_frame_conservatively_retains_cursor_ownership() {
    let mut backend = Backend::with_writer(4, 1, AlwaysFailWriter);

    assert!(backend.flush_measured().is_err());

    let active = backend.session_state();
    assert!(active.owns(TerminalMode::CursorVisibility));
    assert!(active.owns(TerminalMode::CursorShape));
}

#[test]
fn measured_flush_reports_exact_writer_bytes_and_changed_cells() {
    let mut backend = Backend::with_writer(4, 1, Vec::new());
    backend.buffer_mut().set_string(0, 0, "A", Style::new());

    let first = backend.flush_measured().unwrap();
    assert_eq!(first.changed_cells(), 1);
    assert_eq!(first.bytes_written(), backend.writer().len() as u64);
    assert!(backend.writer().starts_with(&style_reset_bytes()));

    let first_total = backend.writer().len();
    backend.buffer_mut().set_string(0, 0, "A", Style::new());
    let unchanged = backend.flush_measured().unwrap();
    assert_eq!(unchanged.changed_cells(), 0);
    assert_eq!(
        unchanged.bytes_written(),
        (backend.writer().len() - first_total) as u64
    );
}

#[test]
fn wide_glyph_telemetry_counts_the_continuation_cell_without_printing_it() {
    let mut backend = Backend::with_writer(2, 1, Vec::new());
    backend.buffer_mut().set_string(0, 0, "界", Style::new());

    let telemetry = backend.flush_measured().unwrap();

    assert_eq!(telemetry.changed_cells(), 2);
    assert_eq!(
        backend
            .writer()
            .windows("界".len())
            .filter(|window| *window == "界".as_bytes())
            .count(),
        1
    );
}

#[test]
fn failed_flush_retains_the_complete_current_frame_for_retry() {
    let mut backend = Backend::with_writer(2, 1, AlwaysFailWriter);
    backend.buffer_mut().set_string(0, 0, "A", Style::new());

    assert!(backend.flush_measured().is_err());
    assert_eq!(
        backend.buffer().get(0, 0).map(|cell| cell.symbol.as_str()),
        Some("A")
    );
}

#[test]
fn failed_clear_does_not_discard_buffer_state() {
    let mut backend = Backend::with_writer(2, 1, AlwaysFailWriter);
    backend.buffer_mut().set_string(0, 0, "A", Style::new());

    assert!(backend.clear().is_err());
    assert_eq!(
        backend.buffer().get(0, 0).map(|cell| cell.symbol.as_str()),
        Some("A")
    );
}

#[test]
fn restore_completes_every_cleanup_action_after_a_writer_failure() {
    let mut backend = Backend::with_writer(4, 1, ArmedFailureWriter::default());
    backend
        .init_with_options(TerminalOptions {
            raw_mode: false,
            alternate_screen: true,
            mouse_capture: true,
            bracketed_paste: true,
            hide_cursor: true,
        })
        .unwrap();
    backend.writer_mut().data.clear();
    backend.writer_mut().fail_next_write = true;

    assert!(backend.restore().is_err());
    // The armed failure hits the first writer-borne release, which stays owned
    // for the retry while every later cleanup still completes. On Windows,
    // crossterm releases mouse capture through the console API instead of the
    // writer, so the first writer-borne release is bracketed paste there.
    #[cfg(windows)]
    let retained = TerminalMode::BracketedPaste;
    #[cfg(not(windows))]
    let retained = TerminalMode::MouseCapture;
    for mode in [
        TerminalMode::MouseCapture,
        TerminalMode::BracketedPaste,
        TerminalMode::AlternateScreen,
        TerminalMode::CursorVisibility,
    ] {
        assert_eq!(backend.session_state().owns(mode), mode == retained);
    }

    let mut show_cursor = Vec::new();
    queue!(show_cursor, Show).unwrap();
    assert!(
        backend
            .writer()
            .data
            .windows(show_cursor.len())
            .any(|window| window == show_cursor.as_slice())
    );

    backend.restore().unwrap();
    assert_eq!(
        backend.session_state().phase(),
        TerminalSessionPhase::Inactive
    );
}

#[test]
fn first_frame_resets_style_before_a_default_style_glyph() {
    let mut backend = Backend::with_writer(2, 1, Vec::new());
    backend.buffer_mut().set_string(0, 0, "A", Style::new());

    backend.flush_measured().unwrap();

    assert!(backend.writer().starts_with(&style_reset_bytes()));
}

#[test]
fn retry_after_a_partially_written_frame_reestablishes_the_style_baseline() {
    // Budget: the opening style reset, the cursor move, and the foreground
    // color are accepted, then the glyph write fails with red still applied.
    let mut backend = Backend::with_writer(
        2,
        1,
        ByteBudgetWriter {
            remaining_bytes: usize::MAX,
            data: Vec::new(),
        },
    );
    backend.buffer_mut().set_string(0, 0, "A", Style::new());
    backend.flush_measured().unwrap();
    backend.writer_mut().data.clear();
    let mut colored_prefix = Vec::new();
    queue!(
        colored_prefix,
        MoveTo(0, 0),
        SetForegroundColor(crossterm::style::Color::DarkRed)
    )
    .unwrap();
    backend.writer_mut().remaining_bytes = colored_prefix.len();
    backend
        .buffer_mut()
        .set_string(0, 0, "A", Style::new().fg(Color::Red));
    assert!(backend.flush_measured().is_err());

    assert_eq!(backend.writer().data, colored_prefix);

    backend.writer_mut().remaining_bytes = usize::MAX;
    backend.writer_mut().data.clear();
    backend.buffer_mut().set_string(0, 0, "B", Style::new());
    backend.flush_measured().unwrap();

    assert!(backend.writer().data.starts_with(&style_reset_bytes()));
}

fn style_reset_bytes() -> Vec<u8> {
    let mut reset = Vec::new();
    queue!(
        reset,
        SetForegroundColor(crossterm::style::Color::Reset),
        SetBackgroundColor(crossterm::style::Color::Reset),
        SetAttribute(Attribute::Reset)
    )
    .unwrap();
    reset
}

#[derive(Debug)]
struct AlwaysFailWriter;

impl Write for AlwaysFailWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("injected writer failure"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected writer failure"))
    }
}

/// Writer that rejects exactly one armed write and then accepts output.
#[derive(Debug, Default)]
struct ArmedFailureWriter {
    fail_next_write: bool,
    data: Vec<u8>,
}

impl Write for ArmedFailureWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.fail_next_write {
            self.fail_next_write = false;
            return Err(io::Error::other("injected writer failure"));
        }
        self.data.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Writer that accepts a byte budget, including partial writes, before failing.
#[derive(Debug)]
struct ByteBudgetWriter {
    remaining_bytes: usize,
    data: Vec<u8>,
}

impl Write for ByteBudgetWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.remaining_bytes == 0 {
            return Err(io::Error::other("injected writer failure"));
        }
        let accepted = buffer.len().min(self.remaining_bytes);
        self.remaining_bytes -= accepted;
        self.data.extend_from_slice(&buffer[..accepted]);
        Ok(accepted)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.remaining_bytes == 0 {
            return Err(io::Error::other("injected writer failure"));
        }
        Ok(())
    }
}
