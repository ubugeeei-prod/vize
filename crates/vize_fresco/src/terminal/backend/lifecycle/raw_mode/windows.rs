//! Windows console-mode snapshots and lock-free emergency restoration.

use std::{
    error::Error,
    fmt, io, ptr,
    sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicUsize, Ordering},
};

use windows_sys::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE},
    System::Console::{GetConsoleMode, GetStdHandle, STD_INPUT_HANDLE, SetConsoleMode},
};

static RAW_SNAPSHOT_ACTIVE: AtomicBool = AtomicBool::new(false);
static RAW_SNAPSHOT_PUBLISHED: AtomicBool = AtomicBool::new(false);
static RAW_SNAPSHOT_READERS: AtomicUsize = AtomicUsize::new(0);
static RAW_SNAPSHOT_HANDLE: AtomicIsize = AtomicIsize::new(0);
static RAW_SNAPSHOT_MODE: AtomicU32 = AtomicU32::new(0);
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
    publish_snapshot(handle, original_input, !crossterm_was_raw);
    if !crossterm_was_raw && let Err(enable) = crossterm::terminal::enable_raw_mode() {
        return fail_enable_transition(enable);
    }
    Ok(())
}

/// Restore and release the native raw-mode snapshot.
///
/// Crossterm cleanup runs before the snapshot is replayed because its Windows
/// implementation re-enables `ENABLE_LINE_INPUT`, `ENABLE_ECHO_INPUT`, and
/// `ENABLE_PROCESSED_INPUT` instead of writing back the mode it observed, which
/// would clobber a valid non-default input mode. The captured mode is written
/// back even when that cleanup fails, so a partial teardown never leaves the
/// console on Crossterm's cooked-input defaults.
pub(in crate::terminal::backend::lifecycle) fn disable_raw_mode() -> io::Result<()> {
    let Some(snapshot) = RawSnapshotReader::acquire() else {
        return Ok(());
    };
    let cleanup = if snapshot.snapshot.owns_crossterm_raw_mode {
        crossterm::terminal::disable_raw_mode()
    } else {
        Ok(())
    };
    let restoration = set_console_mode(
        snapshot.snapshot.input_handle,
        snapshot.snapshot.input_original,
    );
    drop(snapshot);
    let outcome = match (cleanup, restoration) {
        (Ok(()), restoration) => restoration,
        (Err(cleanup), Ok(())) => Err(cleanup),
        (Err(cleanup), Err(restoration)) => Err(combine_transition_errors(
            "disabling Crossterm raw mode",
            cleanup,
            restoration,
        )),
    };
    if outcome.is_ok() {
        deactivate_snapshot();
    }
    outcome
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
    set_console_mode(
        snapshot.snapshot.input_handle,
        snapshot.snapshot.input_original,
    )
    .is_ok()
}

#[derive(Clone, Copy)]
struct RawSnapshot {
    input_handle: HANDLE,
    input_original: u32,
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

fn publish_snapshot(handle: HANDLE, original: u32, owns_crossterm_raw_mode: bool) {
    RAW_SNAPSHOT_HANDLE.store(handle as isize, Ordering::SeqCst);
    RAW_SNAPSHOT_MODE.store(original, Ordering::SeqCst);
    RAW_SNAPSHOT_OWNS_CROSSTERM.store(owns_crossterm_raw_mode, Ordering::SeqCst);
    RAW_SNAPSHOT_PUBLISHED.store(true, Ordering::SeqCst);
}

fn fail_enable_transition(enable: io::Error) -> io::Result<()> {
    let rollback = RawSnapshotReader::acquire()
        .map(|snapshot| {
            set_console_mode(
                snapshot.snapshot.input_handle,
                snapshot.snapshot.input_original,
            )
        })
        .unwrap_or(Ok(()));
    if rollback.is_ok() {
        deactivate_snapshot();
        return Err(enable);
    }
    Err(combine_transition_errors(
        "enabling raw mode",
        enable,
        rollback.unwrap_err(),
    ))
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

fn combine_transition_errors(
    action: &'static str,
    action_error: io::Error,
    restoration: io::Error,
) -> io::Error {
    let kind = action_error.kind();
    io::Error::new(
        kind,
        RawModeTransitionFailure {
            action,
            action_error,
            restoration,
        },
    )
}

#[derive(Debug)]
struct RawModeTransitionFailure {
    action: &'static str,
    action_error: io::Error,
    restoration: io::Error,
}

impl fmt::Display for RawModeTransitionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} failed: {}; restoring the original console mode also failed: {}",
            self.action, self.action_error, self.restoration
        )
    }
}

impl Error for RawModeTransitionFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.action_error)
    }
}
