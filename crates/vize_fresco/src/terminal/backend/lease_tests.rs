//! Process-terminal lease and injected-writer isolation.

use std::{
    io::{self, Write},
    sync::{Mutex, MutexGuard},
};

use super::{Backend, TerminalOptions, TerminalSessionAcquireError};

static PROCESS_LEASE_TEST: Mutex<()> = Mutex::new(());

#[test]
fn process_backends_reject_overlapping_sessions_but_allow_reentry() {
    let _serial = lease_test_guard();
    let mut first = Backend::with_process_writer(80, 24, Vec::new());
    let mut second = Backend::with_process_writer(80, 24, Vec::new());

    first.init_with_options(writer_options()).unwrap();
    first.init_with_options(writer_options()).unwrap();
    assert!(first.holds_process_terminal_lease());

    let error = second
        .init_with_options(writer_options())
        .expect_err("a second process backend must not share terminal ownership");
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(
        acquire_reason(&error),
        TerminalSessionAcquireError::ProcessTerminalAlreadyOwned
    );
    assert!(!second.holds_process_terminal_lease());
    assert!(second.writer().is_empty());

    first.restore().unwrap();
    second.init_with_options(writer_options()).unwrap();
    second.restore().unwrap();
}

#[test]
fn injected_writers_are_independent_and_reject_process_raw_mode() {
    let _serial = lease_test_guard();
    let mut first = Backend::with_writer(80, 24, Vec::new());
    let mut second = Backend::with_writer(80, 24, Vec::new());

    first.init_with_options(writer_options()).unwrap();
    second.init_with_options(writer_options()).unwrap();
    assert!(!first.holds_process_terminal_lease());
    assert!(!second.holds_process_terminal_lease());
    first.restore().unwrap();
    second.restore().unwrap();

    let mut injected = Backend::with_writer(80, 24, Vec::new());
    let error = injected.init().unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(
        acquire_reason(&error),
        TerminalSessionAcquireError::RawModeRequiresProcessTerminal
    );
    assert!(injected.writer().is_empty());
    assert!(injected.session_state().is_inactive());
}

#[test]
fn failed_restoration_retains_the_lease_until_retry_succeeds() {
    let _serial = lease_test_guard();
    let mut first = Backend::with_process_writer(80, 24, SwitchableFailureWriter::default());
    let mut second = Backend::with_process_writer(80, 24, Vec::new());
    first.init_with_options(writer_options()).unwrap();
    first.writer_mut().fail = true;

    assert!(first.restore().is_err());
    assert!(first.holds_process_terminal_lease());
    assert!(second.init_with_options(writer_options()).is_err());

    first.writer_mut().fail = false;
    first.restore().unwrap();
    assert!(!first.holds_process_terminal_lease());
    second.init_with_options(writer_options()).unwrap();
    second.restore().unwrap();
}

#[test]
fn frame_and_clear_output_also_require_the_process_lease() {
    let _serial = lease_test_guard();
    let mut first = Backend::with_process_writer(80, 24, Vec::new());
    let mut second = Backend::with_process_writer(80, 24, Vec::new());

    first.flush().unwrap();
    assert!(first.holds_process_terminal_lease());
    assert_eq!(
        second.clear().unwrap_err().kind(),
        io::ErrorKind::AlreadyExists
    );

    first.restore().unwrap();
    second.clear().unwrap();
    assert!(second.holds_process_terminal_lease());
    second.restore().unwrap();
}

#[test]
fn fully_rolled_back_initialization_releases_the_process_lease() {
    let _serial = lease_test_guard();
    let mut failing = Backend::with_process_writer(80, 24, OneShotFailureWriter::default());
    let mut successor = Backend::with_process_writer(80, 24, Vec::new());

    assert!(failing.init_with_options(writer_options()).is_err());
    assert!(failing.session_state().is_inactive());
    assert!(!failing.holds_process_terminal_lease());

    successor.init_with_options(writer_options()).unwrap();
    successor.restore().unwrap();
}

#[test]
fn no_op_initialization_does_not_claim_the_process_terminal() {
    let _serial = lease_test_guard();
    let mut first = Backend::with_process_writer(80, 24, Vec::new());
    let mut second = Backend::with_process_writer(80, 24, Vec::new());
    let disabled = TerminalOptions {
        raw_mode: false,
        alternate_screen: false,
        mouse_capture: false,
        bracketed_paste: false,
        hide_cursor: false,
    };

    first.init_with_options(disabled).unwrap();
    second.init_with_options(disabled).unwrap();
    assert!(!first.holds_process_terminal_lease());
    assert!(!second.holds_process_terminal_lease());
}

fn writer_options() -> TerminalOptions {
    TerminalOptions {
        raw_mode: false,
        alternate_screen: true,
        mouse_capture: false,
        bracketed_paste: true,
        hide_cursor: true,
    }
}

fn acquire_reason(error: &io::Error) -> TerminalSessionAcquireError {
    *error
        .get_ref()
        .and_then(|source| source.downcast_ref::<TerminalSessionAcquireError>())
        .expect("session acquisition errors must preserve a structured reason")
}

fn lease_test_guard() -> MutexGuard<'static, ()> {
    PROCESS_LEASE_TEST
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Debug, Default)]
struct OneShotFailureWriter {
    failed: bool,
    data: Vec<u8>,
}

impl Write for OneShotFailureWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if !self.failed {
            self.failed = true;
            return Err(io::Error::other("injected initialization failure"));
        }
        self.data.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
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
