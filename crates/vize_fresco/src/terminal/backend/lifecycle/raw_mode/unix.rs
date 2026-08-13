//! Unix `termios` snapshots and lock-free emergency restoration.

use std::{
    cell::UnsafeCell,
    error::Error,
    fmt, io,
    mem::MaybeUninit,
    os::fd::RawFd,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

static RAW_SNAPSHOT: RawSnapshotCell = RawSnapshotCell::new();
static RAW_SNAPSHOT_ACTIVE: AtomicBool = AtomicBool::new(false);
static RAW_SNAPSHOT_PUBLISHED: AtomicBool = AtomicBool::new(false);
static RAW_SNAPSHOT_READERS: AtomicUsize = AtomicUsize::new(0);

struct RawSnapshotCell(UnsafeCell<MaybeUninit<NativeRawSnapshot>>);

impl RawSnapshotCell {
    const fn new() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    fn write(&self, snapshot: NativeRawSnapshot) {
        // SAFETY: only the snapshot claimant writes, and it publishes after
        // this write. A new claimant waits until readers drain.
        unsafe { (*self.0.get()).write(snapshot) };
    }

    fn read(&self) -> NativeRawSnapshot {
        // SAFETY: the value is initialized because every caller reads only
        // after a `write` published `RAW_SNAPSHOT_ACTIVE`. Reader callers
        // hold a reader slot and observed the active flag in the
        // sequentially consistent publication order, and deactivation waits
        // for every slot before a later session can reuse storage. The
        // remaining caller is `deactivate_snapshot`, which is the lease
        // owner reading its own snapshot after readers drained, so no
        // concurrent `write` can overlap it. `NativeRawSnapshot` is `Copy`
        // data (a descriptor, a `termios`, and a flag), so reading it out
        // never duplicates ownership of a Rust resource; the descriptor is
        // closed exactly once by `deactivate_snapshot`.
        unsafe { self.0.get().cast::<NativeRawSnapshot>().read() }
    }
}

// SAFETY: access to the `UnsafeCell` follows the publication and reader
// protocol documented on `RawSnapshotCell::write` and `read`.
unsafe impl Sync for RawSnapshotCell {}

struct NativeRawSnapshot {
    fd: RawFd,
    original: libc::termios,
    owns_crossterm_raw_mode: bool,
}

pub(super) struct RawSnapshotReader {
    snapshot: NativeRawSnapshot,
}

impl RawSnapshotReader {
    pub(super) fn acquire() -> Option<Self> {
        RAW_SNAPSHOT_READERS.fetch_add(1, Ordering::SeqCst);
        if !RAW_SNAPSHOT_PUBLISHED.load(Ordering::SeqCst) {
            RAW_SNAPSHOT_READERS.fetch_sub(1, Ordering::SeqCst);
            return None;
        }
        Some(Self {
            snapshot: RAW_SNAPSHOT.read(),
        })
    }
}

impl Drop for RawSnapshotReader {
    fn drop(&mut self) {
        RAW_SNAPSHOT_READERS.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Enable raw input while retaining the exact prior terminal attributes.
///
/// A pre-existing Crossterm raw-mode session is borrowed, not claimed, so
/// Fresco's later restoration cannot disable application-owned raw mode.
pub(in crate::terminal::backend::lifecycle) fn enable_raw_mode() -> io::Result<()> {
    if !claim_snapshot() {
        return Ok(());
    }
    let fd = match open_controlling_terminal() {
        Ok(fd) => fd,
        Err(error) => {
            release_unpublished_snapshot_claim();
            return Err(error);
        }
    };
    let crossterm_was_raw = match crossterm::terminal::is_raw_mode_enabled() {
        Ok(enabled) => enabled,
        Err(error) => {
            close_owned_fd(fd);
            release_unpublished_snapshot_claim();
            return Err(error);
        }
    };
    let owns_crossterm_raw_mode = !crossterm_was_raw;
    let original = publish_snapshot(fd, owns_crossterm_raw_mode)?;
    if owns_crossterm_raw_mode && let Err(enable) = crossterm::terminal::enable_raw_mode() {
        return fail_enable_transition(fd, &original, enable);
    }
    Ok(())
}

/// Restore and release the native raw-mode snapshot.
pub(in crate::terminal::backend::lifecycle) fn disable_raw_mode() -> io::Result<()> {
    let Some(snapshot) = RawSnapshotReader::acquire() else {
        return Ok(());
    };
    let mut restoration =
        set_terminal_attributes(snapshot.snapshot.fd, &snapshot.snapshot.original);
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

/// Restore raw mode from a panic hook without releasing conservative state.
pub(in crate::terminal::backend::lifecycle) fn emergency_restore_raw_mode() -> bool {
    let Some(snapshot) = RawSnapshotReader::acquire() else {
        return true;
    };
    // `tcsetattr` is a direct libc operation: it does not allocate or take
    // Fresco/stdout locks. State remains active so unwind cleanup can retry.
    unsafe {
        libc::tcsetattr(
            snapshot.snapshot.fd,
            libc::TCSANOW,
            &snapshot.snapshot.original,
        ) == 0
    }
}

#[cfg(test)]
pub(super) fn enable_raw_mode_on_owned_fd(fd: RawFd) -> io::Result<()> {
    if !claim_snapshot() {
        close_owned_fd(fd);
        return Ok(());
    }
    let original = publish_snapshot(fd, false)?;
    let mut raw = original;
    // SAFETY: `raw` is an initialized termios value owned by this function.
    unsafe { libc::cfmakeraw(&mut raw) };

    if let Err(enable) = set_terminal_attributes(fd, &raw) {
        return fail_enable_transition(fd, &original, enable);
    }
    Ok(())
}

fn publish_snapshot(fd: RawFd, owns_crossterm_raw_mode: bool) -> io::Result<libc::termios> {
    let original = match terminal_attributes(fd) {
        Ok(attributes) => attributes,
        Err(error) => {
            close_owned_fd(fd);
            release_unpublished_snapshot_claim();
            return Err(error);
        }
    };
    RAW_SNAPSHOT.write(NativeRawSnapshot {
        fd,
        original,
        owns_crossterm_raw_mode,
    });
    // Sequential consistency closes the late-reader race: a reader that
    // increments after deactivation must observe unpublished, while a reader
    // that observes published is counted before the owner can reuse storage.
    RAW_SNAPSHOT_PUBLISHED.store(true, Ordering::SeqCst);
    Ok(original)
}

fn fail_enable_transition(
    fd: RawFd,
    original: &libc::termios,
    enable: io::Error,
) -> io::Result<()> {
    let rollback = set_terminal_attributes(fd, original);
    if rollback.is_ok() {
        deactivate_snapshot();
        return Err(enable);
    }
    Err(combine_transition_errors(enable, rollback.unwrap_err()))
}

pub(super) fn terminal_attributes(fd: RawFd) -> io::Result<libc::termios> {
    let mut attributes = MaybeUninit::<libc::termios>::zeroed();
    // SAFETY: `attributes` points to writable storage for one termios value.
    if unsafe { libc::tcgetattr(fd, attributes.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful `tcgetattr` initialized the complete zeroed value.
    Ok(unsafe { attributes.assume_init() })
}

fn set_terminal_attributes(fd: RawFd, attributes: &libc::termios) -> io::Result<()> {
    // SAFETY: `attributes` is initialized and borrowed for the syscall.
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, attributes) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

fn open_controlling_terminal() -> io::Result<RawFd> {
    // SAFETY: both operations use valid process file-descriptor constants.
    if unsafe { libc::isatty(libc::STDIN_FILENO) } == 1 {
        return duplicate_cloexec(libc::STDIN_FILENO);
    }

    // SAFETY: the path is static and NUL-terminated. `O_CLOEXEC` sets the
    // flag atomically, leaving no descriptor-inheritance window.
    let fd = unsafe { libc::open(c"/dev/tty".as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(fd)
}

fn duplicate_cloexec(fd: RawFd) -> io::Result<RawFd> {
    // SAFETY: duplicating a valid descriptor does not borrow its resource,
    // and `F_DUPFD_CLOEXEC` takes one integer minimum-descriptor argument.
    // The duplicate is created close-on-exec atomically.
    let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) };
    if duplicate < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(duplicate)
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
    close_owned_fd(RAW_SNAPSHOT.read().fd);
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

fn close_owned_fd(fd: RawFd) {
    // SAFETY: the snapshot owns `fd`; close is attempted exactly once after
    // readers drain. Like `OwnedFd::drop`, close errors are not retried.
    let _ = unsafe { libc::close(fd) };
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
            "enabling raw mode failed: {}; restoring the original terminal attributes also failed: {}",
            self.enable, self.rollback
        )
    }
}

impl Error for RawModeTransitionFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.enable)
    }
}
