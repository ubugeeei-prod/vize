//! Native raw-mode ownership and lock-free emergency restoration.

#[cfg(not(any(unix, windows)))]
pub(super) use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

/// Return whether a failed raw-mode transition may still require restoration.
#[cfg(not(any(unix, windows)))]
pub(super) const fn raw_mode_requires_restoration() -> bool {
    false
}

#[cfg(unix)]
mod unix {
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
    static RAW_SNAPSHOT_READERS: AtomicUsize = AtomicUsize::new(0);

    struct RawSnapshotCell(UnsafeCell<MaybeUninit<NativeRawSnapshot>>);

    impl RawSnapshotCell {
        const fn new() -> Self {
            Self(UnsafeCell::new(MaybeUninit::uninit()))
        }

        fn write(&self, snapshot: NativeRawSnapshot) {
            // SAFETY: only the process-terminal lease owner writes, and it
            // publishes `RAW_SNAPSHOT_ACTIVE` after this write. A new owner
            // cannot write until deactivation has observed zero readers.
            unsafe { (*self.0.get()).write(snapshot) };
        }

        fn read(&self) -> NativeRawSnapshot {
            // SAFETY: callers hold a reader slot and observed the active flag
            // in the sequentially consistent publication order. Deactivation
            // waits for every slot before a later session can reuse storage.
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
            if !RAW_SNAPSHOT_ACTIVE.load(Ordering::SeqCst) {
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
        if RAW_SNAPSHOT_ACTIVE.load(Ordering::SeqCst) {
            return Ok(());
        }
        let fd = open_controlling_terminal()?;
        let crossterm_was_raw = match crossterm::terminal::is_raw_mode_enabled() {
            Ok(enabled) => enabled,
            Err(error) => {
                close_owned_fd(fd);
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
        RAW_SNAPSHOT_ACTIVE.load(Ordering::SeqCst)
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
                return Err(error);
            }
        };
        RAW_SNAPSHOT.write(NativeRawSnapshot {
            fd,
            original,
            owns_crossterm_raw_mode,
        });
        // Sequential consistency closes the late-reader race: a reader that
        // increments after deactivation must observe inactive, while a reader
        // that observes active is counted before the owner can reuse storage.
        RAW_SNAPSHOT_ACTIVE.store(true, Ordering::SeqCst);
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

        // SAFETY: the path is a static NUL-terminated C string and `open`
        // receives no variadic mode because `O_CREAT` is absent.
        let fd = unsafe { libc::open(c"/dev/tty".as_ptr(), libc::O_RDWR) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        set_cloexec_or_close(fd)
    }

    fn duplicate_cloexec(fd: RawFd) -> io::Result<RawFd> {
        // SAFETY: duplicating a valid descriptor does not borrow its resource.
        let duplicate = unsafe { libc::dup(fd) };
        if duplicate < 0 {
            return Err(io::Error::last_os_error());
        }
        set_cloexec_or_close(duplicate)
    }

    fn set_cloexec_or_close(fd: RawFd) -> io::Result<RawFd> {
        // SAFETY: `fd` is owned by the caller and `F_SETFD` takes one integer.
        if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } == 0 {
            return Ok(fd);
        }
        let error = io::Error::last_os_error();
        close_owned_fd(fd);
        Err(error)
    }

    fn deactivate_snapshot() {
        RAW_SNAPSHOT_ACTIVE.store(false, Ordering::SeqCst);
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
}

#[cfg(unix)]
pub(super) use unix::{
    disable_raw_mode, emergency_restore_raw_mode, enable_raw_mode, raw_mode_requires_restoration,
};

#[cfg(windows)]
mod windows {
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
}

#[cfg(windows)]
pub(super) use windows::{
    disable_raw_mode, emergency_restore_raw_mode, enable_raw_mode, raw_mode_requires_restoration,
};

#[cfg(all(test, unix))]
mod tests;
