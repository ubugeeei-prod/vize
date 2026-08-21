use std::{
    os::fd::RawFd,
    sync::{
        Barrier, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use super::unix::{
    RawSnapshotReader, disable_raw_mode, emergency_restore_raw_mode, enable_raw_mode_on_owned_fd,
    raw_mode_requires_restoration,
};
use crate::terminal::backend::lifecycle::pty_test_support::PtyFixture;

static RAW_MODE_TEST: Mutex<()> = Mutex::new(());

#[test]
fn raw_mode_and_emergency_restore_preserve_exact_attributes() {
    let _serial = raw_mode_test_guard();
    let mut pty = PtyFixture::open();
    let original = pty.original();
    let mut expected_raw = original;
    // SAFETY: `expected_raw` is an initialized termios value.
    unsafe { libc::cfmakeraw(&mut expected_raw) };

    enable_raw_mode_on_owned_fd(pty.take_terminal_fd()).unwrap();
    pty.assert_attributes_eq(&expected_raw);
    assert!(raw_mode_requires_restoration());

    assert!(emergency_restore_raw_mode());
    pty.assert_restored();
    assert!(raw_mode_requires_restoration());

    disable_raw_mode().unwrap();
    pty.assert_restored();
    assert!(!raw_mode_requires_restoration());
}

#[test]
fn concurrent_emergency_readers_drain_before_snapshot_release() {
    let _serial = raw_mode_test_guard();
    let mut pty = PtyFixture::open();
    enable_raw_mode_on_owned_fd(pty.take_terminal_fd()).unwrap();

    let readers_ready = Barrier::new(9);
    let release_readers = Barrier::new(9);
    let disable_completed = AtomicBool::new(false);
    thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                let snapshot = RawSnapshotReader::acquire().unwrap();
                readers_ready.wait();
                release_readers.wait();
                drop(snapshot);
            });
        }
        readers_ready.wait();
        let disable = scope.spawn(|| {
            disable_raw_mode().unwrap();
            disable_completed.store(true, Ordering::Release);
        });
        while raw_mode_requires_restoration() {
            thread::yield_now();
        }
        assert!(!disable_completed.load(Ordering::Acquire));
        release_readers.wait();
        disable.join().unwrap();
    });

    assert!(disable_completed.load(Ordering::Acquire));
    assert!(!raw_mode_requires_restoration());
    pty.assert_restored();
}

#[test]
fn invalid_terminal_descriptor_is_closed_without_publishing_a_snapshot() {
    let _serial = raw_mode_test_guard();
    // SAFETY: the path is a static NUL-terminated C string.
    let fd = unsafe { libc::open(c"/dev/null".as_ptr(), libc::O_RDWR) };
    assert!(fd >= 0);
    let fd = duplicate_outside_common_test_fd_range(fd);

    let error = enable_raw_mode_on_owned_fd(fd).unwrap_err();
    assert!(matches!(
        error.raw_os_error(),
        Some(libc::ENOTTY | libc::ENODEV)
    ));
    assert!(!raw_mode_requires_restoration());
    // SAFETY: `F_GETFD` only inspects the descriptor number.
    assert_eq!(unsafe { libc::fcntl(fd, libc::F_GETFD) }, -1);
}

#[test]
fn second_owned_fd_does_not_replace_or_modify_active_snapshot() {
    let _serial = raw_mode_test_guard();
    let mut first = PtyFixture::open();
    let first_raw_fd = first.take_terminal_fd();
    enable_raw_mode_on_owned_fd(first_raw_fd).unwrap();

    let mut second = PtyFixture::open();
    let second_fd = second.take_terminal_fd();
    enable_raw_mode_on_owned_fd(second_fd).unwrap();

    second.assert_restored();
    assert!(emergency_restore_raw_mode());
    first.assert_restored();
    second.assert_restored();
    disable_raw_mode().unwrap();
}

fn raw_mode_test_guard() -> MutexGuard<'static, ()> {
    RAW_MODE_TEST
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn duplicate_outside_common_test_fd_range(fd: RawFd) -> RawFd {
    let minimum_fd = if open_max() > 257 { 256 } else { 0 };
    // SAFETY: `fd` is valid, and `F_DUPFD_CLOEXEC` takes one integer minimum
    // descriptor. The original low descriptor is closed before returning.
    let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, minimum_fd) };
    // SAFETY: `fd` is still the original descriptor and has not been
    // transferred to raw-mode ownership.
    let close_result = unsafe { libc::close(fd) };
    assert_eq!(close_result, 0);
    assert!(duplicate >= 0);
    duplicate
}

fn open_max() -> libc::c_long {
    // SAFETY: `_SC_OPEN_MAX` does not require additional arguments.
    let value = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) };
    if value > 0 { value } else { 0 }
}
