//! Windows console-mode snapshots and lock-free emergency restoration.

use std::{
    error::Error,
    fmt, io, ptr,
    sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering},
};

use windows_sys::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE},
    System::Console::{GetConsoleMode, GetStdHandle, STD_INPUT_HANDLE, SetConsoleMode},
};

static RAW_SNAPSHOT_ACTIVE: AtomicBool = AtomicBool::new(false);
static RAW_SNAPSHOT_HANDLE: AtomicIsize = AtomicIsize::new(0);
static RAW_SNAPSHOT_MODE: AtomicU32 = AtomicU32::new(0);
static RAW_SNAPSHOT_OWNS_CROSSTERM: AtomicBool = AtomicBool::new(false);

/// Enable raw input while retaining the exact prior Windows console mode.
///
/// A pre-existing Crossterm raw-mode session is borrowed, not claimed, so
/// Fresco's later restoration cannot disable application-owned raw mode.
pub(in crate::terminal::backend::lifecycle) fn enable_raw_mode() -> io::Result<()> {
    if RAW_SNAPSHOT_ACTIVE.load(Ordering::SeqCst) {
        return Ok(());
    }
    let handle = stdin_handle()?;
    let crossterm_was_raw = crossterm::terminal::is_raw_mode_enabled()?;
    let original = console_mode(handle)?;
    publish_snapshot(handle, original, !crossterm_was_raw);
    if !crossterm_was_raw && let Err(enable) = crossterm::terminal::enable_raw_mode() {
        return fail_enable_transition(handle, original, enable);
    }
    Ok(())
}

/// Restore and release the native raw-mode snapshot.
pub(in crate::terminal::backend::lifecycle) fn disable_raw_mode() -> io::Result<()> {
    let Some(snapshot) = RawSnapshot::current() else {
        return Ok(());
    };
    let mut restoration = set_console_mode(snapshot.handle, snapshot.original);
    if restoration.is_ok() && snapshot.owns_crossterm_raw_mode {
        restoration = crossterm::terminal::disable_raw_mode();
    }
    if restoration.is_ok() {
        RAW_SNAPSHOT_ACTIVE.store(false, Ordering::SeqCst);
    }
    restoration
}

/// Return whether raw mode owns a native snapshot requiring restoration.
pub(in crate::terminal::backend::lifecycle) fn raw_mode_requires_restoration() -> bool {
    RAW_SNAPSHOT_ACTIVE.load(Ordering::SeqCst)
}

/// Restore raw mode from a panic or console-control handler without
/// releasing conservative state, allowing normal cleanup to retry.
pub(in crate::terminal::backend::lifecycle) fn emergency_restore_raw_mode() -> bool {
    let Some(snapshot) = RawSnapshot::current() else {
        return true;
    };
    set_console_mode(snapshot.handle, snapshot.original).is_ok()
}

#[derive(Clone, Copy)]
struct RawSnapshot {
    handle: HANDLE,
    original: u32,
    owns_crossterm_raw_mode: bool,
}

impl RawSnapshot {
    fn current() -> Option<Self> {
        if !RAW_SNAPSHOT_ACTIVE.load(Ordering::SeqCst) {
            return None;
        }
        let handle = RAW_SNAPSHOT_HANDLE.load(Ordering::SeqCst) as HANDLE;
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return None;
        }
        Some(Self {
            handle,
            original: RAW_SNAPSHOT_MODE.load(Ordering::SeqCst),
            owns_crossterm_raw_mode: RAW_SNAPSHOT_OWNS_CROSSTERM.load(Ordering::SeqCst),
        })
    }
}

fn publish_snapshot(handle: HANDLE, original: u32, owns_crossterm_raw_mode: bool) {
    RAW_SNAPSHOT_HANDLE.store(handle as isize, Ordering::SeqCst);
    RAW_SNAPSHOT_MODE.store(original, Ordering::SeqCst);
    RAW_SNAPSHOT_OWNS_CROSSTERM.store(owns_crossterm_raw_mode, Ordering::SeqCst);
    RAW_SNAPSHOT_ACTIVE.store(true, Ordering::SeqCst);
}

fn fail_enable_transition(handle: HANDLE, original: u32, enable: io::Error) -> io::Result<()> {
    let rollback = set_console_mode(handle, original);
    if rollback.is_ok() {
        RAW_SNAPSHOT_ACTIVE.store(false, Ordering::SeqCst);
        return Err(enable);
    }
    Err(combine_transition_errors(enable, rollback.unwrap_err()))
}

fn stdin_handle() -> io::Result<HANDLE> {
    // SAFETY: `STD_INPUT_HANDLE` is the documented selector for the
    // process standard input handle.
    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    Ok(handle)
}

fn console_mode(handle: HANDLE) -> io::Result<u32> {
    let mut mode = 0_u32;
    // SAFETY: `mode` points to writable storage and `handle` was accepted
    // by `GetStdHandle` or stored from a prior successful call.
    if unsafe { GetConsoleMode(handle, ptr::addr_of_mut!(mode)) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(mode)
    }
}

fn set_console_mode(handle: HANDLE, mode: u32) -> io::Result<()> {
    // SAFETY: `mode` is the exact value previously returned by
    // `GetConsoleMode` for this console input handle.
    if unsafe { SetConsoleMode(handle, mode) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn combine_transition_errors(enable: io::Error, rollback: io::Error) -> io::Error {
    let kind = enable.kind();
    io::Error::new(kind, RawModeTransitionFailure { enable, rollback })
}

#[derive(Debug)]
struct RawModeTransitionFailure {
    enable: io::Error,
    rollback: io::Error,
}

impl fmt::Display for RawModeTransitionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "enabling raw mode failed: {}; restoring the original console mode also failed: {}",
            self.enable, self.rollback
        )
    }
}

impl Error for RawModeTransitionFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.enable)
    }
}
