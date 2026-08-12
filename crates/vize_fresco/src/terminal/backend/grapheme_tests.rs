use super::Backend;
use crate::terminal::Style;

#[test]
fn measured_output_writes_complete_graphemes_and_exact_byte_counts() {
    let text = "e\u{301}界👨‍👩‍👧‍👦";
    let mut backend = Backend::with_writer(5, 1, Vec::new());
    assert_eq!(backend.buffer_mut().set_string(0, 0, text, Style::new()), 5);

    let telemetry = backend.flush_measured().unwrap();
    let bytes = backend.writer();

    assert_eq!(telemetry.changed_cells(), 5);
    assert_eq!(telemetry.bytes_written(), bytes.len() as u64);
    for grapheme in ["e\u{301}", "界", "👨‍👩‍👧‍👦"] {
        assert_eq!(
            bytes
                .windows(grapheme.len())
                .filter(|window| *window == grapheme.as_bytes())
                .count(),
            1
        );
    }
}

#[test]
fn clipped_graphemes_emit_no_partial_utf8_bytes() {
    let mut backend = Backend::with_writer(1, 1, Vec::new());
    assert_eq!(backend.buffer_mut().set_string(0, 0, "界", Style::new()), 0);

    backend.flush_measured().unwrap();

    assert!(
        !backend
            .writer()
            .windows("界".len())
            .any(|bytes| bytes == "界".as_bytes())
    );
}
