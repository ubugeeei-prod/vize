use std::ptr;

use windows_sys::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE},
    System::Console::{
        ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, GetConsoleMode, GetStdHandle, STD_INPUT_HANDLE,
        SetConsoleMode,
    },
};

use super::windows::{disable_raw_mode, enable_raw_mode, raw_mode_requires_restoration};

/// Crossterm's Windows teardown re-enables the cooked-input flags instead of
/// writing back the mode it observed, so restoration must finish by replaying
/// Fresco's snapshot verbatim. The pre-Fresco mode below keeps line input (so
/// Fresco owns the Crossterm raw-mode session) while echo stays disabled, which
/// is exactly the bit Crossterm would otherwise resurrect.
#[test]
fn disable_raw_mode_restores_a_non_default_input_mode_after_crossterm_cleanup() {
    let Some((handle, original)) = console_input_mode() else {
        return;
    };
    let expected = (original | ENABLE_LINE_INPUT) & !ENABLE_ECHO_INPUT;
    assert!(set_console_mode(handle, expected));

    let enabled = enable_raw_mode();
    let requires_restoration = raw_mode_requires_restoration();
    let disabled = enabled.is_ok().then(disable_raw_mode);
    let restored = console_mode(handle);
    assert!(set_console_mode(handle, original));

    enabled.unwrap();
    assert!(requires_restoration);
    disabled.unwrap().unwrap();
    assert_eq!(
        restored,
        Some(expected),
        "Crossterm cleanup must not re-enable echo input over Fresco's snapshot"
    );
    assert!(!raw_mode_requires_restoration());
}

fn console_input_mode() -> Option<(HANDLE, u32)> {
    // SAFETY: `STD_INPUT_HANDLE` is the documented selector for the process
    // standard input handle. The handle is only inspected by `GetConsoleMode`.
    let handle: HANDLE = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return None;
    }
    console_mode(handle).map(|mode| (handle, mode))
}

fn console_mode(handle: HANDLE) -> Option<u32> {
    let mut mode = 0_u32;
    // SAFETY: `mode` points to writable storage and `handle` came from a prior
    // successful `GetStdHandle(STD_INPUT_HANDLE)` call.
    (unsafe { GetConsoleMode(handle, ptr::addr_of_mut!(mode)) } != 0).then_some(mode)
}

fn set_console_mode(handle: HANDLE, mode: u32) -> bool {
    // SAFETY: `mode` was derived from a value `GetConsoleMode` returned for
    // this console input handle.
    unsafe { SetConsoleMode(handle, mode) != 0 }
}
