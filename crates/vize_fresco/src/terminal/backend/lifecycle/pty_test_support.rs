//! Shared pseudo-terminal fixture for native lifecycle conformance tests.

use std::{
    mem::MaybeUninit,
    os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd, RawFd},
    process::Stdio,
    ptr,
};

pub(super) struct PtyFixture {
    _master: OwnedFd,
    terminal: Option<OwnedFd>,
    probe: OwnedFd,
    original: libc::termios,
}

impl PtyFixture {
    pub(super) fn open() -> Self {
        let mut master = -1;
        let mut terminal = -1;
        // SAFETY: output pointers are valid and null configuration pointers ask
        // the operating system for default pseudo-terminal attributes.
        assert_eq!(
            unsafe {
                libc::openpty(
                    &mut master,
                    &mut terminal,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            },
            0
        );
        // SAFETY: successful `openpty` returned two newly owned descriptors.
        let master = unsafe { OwnedFd::from_raw_fd(master) };
        // SAFETY: successful `openpty` returned two newly owned descriptors.
        let terminal = unsafe { OwnedFd::from_raw_fd(terminal) };
        // SAFETY: the source descriptor remains owned by `terminal`.
        let probe = unsafe { libc::dup(terminal.as_raw_fd()) };
        assert!(probe >= 0);
        // SAFETY: successful `dup` returned one newly owned descriptor.
        let probe = unsafe { OwnedFd::from_raw_fd(probe) };
        let original = read_terminal_attributes(probe.as_raw_fd());
        Self {
            _master: master,
            terminal: Some(terminal),
            probe,
            original,
        }
    }

    pub(super) fn original(&self) -> libc::termios {
        self.original
    }

    pub(super) fn take_terminal_fd(&mut self) -> RawFd {
        self.terminal.take().unwrap().into_raw_fd()
    }

    pub(super) fn take_child_stdin(&mut self) -> Stdio {
        Stdio::from(self.terminal.take().unwrap())
    }

    pub(super) fn assert_restored(&self) {
        self.assert_attributes_eq(&self.original);
    }

    pub(super) fn assert_attributes_eq(&self, expected: &libc::termios) {
        let actual = read_terminal_attributes(self.probe.as_raw_fd());
        assert_eq!(actual.c_iflag, expected.c_iflag);
        assert_eq!(actual.c_oflag, expected.c_oflag);
        assert_eq!(actual.c_cflag, expected.c_cflag);
        // PENDIN is kernel-observed queue state, not a configurable terminal
        // mode; pseudo terminals may report it after an exact roundtrip.
        assert_eq!(
            actual.c_lflag & !libc::PENDIN,
            expected.c_lflag & !libc::PENDIN
        );
        assert_eq!(actual.c_cc, expected.c_cc);
        // SAFETY: both references point to initialized termios values.
        assert_eq!(unsafe { libc::cfgetispeed(&actual) }, unsafe {
            libc::cfgetispeed(expected)
        });
        // SAFETY: both references point to initialized termios values.
        assert_eq!(unsafe { libc::cfgetospeed(&actual) }, unsafe {
            libc::cfgetospeed(expected)
        });
    }
}

fn read_terminal_attributes(fd: RawFd) -> libc::termios {
    let mut attributes = MaybeUninit::<libc::termios>::zeroed();
    // SAFETY: `attributes` points to writable storage for one termios value.
    assert_eq!(unsafe { libc::tcgetattr(fd, attributes.as_mut_ptr()) }, 0);
    // SAFETY: successful `tcgetattr` initialized the complete zeroed value.
    unsafe { attributes.assume_init() }
}
