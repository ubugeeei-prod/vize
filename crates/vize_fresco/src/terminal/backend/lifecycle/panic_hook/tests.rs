#[cfg(unix)]
use std::{
    io,
    panic::{self, AssertUnwindSafe},
    sync::Barrier,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

#[cfg(unix)]
use std::{process::Command, sync::atomic::AtomicUsize};

use crossterm::{
    cursor::{SetCursorStyle, Show},
    event::{DisableBracketedPaste, DisableMouseCapture},
    terminal::LeaveAlternateScreen,
};

use super::*;
#[cfg(unix)]
use crate::terminal::backend::lifecycle::pty_test_support::PtyFixture;
#[cfg(unix)]
use crate::terminal::{Backend, TerminalOptions};

#[cfg(unix)]
static PREVIOUS_HOOK_CALLS: AtomicUsize = AtomicUsize::new(0);
#[cfg(unix)]
static PANICKING_INSTALL_REJECTED: AtomicBool = AtomicBool::new(false);
#[cfg(unix)]
const CHILD_MARKER: &str = "VIZE_FRESCO_PANIC_HOOK_CHILD";
#[cfg(unix)]
const CHILD_MARKER_VALUE: &str = "panic-hook-v1";
#[cfg(unix)]
const SUBPROCESS_TEST: &str = concat!(
    "terminal::backend::lifecycle::panic_hook::tests::",
    "panic_hook_subprocess_restores_terminal_and_chains_once"
);
#[cfg(unix)]
const NORMAL_RAW_CHILD_MARKER: &str = "VIZE_FRESCO_NORMAL_RAW_CHILD";
#[cfg(unix)]
const NORMAL_RAW_CHILD_VALUE: &str = "normal-raw-v1";
#[cfg(unix)]
const NORMAL_RAW_SUBPROCESS_TEST: &str = concat!(
    "terminal::backend::lifecycle::panic_hook::tests::",
    "normal_raw_mode_subprocess_keeps_crossterm_state_consistent"
);
#[test]
fn emergency_sequences_match_crossterm_commands() {
    assert_command_bytes(DISABLE_MOUSE_CAPTURE, DisableMouseCapture);
    assert_command_bytes(DISABLE_BRACKETED_PASTE, DisableBracketedPaste);
    assert_command_bytes(LEAVE_ALTERNATE_SCREEN, LeaveAlternateScreen);
    assert_command_bytes(RESET_CURSOR_SHAPE, SetCursorStyle::DefaultUserShape);
    assert_command_bytes(SHOW_CURSOR, Show);
}

#[test]
fn presentation_modes_restore_in_normal_cleanup_order() {
    let owned_modes = PRESENTATION_RESETS
        .iter()
        .fold(TerminalMode::RawMode.bit(), |modes, (mode, _)| {
            modes | mode.bit()
        });
    let mut actual = Vec::new();

    restore_owned_presentation_modes(owned_modes, |bytes| {
        actual.extend_from_slice(bytes);
        true
    });

    let expected = PRESENTATION_RESETS
        .iter()
        .flat_map(|(_, bytes)| bytes.iter().copied())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn rejected_reset_does_not_block_later_owned_modes() {
    let owned_modes = TerminalMode::MouseCapture.bit()
        | TerminalMode::AlternateScreen.bit()
        | TerminalMode::CursorVisibility.bit();
    let mut attempted_lengths = Vec::new();

    restore_owned_presentation_modes(owned_modes, |bytes| {
        attempted_lengths.push(bytes.len());
        bytes != LEAVE_ALTERNATE_SCREEN
    });

    assert_eq!(
        attempted_lengths,
        [
            DISABLE_MOUSE_CAPTURE.len(),
            LEAVE_ALTERNATE_SCREEN.len(),
            SHOW_CURSOR.len(),
        ]
    );
}

#[cfg(not(any(unix, windows)))]
#[test]
fn unsupported_platform_is_explicit_and_does_not_install() {
    assert_eq!(
        install_terminal_panic_hook(),
        Err(TerminalPanicHookError::UnsupportedPlatform)
    );
}

#[cfg(unix)]
#[test]
fn panic_hook_subprocess_restores_terminal_and_chains_once() {
    if std::env::var(CHILD_MARKER).as_deref() != Ok(CHILD_MARKER_VALUE) {
        let mut pty = PtyFixture::open();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", SUBPROCESS_TEST, "--nocapture"])
            .env(CHILD_MARKER, CHILD_MARKER_VALUE)
            .stdin(pty.take_child_stdin())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "child failed\nstdout: {}\nstderr: {}",
            std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8>"),
            std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8>")
        );
        for (_, reset) in PRESENTATION_RESETS {
            assert!(
                output
                    .stdout
                    .windows(reset.len())
                    .any(|bytes| bytes == reset),
                "missing emergency reset {reset:?} in {:?}",
                output.stdout
            );
        }
        pty.assert_restored();
        return;
    }

    PANICKING_INSTALL_REJECTED.store(false, Ordering::Release);
    panic::set_hook(Box::new(|_| {
        PANICKING_INSTALL_REJECTED.store(
            matches!(
                install_terminal_panic_hook(),
                Err(TerminalPanicHookError::PanickingThread)
            ),
            Ordering::Release,
        );
    }));
    let rejected_install = panic::catch_unwind(|| {
        panic!("reject installation from a panicking thread");
    });
    assert!(rejected_install.is_err());
    assert!(PANICKING_INSTALL_REJECTED.load(Ordering::Acquire));

    PREVIOUS_HOOK_CALLS.store(0, Ordering::Release);
    panic::set_hook(Box::new(|_| {
        PREVIOUS_HOOK_CALLS.fetch_add(1, Ordering::AcqRel);
    }));
    let installation_barrier = Barrier::new(8);
    let installations = thread::scope(|scope| {
        let installers = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    installation_barrier.wait();
                    install_terminal_panic_hook().unwrap()
                })
            })
            .collect::<Vec<_>>();
        installers
            .into_iter()
            .map(|installer| installer.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(
        installations
            .iter()
            .filter(|installation| **installation == TerminalPanicHookInstallation::Installed)
            .count(),
        1
    );
    assert_eq!(
        installations
            .iter()
            .filter(|installation| {
                **installation == TerminalPanicHookInstallation::AlreadyInstalled
            })
            .count(),
        7
    );
    assert_eq!(
        install_terminal_panic_hook().unwrap(),
        TerminalPanicHookInstallation::AlreadyInstalled
    );

    let mut backend = Backend::with_process_writer(8, 2, io::stdout());
    backend
        .init_with_options(TerminalOptions {
            raw_mode: true,
            alternate_screen: true,
            mouse_capture: true,
            bracketed_paste: true,
            hide_cursor: true,
        })
        .unwrap();
    backend.acquire_mode(TerminalMode::CursorShape);

    let panic_result = panic::catch_unwind(AssertUnwindSafe(|| {
        panic!("exercise terminal panic restoration");
    }));
    assert!(panic_result.is_err());
    assert_eq!(PREVIOUS_HOOK_CALLS.load(Ordering::Acquire), 1);
    assert!(crossterm::terminal::is_raw_mode_enabled().unwrap());

    // Avoid normal restoration so the parent can prove that the observed
    // reset sequences came from the panic hook rather than `Drop`.
    std::mem::forget(backend);
}

#[cfg(unix)]
#[test]
fn normal_raw_mode_subprocess_keeps_crossterm_state_consistent() {
    if std::env::var(NORMAL_RAW_CHILD_MARKER).as_deref() != Ok(NORMAL_RAW_CHILD_VALUE) {
        let mut pty = PtyFixture::open();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", NORMAL_RAW_SUBPROCESS_TEST, "--nocapture"])
            .env(NORMAL_RAW_CHILD_MARKER, NORMAL_RAW_CHILD_VALUE)
            .stdin(pty.take_child_stdin())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "child failed\nstdout: {}\nstderr: {}",
            std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8>"),
            std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8>")
        );
        pty.assert_restored();
        return;
    }

    let mut backend = Backend::with_process_writer(8, 2, Vec::new());
    backend
        .init_with_options(TerminalOptions {
            raw_mode: true,
            alternate_screen: false,
            mouse_capture: false,
            bracketed_paste: false,
            hide_cursor: false,
        })
        .unwrap();
    assert!(crossterm::terminal::is_raw_mode_enabled().unwrap());
    backend.restore().unwrap();
    assert!(!crossterm::terminal::is_raw_mode_enabled().unwrap());

    crossterm::terminal::enable_raw_mode().unwrap();
    backend
        .init_with_options(TerminalOptions {
            raw_mode: true,
            alternate_screen: false,
            mouse_capture: false,
            bracketed_paste: false,
            hide_cursor: false,
        })
        .unwrap();
    backend.restore().unwrap();
    assert!(crossterm::terminal::is_raw_mode_enabled().unwrap());
    crossterm::terminal::disable_raw_mode().unwrap();
}

fn assert_command_bytes(command_bytes: &[u8], command: impl crossterm::Command) {
    // `queue!` routes some commands through the Windows console API instead of
    // the byte stream, so it cannot derive expectations on a headless runner.
    // `write_ansi` yields the escape sequence on every platform, which is
    // exactly what the emergency constants must reproduce.
    let mut expected = vize_carton::String::default();
    command.write_ansi(&mut expected).unwrap();
    assert_eq!(command_bytes, expected.as_bytes());
}
