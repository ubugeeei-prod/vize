//! Unix `termios` snapshots and lock-free emergency restoration.
//!
//! Terminal calls go through rustix's safe `termios` and descriptor wrappers,
//! which crossterm already links. The one `unsafe` contract left here is the
//! snapshot cell that a panic hook or signal handler reads without a lock.

use std::{
    cell::UnsafeCell,
    error::Error,
    fmt,
    fs::OpenOptions,
    io, mem,
    os::fd::OwnedFd,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use rustix::termios::{self, OptionalActions, Termios};

static RAW_SNAPSHOT: RawSnapshotCell = RawSnapshotCell::new();
static RAW_SNAPSHOT_ACTIVE: AtomicBool = AtomicBool::new(false);
static RAW_SNAPSHOT_PUBLISHED: AtomicBool = AtomicBool::new(false);
static RAW_SNAPSHOT_READERS: AtomicUsize = AtomicUsize::new(0);

/// The published snapshot. Only the claimant of `RAW_SNAPSHOT_ACTIVE` stores
/// or removes it, and only while it is unpublished and no reader holds a slot.
/// Readers borrow it only through [`RawSnapshotReader`].
struct RawSnapshotCell(UnsafeCell<Option<NativeRawSnapshot>>);

impl RawSnapshotCell {
    const fn new() -> Self {
        Self(UnsafeCell::new(None))
    }

    /// Store or remove the snapshot as its owner, returning the old value.
    fn replace(&self, snapshot: Option<NativeRawSnapshot>) -> Option<NativeRawSnapshot> {
        // SAFETY: only the `RAW_SNAPSHOT_ACTIVE` claimant calls this, so no two
        // calls overlap. It stores before `RAW_SNAPSHOT_PUBLISHED` is set and
        // removes after the flag is cleared and `RAW_SNAPSHOT_READERS` drained
        // to zero, so no reader borrow overlaps the mutation (see
        // `RawSnapshotReader::acquire` for the ordering argument).
        unsafe { mem::replace(&mut *self.0.get(), snapshot) }
    }
}

// SAFETY: every access to the `UnsafeCell` follows the publication and reader
// protocol documented on `RawSnapshotCell::replace` and
// `RawSnapshotReader::acquire`. `NativeRawSnapshot` is `Send + Sync` data.
unsafe impl Sync for RawSnapshotCell {}

struct NativeRawSnapshot {
    fd: OwnedFd,
    original: Termios,
    owns_crossterm_raw_mode: bool,
}

pub(super) struct RawSnapshotReader {
    snapshot: &'static NativeRawSnapshot,
}

impl RawSnapshotReader {
    pub(super) fn acquire() -> Option<Self> {
        RAW_SNAPSHOT_READERS.fetch_add(1, Ordering::SeqCst);
        if !RAW_SNAPSHOT_PUBLISHED.load(Ordering::SeqCst) {
            RAW_SNAPSHOT_READERS.fetch_sub(1, Ordering::SeqCst);
            return None;
        }
        // SAFETY: this reader took a slot and then observed the published
        // flag, both in the sequentially consistent order. The owner stores
        // the snapshot before publishing it, and removes it only after
        // clearing the flag and waiting for every slot, so the value stays in
        // place and unmutated until `Drop` releases this slot. The `'static`
        // borrow never leaves the guard: `snapshot()` lends it for the guard's
        // lifetime only.
        let snapshot = unsafe { (*RAW_SNAPSHOT.0.get()).as_ref() };
        match snapshot {
            Some(snapshot) => Some(Self { snapshot }),
            None => {
                RAW_SNAPSHOT_READERS.fetch_sub(1, Ordering::SeqCst);
                None
            }
        }
    }

    fn snapshot(&self) -> &NativeRawSnapshot {
        self.snapshot
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
            drop(fd);
            release_unpublished_snapshot_claim();
            return Err(error);
        }
    };
    let owns_crossterm_raw_mode = !crossterm_was_raw;
    publish_snapshot(fd, owns_crossterm_raw_mode)?;
    if owns_crossterm_raw_mode && let Err(enable) = crossterm::terminal::enable_raw_mode() {
        return fail_enable_transition(enable);
    }
    Ok(())
}

/// Restore and release the native raw-mode snapshot.
pub(in crate::terminal::backend::lifecycle) fn disable_raw_mode() -> io::Result<()> {
    let Some(reader) = RawSnapshotReader::acquire() else {
        return Ok(());
    };
    let snapshot = reader.snapshot();
    let mut restoration = restore_original_attributes(snapshot);
    if restoration.is_ok() && snapshot.owns_crossterm_raw_mode {
        restoration = crossterm::terminal::disable_raw_mode();
    }
    drop(reader);
    if restoration.is_ok() {
        deactivate_snapshot();
    }
    restoration
}

/// Return whether raw mode owns a native snapshot requiring restoration.
pub(in crate::terminal::backend::lifecycle) fn raw_mode_requires_restoration() -> bool {
    RAW_SNAPSHOT_PUBLISHED.load(Ordering::SeqCst)
}

/// Restore raw mode from a panic hook or signal handler without releasing
/// conservative state.
///
/// The reader slot is two atomic operations, and rustix's `tcsetattr` is one
/// `ioctl` on Linux and libc's `tcsetattr` elsewhere; POSIX lists both as
/// async-signal-safe. The path neither allocates nor takes a Fresco or
/// standard-output lock. State remains active so unwind cleanup can retry.
pub(in crate::terminal::backend::lifecycle) fn emergency_restore_raw_mode() -> bool {
    let Some(reader) = RawSnapshotReader::acquire() else {
        return true;
    };
    restore_original_attributes(reader.snapshot()).is_ok()
}

#[cfg(test)]
pub(super) fn enable_raw_mode_on_owned_fd(fd: OwnedFd) -> io::Result<()> {
    if !claim_snapshot() {
        drop(fd);
        return Ok(());
    }
    let mut raw = publish_snapshot(fd, false)?;
    raw.make_raw();

    let applied = match RawSnapshotReader::acquire() {
        Some(reader) => set_terminal_attributes(&reader.snapshot().fd, &raw),
        None => Ok(()),
    };
    if let Err(enable) = applied {
        return fail_enable_transition(enable);
    }
    Ok(())
}

/// Read the descriptor's attributes, then store and publish the snapshot.
/// Returns a copy of the captured attributes.
fn publish_snapshot(fd: OwnedFd, owns_crossterm_raw_mode: bool) -> io::Result<Termios> {
    let original = match termios::tcgetattr(&fd) {
        Ok(attributes) => attributes,
        Err(error) => {
            drop(fd);
            release_unpublished_snapshot_claim();
            return Err(error.into());
        }
    };
    drop(RAW_SNAPSHOT.replace(Some(NativeRawSnapshot {
        fd,
        original: original.clone(),
        owns_crossterm_raw_mode,
    })));
    // Sequential consistency closes the late-reader race: a reader that
    // increments after deactivation must observe unpublished, while a reader
    // that observes published is counted before the owner can reuse storage.
    RAW_SNAPSHOT_PUBLISHED.store(true, Ordering::SeqCst);
    Ok(original)
}

fn fail_enable_transition(enable: io::Error) -> io::Result<()> {
    let rollback = match RawSnapshotReader::acquire() {
        Some(reader) => restore_original_attributes(reader.snapshot()),
        None => Ok(()),
    };
    match rollback {
        Ok(()) => {
            deactivate_snapshot();
            Err(enable)
        }
        Err(rollback) => Err(combine_transition_errors(enable, rollback)),
    }
}

fn restore_original_attributes(snapshot: &NativeRawSnapshot) -> io::Result<()> {
    set_terminal_attributes(&snapshot.fd, &snapshot.original)
}

fn set_terminal_attributes(fd: &OwnedFd, attributes: &Termios) -> io::Result<()> {
    Ok(termios::tcsetattr(fd, OptionalActions::Now, attributes)?)
}

pub(super) fn open_controlling_terminal() -> io::Result<OwnedFd> {
    let stdin = rustix::stdio::stdin();
    if termios::isatty(stdin) {
        // `F_DUPFD_CLOEXEC` creates the duplicate close-on-exec atomically.
        return Ok(rustix::io::fcntl_dupfd_cloexec(stdin, 0)?);
    }
    // std opens with `O_CLOEXEC`, leaving no descriptor-inheritance window.
    let terminal = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    Ok(terminal.into())
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
    // Dropping the snapshot closes its descriptor exactly once. Like
    // `OwnedFd::drop`, close errors are not retried.
    drop(RAW_SNAPSHOT.replace(None));
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
