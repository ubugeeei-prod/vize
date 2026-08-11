use std::io::{self, Write};

use super::{Backend, TerminalOptions};
use crate::terminal::Style;

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
fn measured_flush_reports_exact_writer_bytes_and_changed_cells() {
    let mut backend = Backend::with_writer(4, 1, Vec::new());
    backend.buffer_mut().set_string(0, 0, "A", Style::new());

    let first = backend.flush_measured().unwrap();
    assert_eq!(first.changed_cells(), 1);
    assert_eq!(first.bytes_written(), backend.writer().len() as u64);

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
