//! Structured, multi-mode terminal restoration failures.

use std::io::{self, Write};

use vize_s0::ToCompactString;

use super::{Backend, TerminalMode, TerminalOptions, TerminalRestorationError};

#[test]
fn restoration_reports_every_failed_mode_in_cleanup_order() {
    let mut backend = Backend::with_writer(80, 24, SwitchableFailureWriter::default());
    backend
        .init_with_options(TerminalOptions {
            raw_mode: false,
            alternate_screen: true,
            bracketed_paste: true,
            mouse_capture: true,
            hide_cursor: true,
        })
        .unwrap();
    backend.writer_mut().fail = true;

    let error = backend.restore().unwrap_err();
    let restoration = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<TerminalRestorationError>())
        .expect("restoration error must preserve structured cleanup failures");
    assert_eq!(
        restoration
            .failures()
            .iter()
            .map(|failure| failure.mode())
            .collect::<Vec<_>>(),
        [
            TerminalMode::MouseCapture,
            TerminalMode::BracketedPaste,
            TerminalMode::AlternateScreen,
            TerminalMode::CursorVisibility,
        ]
    );
    assert!(
        restoration
            .failures()
            .iter()
            .all(|failure| failure.error().kind() == io::ErrorKind::Other)
    );
    let message = error.to_compact_string();
    for mode in [
        "mouse capture",
        "bracketed paste",
        "alternate screen",
        "cursor visibility",
    ] {
        assert!(message.contains(mode), "missing {mode} in {message}");
    }
    for mode in [
        TerminalMode::MouseCapture,
        TerminalMode::BracketedPaste,
        TerminalMode::AlternateScreen,
        TerminalMode::CursorVisibility,
    ] {
        assert!(backend.session_state().owns(mode));
    }

    backend.writer_mut().fail = false;
    backend.restore().unwrap();
}

#[test]
fn cursor_shape_and_visibility_failures_remain_independently_retryable() {
    let mut backend = Backend::with_writer(80, 24, SwitchableFailureWriter::default());
    backend.flush().unwrap();
    backend.writer_mut().fail = true;

    let error = backend.restore().unwrap_err();
    let restoration = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<TerminalRestorationError>())
        .expect("cursor cleanup must retain structured failures");
    assert_eq!(
        restoration
            .failures()
            .iter()
            .map(|failure| failure.mode())
            .collect::<Vec<_>>(),
        [TerminalMode::CursorShape, TerminalMode::CursorVisibility]
    );
    assert!(backend.session_state().owns(TerminalMode::CursorShape));
    assert!(backend.session_state().owns(TerminalMode::CursorVisibility));

    backend.writer_mut().fail = false;
    backend.restore().unwrap();
    assert!(backend.session_state().is_inactive());
}

#[derive(Debug, Default)]
struct SwitchableFailureWriter {
    fail: bool,
    data: Vec<u8>,
}

impl Write for SwitchableFailureWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.fail {
            return Err(io::Error::other("injected restoration failure"));
        }
        self.data.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.fail {
            return Err(io::Error::other("injected restoration failure"));
        }
        Ok(())
    }
}
