//! Windows console-mode snapshots and lock-free emergency restoration.

use std::{
    error::Error,
    fmt, io, ptr,
    sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicUsize, Ordering},
};

use windows_sys::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE},
    System::Console::{
        GetConsoleMode, GetStdHandle, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, SetConsoleMode,
    },
};

static RAW_SNAPSHOT_ACTIVE: AtomicBool = AtomicBool::new(false);
static RAW_SNAPSHOT_PUBLISHED: AtomicBool = AtomicBool::new(false);
static RAW_SNAPSHOT_READERS: AtomicUsize = AtomicUsize::new(0);
static RAW_SNAPSHOT_HANDLE: AtomicIsize = AtomicIsize::new(0);
static RAW_SNAPSHOT_MODE: AtomicU32 = AtomicU32::new(0);
static RAW_SNAPSHOT_OUTPUT_HANDLE: AtomicIsize = AtomicIsize::new(0);
static RAW_SNAPSHOT_OUTPUT_MODE: AtomicU32 = AtomicU32::new(0);
static RAW_SNAPSHOT_HAS_OUTPUT_MODE: AtomicBool = AtomicBool::new(false);
static RAW_SNAPSHOT_OWNS_CROSSTERM: AtomicBool = AtomicBool::new(false);

/// Enable raw input while retaining the exact prior Windows console mode.
///
/// A pre-existing Crossterm raw-mode session is borrowed, not claimed, so
/// Fresco's later restoration cannot disable application-owned raw mode.
pub(in crate::terminal::backend::lifecycle) fn enable_raw_mode() -> io::Result<()> {
    if !claim_snapshot() {
        return Ok(());
    }
    let handle = match stdin_handle() {
        Ok(handle) => handle,
        Err(error) => {
            release_unpublished_snapshot_claim();
            return Err(error);
        }
    };
    let crossterm_was_raw = match crossterm::terminal::is_raw_mode_enabled() {
        Ok(enabled) => enabled,
        Err(error) => {
            release_unpublished_snapshot_claim();
            return Err(error);
        }
    };
    let original_input = match console_mode(handle) {
        Ok(mode) => mode,
        Err(error) => {
            release_unpublished_snapshot_claim();
            return Err(error);
        }
    };
    let original_output = output_mode_snapshot();
    publish_snapshot(handle, original_input, original_output, !crossterm_was_raw);
    if !crossterm_was_raw && let Err(enable) = crossterm::terminal::enable_raw_mode() {
        return fail_enable_transition(enable);
    }
    Ok(())
}

/// Restore and release the native raw-mode snapshot.
pub(in crate::terminal::backend::lifecycle) fn disable_raw_mode() -> io::Result<()> {
    let Some(snapshot) = RawSnapshotReader::acquire() else {
        return Ok(());
    };
    let mut restoration = restore_snapshot_modes(snapshot.snapshot);
    if restoration.is_ok() && snapshot.snapshot.owns_crossterm_raw_mode {
        restoration = crossterm::terminal::disable_raw_mode();
    }
    drop(snapshot);
    if restoration.is_ok() {
        deactivate_snapshot();
    }
    restoration
}

/// Return whether raw mode owns a native snapshot requiring restoration.
pub(in crate::terminal::backend::lifecycle) fn raw_mode_requires_restoration() -> bool {
    RAW_SNAPSHOT_PUBLISHED.load(Ordering::SeqCst)
}

/// Restore raw mode from a panic or console-control handler without
/// releasing conservative state, allowing normal cleanup to retry.
pub(in crate::terminal::backend::lifecycle) fn emergency_restore_raw_mode() -> bool {
    let Some(snapshot) = RawSnapshotReader::acquire() else {
        return true;
    };
    restore_snapshot_modes(snapshot.snapshot).is_ok()
}

#[derive(Clone, Copy)]
struct RawSnapshot {
    input_handle: HANDLE,
    input_original: u32,
    output_handle: HANDLE,
    output_original: u32,
    has_output_mode: bool,
    owns_crossterm_raw_mode: bool,
}

struct RawSnapshotReader {
    snapshot: RawSnapshot,
}

impl RawSnapshotReader {
    fn acquire() -> Option<Self> {
        RAW_SNAPSHOT_READERS.fetch_add(1, Ordering::SeqCst);
        if !RAW_SNAPSHOT_PUBLISHED.load(Ordering::SeqCst) {
            RAW_SNAPSHOT_READERS.fetch_sub(1, Ordering::SeqCst);
            return None;
        }
        let input_handle = RAW_SNAPSHOT_HANDLE.load(Ordering::SeqCst) as HANDLE;
        if input_handle.is_null() || input_handle == INVALID_HANDLE_VALUE {
            RAW_SNAPSHOT_READERS.fetch_sub(1, Ordering::SeqCst);
            return None;
        }
        Some(Self {
            snapshot: RawSnapshot {
                input_handle,
                input_original: RAW_SNAPSHOT_MODE.load(Ordering::SeqCst),
                output_handle: RAW_SNAPSHOT_OUTPUT_HANDLE.load(Ordering::SeqCst) as HANDLE,
                output_original: RAW_SNAPSHOT_OUTPUT_MODE.load(Ordering::SeqCst),
                has_output_mode: RAW_SNAPSHOT_HAS_OUTPUT_MODE.load(Ordering::SeqCst),
                owns_crossterm_raw_mode: RAW_SNAPSHOT_OWNS_CROSSTERM.load(Ordering::SeqCst),
            },
        })
    }
}

impl Drop for RawSnapshotReader {
    fn drop(&mut self) {
        RAW_SNAPSHOT_READERS.fetch_sub(1, Ordering::SeqCst);
    }
}

fn publish_snapshot(
    handle: HANDLE,
    original: u32,
    output: Option<(HANDLE, u32)>,
    owns_crossterm_raw_mode: bool,
) {
    RAW_SNAPSHOT_HANDLE.store(handle as isize, Ordering::SeqCst);
    RAW_SNAPSHOT_MODE.store(original, Ordering::SeqCst);
    if let Some((output_handle, output_mode)) = output {
        RAW_SNAPSHOT_OUTPUT_HANDLE.store(output_handle as isize, Ordering::SeqCst);
        RAW_SNAPSHOT_OUTPUT_MODE.store(output_mode, Ordering::SeqCst);
        RAW_SNAPSHOT_HAS_OUTPUT_MODE.store(true, Ordering::SeqCst);
    } else {
        RAW_SNAPSHOT_OUTPUT_HANDLE.store(0, Ordering::SeqCst);
        RAW_SNAPSHOT_OUTPUT_MODE.store(0, Ordering::SeqCst);
        RAW_SNAPSHOT_HAS_OUTPUT_MODE.store(false, Ordering::SeqCst);
    }
    RAW_SNAPSHOT_OWNS_CROSSTERM.store(owns_crossterm_raw_mode, Ordering::SeqCst);
    RAW_SNAPSHOT_PUBLISHED.store(true, Ordering::SeqCst);
}

fn fail_enable_transition(enable: io::Error) -> io::Result<()> {
    let rollback = RawSnapshotReader::acquire()
        .map(|snapshot| restore_snapshot_modes(snapshot.snapshot))
        .unwrap_or(Ok(()));
    if rollback.is_ok() {
        deactivate_snapshot();
        return Err(enable);
    }
    Err(combine_transition_errors(enable, rollback.unwrap_err()))
}

fn deactivate_snapshot() {
    RAW_SNAPSHOT_PUBLISHED.store(false, Ordering::SeqCst);
    let mut spins = 0_u8;
    while RAW_SNAPSHOT_READERS.load(Ordering::SeqCst) != 0 {
        if spins < 64 {
            spins = spins.saturating_add(1);
            std::hint::spin_loop();
        } else {
            std::thread::yield_now();
        }
    }
    RAW_SNAPSHOT_ACTIVE.store(false, Ordering::SeqCst);
}

fn claim_snapshot() -> bool {
    RAW_SNAPSHOT_ACTIVE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}

fn release_unpublished_snapshot_claim() {
    RAW_SNAPSHOT_ACTIVE.store(false, Ordering::SeqCst);
}

fn restore_snapshot_modes(snapshot: RawSnapshot) -> io::Result<()> {
    let mut restoration = set_console_mode(snapshot.input_handle, snapshot.input_original);
    if snapshot.has_output_mode
        && let Err(error) = set_console_mode(snapshot.output_handle, snapshot.output_original)
        && restoration.is_ok()
    {
        restoration = Err(error);
    }
    restoration
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

fn output_mode_snapshot() -> Option<(HANDLE, u32)> {
    let handle = stdout_handle().ok()?;
    let mode = console_mode(handle).ok()?;
    Some((handle, mode))
}

fn stdout_handle() -> io::Result<HANDLE> {
    // SAFETY: `STD_OUTPUT_HANDLE` is the documented selector for the
    // process standard output handle.
    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
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
