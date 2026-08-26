use std::io::{self, Write};

use vize_s0::ToCompactString;

use super::{Backend, TerminalMode, TerminalOptions};

#[test]
fn failed_escape_modes_remain_active_when_rollback_cannot_confirm_restoration() {
    assert_uncertain_mode_remains_active(
        TerminalOptions {
            alternate_screen: true,
            ..disabled_options()
        },
        |backend| backend.session_state().owns(TerminalMode::AlternateScreen),
    );
    assert_uncertain_mode_remains_active(
        TerminalOptions {
            bracketed_paste: true,
            ..disabled_options()
        },
        |backend| backend.session_state().owns(TerminalMode::BracketedPaste),
    );
    assert_uncertain_mode_remains_active(
        TerminalOptions {
            mouse_capture: true,
            ..disabled_options()
        },
        |backend| backend.session_state().owns(TerminalMode::MouseCapture),
    );
    assert_uncertain_mode_remains_active(
        TerminalOptions {
            hide_cursor: true,
            ..disabled_options()
        },
        |backend| backend.session_state().owns(TerminalMode::CursorVisibility),
    );
}

#[test]
fn one_shot_enable_failure_is_rolled_back_before_returning() {
    let mut backend = Backend::with_writer(80, 24, OneShotFailureWriter::default());
    let error = backend
        .init_with_options(TerminalOptions {
            alternate_screen: true,
            ..disabled_options()
        })
        .unwrap_err();

    assert!(
        error
            .to_compact_string()
            .contains("injected enable failure")
    );
    assert!(backend.session_state().is_inactive());
    assert!(!backend.writer().data.is_empty());
    backend.restore().unwrap();
}

#[test]
fn repeated_initialization_does_not_reenter_active_terminal_modes() {
    let options = TerminalOptions {
        alternate_screen: true,
        bracketed_paste: true,
        hide_cursor: true,
        ..disabled_options()
    };
    let mut backend = Backend::with_writer(80, 24, Vec::new());
    backend.init_with_options(options).unwrap();
    let initialized_bytes = backend.writer().len();

    backend.init_with_options(options).unwrap();
    assert_eq!(backend.writer().len(), initialized_bytes);
    backend.restore().unwrap();
}

#[test]
fn partial_command_failure_is_retryable_after_rollback_also_fails() {
    let mut backend = Backend::with_writer(
        80,
        24,
        ByteBudgetWriter {
            remaining: 1,
            data: Vec::new(),
        },
    );
    let error = backend
        .init_with_options(TerminalOptions {
            alternate_screen: true,
            ..disabled_options()
        })
        .unwrap_err();

    assert!(error.to_compact_string().contains("rollback also failed"));
    assert!(backend.session_state().owns(TerminalMode::AlternateScreen));
    assert_eq!(backend.writer().data.len(), 1);

    backend.writer_mut().remaining = usize::MAX;
    backend.restore().unwrap();
    assert!(backend.session_state().is_inactive());
}

#[test]
fn rollback_restores_only_modes_started_by_the_failing_call() {
    let mut backend = Backend::with_writer(80, 24, FailOnFlushWriter::default());
    backend
        .init_with_options(TerminalOptions {
            alternate_screen: true,
            ..disabled_options()
        })
        .unwrap();
    assert!(backend.session_state().owns(TerminalMode::AlternateScreen));

    backend.writer_mut().fail_on_flush = Some(2);
    assert!(
        backend
            .init_with_options(TerminalOptions {
                alternate_screen: true,
                bracketed_paste: true,
                ..disabled_options()
            })
            .is_err()
    );

    assert!(backend.session_state().owns(TerminalMode::AlternateScreen));
    assert!(!backend.session_state().owns(TerminalMode::BracketedPaste));
    backend.restore().unwrap();
}

#[test]
fn rollback_attempts_every_mode_started_before_a_later_failure() {
    let mut backend = Backend::with_writer(
        80,
        24,
        FailOnFlushWriter {
            fail_on_flush: Some(2),
            ..FailOnFlushWriter::default()
        },
    );

    assert!(
        backend
            .init_with_options(TerminalOptions {
                alternate_screen: true,
                bracketed_paste: true,
                ..disabled_options()
            })
            .is_err()
    );
    assert!(backend.session_state().is_inactive());
    assert_eq!(backend.writer().flushes, 4);
}

fn disabled_options() -> TerminalOptions {
    TerminalOptions {
        raw_mode: false,
        alternate_screen: false,
        mouse_capture: false,
        bracketed_paste: false,
        hide_cursor: false,
    }
}

fn assert_uncertain_mode_remains_active(
    options: TerminalOptions,
    active: impl FnOnce(&Backend<AlwaysFailWriter>) -> bool,
) {
    let mut backend = Backend::with_writer(80, 24, AlwaysFailWriter);
    let error = backend.init_with_options(options).unwrap_err();
    assert!(error.to_compact_string().contains("rollback also failed"));
    assert!(active(&backend));
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

#[derive(Debug, Default)]
struct OneShotFailureWriter {
    fail_next_write: bool,
    data: Vec<u8>,
}

impl Write for OneShotFailureWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if !self.fail_next_write {
            self.fail_next_write = true;
            return Err(io::Error::other("injected enable failure"));
        }
        self.data.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct ByteBudgetWriter {
    remaining: usize,
    data: Vec<u8>,
}

impl Write for ByteBudgetWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::other("injected writer failure"));
        }
        let accepted = buffer.len().min(self.remaining);
        self.remaining -= accepted;
        self.data.extend_from_slice(&buffer[..accepted]);
        Ok(accepted)
    }

    fn flush(&mut self) -> io::Result<()> {
        (self.remaining > 0)
            .then_some(())
            .ok_or_else(|| io::Error::other("injected writer failure"))
    }
}

#[derive(Debug, Default)]
struct FailOnFlushWriter {
    fail_on_flush: Option<usize>,
    flushes: usize,
    data: Vec<u8>,
}

impl Write for FailOnFlushWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.data.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        if self.fail_on_flush == Some(self.flushes) {
            self.fail_on_flush = None;
            return Err(io::Error::other("injected flush failure"));
        }
        Ok(())
    }
}
