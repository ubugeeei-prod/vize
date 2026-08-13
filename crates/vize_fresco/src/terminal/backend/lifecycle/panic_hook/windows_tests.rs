use std::{
    panic,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::*;

static PREVIOUS_HOOK_CALLS: AtomicUsize = AtomicUsize::new(0);

const WINDOWS_CHILD_MARKER: &str = "VIZE_FRESCO_WINDOWS_PANIC_HOOK_CHILD";
const WINDOWS_CHILD_VALUE: &str = "windows-panic-hook-v1";
const WINDOWS_SUBPROCESS_TEST: &str = concat!(
    "terminal::backend::lifecycle::panic_hook::windows_tests::",
    "windows_panic_hook_installation_is_supported_and_idempotent"
);

#[test]
fn windows_panic_hook_installation_is_supported_and_idempotent() {
    if std::env::var(WINDOWS_CHILD_MARKER).as_deref() != Ok(WINDOWS_CHILD_VALUE) {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", WINDOWS_SUBPROCESS_TEST, "--nocapture"])
            .env(WINDOWS_CHILD_MARKER, WINDOWS_CHILD_VALUE)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "child failed\nstdout: {}\nstderr: {}",
            std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8>"),
            std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8>")
        );
        return;
    }

    PREVIOUS_HOOK_CALLS.store(0, Ordering::Release);
    panic::set_hook(Box::new(|_| {
        PREVIOUS_HOOK_CALLS.fetch_add(1, Ordering::AcqRel);
    }));
    let original_input = windows_console_input_mode();

    assert!(matches!(
        install_terminal_panic_hook().unwrap(),
        TerminalPanicHookInstallation::Installed | TerminalPanicHookInstallation::AlreadyInstalled
    ));
    assert_eq!(
        install_terminal_panic_hook().unwrap(),
        TerminalPanicHookInstallation::AlreadyInstalled
    );

    let raw_enabled = super::super::raw_mode::enable_raw_mode().is_ok();
    let panic_result = panic::catch_unwind(|| {
        panic!("exercise Windows terminal panic restoration");
    });
    assert!(panic_result.is_err());
    assert_eq!(PREVIOUS_HOOK_CALLS.load(Ordering::Acquire), 1);

    if let Some((handle, mode)) = original_input {
        assert!(
            raw_enabled,
            "raw mode should enable when a console input handle exists"
        );
        assert_eq!(
            windows_console_mode(handle),
            Some(mode),
            "panic hook should restore the original Windows input console mode"
        );
    }
    if raw_enabled {
        super::super::raw_mode::disable_raw_mode().unwrap();
    }
}

fn windows_console_input_mode() -> Option<(windows_sys::Win32::Foundation::HANDLE, u32)> {
    use std::ptr;
    use windows_sys::Win32::{
        Foundation::{HANDLE, INVALID_HANDLE_VALUE},
        System::Console::{GetConsoleMode, GetStdHandle, STD_INPUT_HANDLE},
    };

    // SAFETY: `STD_INPUT_HANDLE` is the documented selector for the process
    // standard input handle. The handle is only inspected by `GetConsoleMode`.
    let handle: HANDLE = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return None;
    }
    let mut mode = 0_u32;
    // SAFETY: `mode` points to writable storage for the duration of the call.
    (unsafe { GetConsoleMode(handle, ptr::addr_of_mut!(mode)) } != 0).then_some((handle, mode))
}

fn windows_console_mode(handle: windows_sys::Win32::Foundation::HANDLE) -> Option<u32> {
    use std::ptr;
    use windows_sys::Win32::System::Console::GetConsoleMode;

    let mut mode = 0_u32;
    // SAFETY: `mode` points to writable storage and `handle` came from a prior
    // successful `GetStdHandle(STD_INPUT_HANDLE)`/`GetConsoleMode` pair.
    (unsafe { GetConsoleMode(handle, ptr::addr_of_mut!(mode)) } != 0).then_some(mode)
}
