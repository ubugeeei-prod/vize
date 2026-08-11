use super::*;

#[cfg(unix)]
use std::{
    io,
    mem::MaybeUninit,
    os::unix::process::ExitStatusExt,
    process::Command,
    ptr,
    sync::atomic::{AtomicI32, AtomicUsize, Ordering},
};

#[cfg(unix)]
use crate::terminal::backend::lifecycle::{
    panic_hook::PRESENTATION_RESETS, pty_test_support::PtyFixture,
};
#[cfg(unix)]
use crate::terminal::{Backend, TerminalMode, TerminalOptions};

#[cfg(unix)]
const CHAIN_CHILD_MARKER: &str = "VIZE_FRESCO_SIGNAL_CHAIN_CHILD";
#[cfg(unix)]
const CHAIN_CHILD_VALUE: &str = "signal-chain-v1";
#[cfg(unix)]
const CHAIN_SUBPROCESS_TEST: &str = concat!(
    "terminal::backend::lifecycle::signal_hook::tests::",
    "signal_hook_restores_and_preserves_existing_actions"
);
#[cfg(unix)]
const DEFAULT_CHILD_MARKER: &str = "VIZE_FRESCO_SIGNAL_DEFAULT_CHILD";
#[cfg(unix)]
const DEFAULT_CHILD_VALUE: &str = "signal-default-v1";
#[cfg(unix)]
const DEFAULT_SUBPROCESS_TEST: &str = concat!(
    "terminal::backend::lifecycle::signal_hook::tests::",
    "signal_hook_restores_before_default_termination"
);

#[cfg(unix)]
static SIMPLE_HANDLER_CALLS: AtomicUsize = AtomicUsize::new(0);
#[cfg(unix)]
static SIGINFO_HANDLER_CALLS: AtomicUsize = AtomicUsize::new(0);
#[cfg(unix)]
static SIGINFO_SIGNAL: AtomicI32 = AtomicI32::new(0);
#[cfg(unix)]
static OBSERVED_ERRNO: AtomicI32 = AtomicI32::new(0);

#[cfg(unix)]
extern "C" fn simple_handler(_: libc::c_int) {
    SIMPLE_HANDLER_CALLS.fetch_add(1, Ordering::Relaxed);
    // SAFETY: this test runs only on platforms accepted by the production
    // installer, where `errno_location` returns this thread's live errno slot.
    OBSERVED_ERRNO.store(unsafe { *errno_location() }, Ordering::Relaxed);
}

#[cfg(unix)]
extern "C" fn siginfo_handler(
    signal: libc::c_int,
    information: *mut libc::siginfo_t,
    _: *mut libc::c_void,
) {
    SIGINFO_HANDLER_CALLS.fetch_add(1, Ordering::Relaxed);
    if !information.is_null() {
        // SAFETY: POSIX supplies a live `siginfo_t` for an `SA_SIGINFO` action.
        SIGINFO_SIGNAL.store(unsafe { (*information).si_signo }, Ordering::Relaxed);
    }
    let _ = signal;
}

#[cfg(not(unix))]
#[test]
fn unsupported_platform_is_explicit_and_does_not_install() {
    assert!(matches!(
        install_terminal_signal_hook(),
        Err(TerminalSignalHookError::UnsupportedPlatform)
    ));
}

#[test]
fn unknown_signal_has_stable_fallback_name() {
    assert_eq!(signal_name(i32::MAX), "unknown signal");
}

#[cfg(unix)]
#[test]
fn partial_install_rolls_back_every_prior_action_in_reverse_order() {
    let previous = SUPERVISED_SIGNALS
        .into_iter()
        .map(|signal| inspect_action(signal).unwrap())
        .collect::<Vec<_>>();
    let mut calls = Vec::new();

    let failure = install_actions_with(&previous, |signal, _| {
        calls.push(signal);
        match calls.len() {
            3 => Err(io::Error::from_raw_os_error(41)),
            4 => Err(io::Error::from_raw_os_error(42)),
            _ => Ok(()),
        }
    })
    .unwrap_err();

    assert_eq!(
        calls,
        [
            libc::SIGINT,
            libc::SIGTERM,
            libc::SIGHUP,
            libc::SIGTERM,
            libc::SIGINT,
        ]
    );
    assert_eq!(failure.signal, libc::SIGHUP);
    assert_eq!(failure.source.raw_os_error(), Some(41));
    assert_eq!(failure.rollback_failures.len(), 1);
    assert_eq!(failure.rollback_failures[0].signal(), libc::SIGTERM);
    assert_eq!(
        failure.rollback_failures[0].error().raw_os_error(),
        Some(42)
    );
}

#[cfg(unix)]
#[test]
fn signal_hook_restores_and_preserves_existing_actions() {
    if std::env::var(CHAIN_CHILD_MARKER).as_deref() != Ok(CHAIN_CHILD_VALUE) {
        let mut pty = PtyFixture::open();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHAIN_SUBPROCESS_TEST, "--nocapture"])
            .env(CHAIN_CHILD_MARKER, CHAIN_CHILD_VALUE)
            .stdin(pty.take_child_stdin())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "child failed\nstdout: {}\nstderr: {}",
            std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8>"),
            std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8>")
        );
        assert_emergency_resets(&output.stdout);
        pty.assert_restored();
        return;
    }

    install_test_action(
        libc::SIGTERM,
        simple_handler as *const () as usize,
        libc::SA_RESTART,
    );
    install_test_action(
        libc::SIGINT,
        siginfo_handler as *const () as usize,
        libc::SA_SIGINFO,
    );
    install_test_action(libc::SIGQUIT, libc::SIG_IGN, 0);

    assert_eq!(
        install_terminal_signal_hook().unwrap(),
        TerminalSignalHookInstallation::Installed
    );
    assert_eq!(
        install_terminal_signal_hook().unwrap(),
        TerminalSignalHookInstallation::AlreadyInstalled
    );

    let installed = inspect_action(libc::SIGTERM).unwrap();
    assert_eq!(
        installed.sa_sigaction,
        terminal_signal_handler as *const () as usize
    );
    #[allow(unused_assignments)]
    let mut restart = installed.sa_flags;
    restart = libc::SA_RESTART as _;
    #[allow(unused_assignments)]
    let mut siginfo = installed.sa_flags;
    siginfo = libc::SA_SIGINFO as _;
    assert_ne!(installed.sa_flags & restart, 0);
    assert_ne!(installed.sa_flags & siginfo, 0);
    // The test action blocks SIGUSR1. Fresco must not silently replace that
    // application-owned mask while wrapping it.
    // SAFETY: `installed.sa_mask` was initialized by `sigaction`.
    assert_eq!(
        unsafe { libc::sigismember(&installed.sa_mask, libc::SIGUSR1) },
        1
    );

    let backend = active_backend();
    // SAFETY: this platform is accepted by the installer and exposes a live
    // thread-local errno slot.
    unsafe { *errno_location() = libc::E2BIG };
    // SAFETY: the three signals have valid actions installed above.
    assert_eq!(unsafe { libc::raise(libc::SIGTERM) }, 0);
    assert_eq!(unsafe { libc::raise(libc::SIGINT) }, 0);
    assert_eq!(unsafe { libc::raise(libc::SIGQUIT) }, 0);

    assert_eq!(SIMPLE_HANDLER_CALLS.load(Ordering::Relaxed), 1);
    assert_eq!(SIGINFO_HANDLER_CALLS.load(Ordering::Relaxed), 1);
    assert_eq!(SIGINFO_SIGNAL.load(Ordering::Relaxed), libc::SIGINT);
    assert_eq!(OBSERVED_ERRNO.load(Ordering::Relaxed), libc::E2BIG);

    // The parent verifies native terminal flags. Avoid ordinary restoration so
    // every captured reset byte is attributable to the signal path.
    std::mem::forget(backend);
}

#[cfg(unix)]
#[test]
fn signal_hook_restores_before_default_termination() {
    if std::env::var(DEFAULT_CHILD_MARKER).as_deref() != Ok(DEFAULT_CHILD_VALUE) {
        let mut pty = PtyFixture::open();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", DEFAULT_SUBPROCESS_TEST, "--nocapture"])
            .env(DEFAULT_CHILD_MARKER, DEFAULT_CHILD_VALUE)
            .stdin(pty.take_child_stdin())
            .output()
            .unwrap();
        assert_eq!(output.status.signal(), Some(libc::SIGHUP));
        assert_emergency_resets(&output.stdout);
        pty.assert_restored();
        return;
    }

    install_test_action(libc::SIGHUP, libc::SIG_DFL, 0);
    install_terminal_signal_hook().unwrap();
    let _backend = active_backend();

    // SAFETY: SIGHUP has Fresco's wrapper around its default disposition. The
    // call never returns: the wrapper restores, reinstates default, and re-raises.
    unsafe { libc::raise(libc::SIGHUP) };
    unreachable!("default SIGHUP disposition must terminate the child");
}

#[cfg(unix)]
fn install_test_action(signal: libc::c_int, handler: usize, flags: libc::c_int) {
    let mut action = MaybeUninit::<libc::sigaction>::zeroed();
    // SAFETY: zero is a valid base representation for `sigaction`; all required
    // fields are initialized before it is passed to libc.
    let action = unsafe { action.assume_init_mut() };
    action.sa_sigaction = handler;
    action.sa_flags = flags as _;
    // SAFETY: the mask points to initialized storage owned by `action`.
    assert_eq!(unsafe { libc::sigemptyset(&mut action.sa_mask) }, 0);
    assert_eq!(
        unsafe { libc::sigaddset(&mut action.sa_mask, libc::SIGUSR1) },
        0
    );
    // SAFETY: `action` is complete and the signal is one of the supported set.
    assert_eq!(
        unsafe { libc::sigaction(signal, action, ptr::null_mut()) },
        0
    );
}

#[cfg(unix)]
fn active_backend() -> Backend<io::Stdout> {
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
    backend
}

#[cfg(unix)]
fn assert_emergency_resets(output: &[u8]) {
    for (_, reset) in PRESENTATION_RESETS {
        assert!(
            output.windows(reset.len()).any(|bytes| bytes == reset),
            "missing emergency reset {reset:?} in {output:?}"
        );
    }
}
