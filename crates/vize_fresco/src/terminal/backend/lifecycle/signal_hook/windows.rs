//! Windows console-control handler installation.

use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

use windows_sys::Win32::{
    Foundation::{FALSE, TRUE},
    System::Console::{
        CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT,
        SetConsoleCtrlHandler,
    },
};

use crate::terminal::backend::lifecycle::{
    TerminalSignalHookError, TerminalSignalHookInstallation,
    lease::emergency_presentation_modes,
    panic_hook::{emergency_write_stdout, restore_owned_presentation_modes},
    raw_mode::emergency_restore_raw_mode,
};

static SIGNAL_HOOK_INSTALLATION: Mutex<()> = Mutex::new(());
static SIGNAL_HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

pub(super) fn install() -> Result<TerminalSignalHookInstallation, TerminalSignalHookError> {
    if SIGNAL_HOOK_INSTALLED.load(Ordering::Acquire) {
        return Ok(TerminalSignalHookInstallation::AlreadyInstalled);
    }

    let _installation = SIGNAL_HOOK_INSTALLATION
        .lock()
        .map_err(|_| TerminalSignalHookError::InstallationPoisoned)?;
    if SIGNAL_HOOK_INSTALLED.load(Ordering::Acquire) {
        return Ok(TerminalSignalHookInstallation::AlreadyInstalled);
    }

    // SAFETY: `terminal_control_handler` is a static callback with the ABI
    // required by `SetConsoleCtrlHandler` and remains valid for process life.
    if unsafe { SetConsoleCtrlHandler(Some(terminal_control_handler), TRUE) } == 0 {
        return Err(TerminalSignalHookError::InstallAction {
            signal: CTRL_C_EVENT as i32,
            source: std::io::Error::last_os_error(),
            rollback_failures: Vec::new(),
        });
    }
    SIGNAL_HOOK_INSTALLED.store(true, Ordering::Release);
    Ok(TerminalSignalHookInstallation::Installed)
}

unsafe extern "system" fn terminal_control_handler(control: u32) -> windows_sys::core::BOOL {
    if !is_supervised_control(control) {
        return FALSE;
    }
    restore_owned_presentation_modes(emergency_presentation_modes(), emergency_write_stdout);
    let _ = emergency_restore_raw_mode();
    // Returning FALSE preserves the process handler chain and default action.
    FALSE
}

const fn is_supervised_control(control: u32) -> bool {
    matches!(
        control,
        CTRL_C_EVENT
            | CTRL_BREAK_EVENT
            | CTRL_CLOSE_EVENT
            | CTRL_LOGOFF_EVENT
            | CTRL_SHUTDOWN_EVENT
    )
}
