//! Unix `sigaction` installation and async-signal-safe dispatch.

mod errno;

use std::{
    cell::UnsafeCell,
    io,
    mem::{self, MaybeUninit},
    ptr,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering, compiler_fence},
    },
};

use super::{
    TerminalSignalHookError, TerminalSignalHookInstallation, TerminalSignalRollbackFailure,
};
use crate::terminal::backend::lifecycle::{
    lease::emergency_presentation_modes,
    panic_hook::{emergency_write_stdout, restore_owned_presentation_modes},
    raw_mode::emergency_restore_raw_mode,
};

pub(super) const SUPERVISED_SIGNALS: [libc::c_int; 4] =
    [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGQUIT];

static SIGNAL_HOOK_INSTALLATION: Mutex<()> = Mutex::new(());
static SIGNAL_HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);
static SIGNAL_HOOK_UNCERTAIN: AtomicBool = AtomicBool::new(false);
static PREVIOUS_ACTIONS: PreviousActions = PreviousActions::new();

pub(super) unsafe fn errno_location() -> *mut libc::c_int {
    // SAFETY: the platform-specific function has the same contract.
    unsafe { errno::location() }
}

pub(super) fn install() -> Result<TerminalSignalHookInstallation, TerminalSignalHookError> {
    if !errno::SUPPORTED {
        return Err(TerminalSignalHookError::UnsupportedPlatform);
    }
    if SIGNAL_HOOK_INSTALLED.load(Ordering::Acquire) {
        return Ok(TerminalSignalHookInstallation::AlreadyInstalled);
    }
    if SIGNAL_HOOK_UNCERTAIN.load(Ordering::Acquire) {
        return Err(TerminalSignalHookError::InstallationStateUncertain);
    }

    let _installation = SIGNAL_HOOK_INSTALLATION
        .lock()
        .map_err(|_| TerminalSignalHookError::InstallationPoisoned)?;
    if SIGNAL_HOOK_INSTALLED.load(Ordering::Acquire) {
        return Ok(TerminalSignalHookInstallation::AlreadyInstalled);
    }
    if SIGNAL_HOOK_UNCERTAIN.load(Ordering::Acquire) {
        return Err(TerminalSignalHookError::InstallationStateUncertain);
    }

    install_actions()
}

fn install_actions() -> Result<TerminalSignalHookInstallation, TerminalSignalHookError> {
    let mut previous = Vec::with_capacity(SUPERVISED_SIGNALS.len());
    for signal in SUPERVISED_SIGNALS {
        previous.push(inspect_action(signal)?);
    }

    for (index, action) in previous.iter().enumerate() {
        PREVIOUS_ACTIONS.write(index, action);
    }
    // The kernel cannot invoke our handler until a later `sigaction` call.
    // Prevent compiler motion across that publication boundary.
    compiler_fence(Ordering::Release);

    if let Err(failure) = install_actions_with(&previous, replace_action) {
        let InstallAttemptFailure {
            signal,
            source,
            rollback_failures,
        } = failure;
        if !rollback_failures.is_empty() {
            SIGNAL_HOOK_UNCERTAIN.store(true, Ordering::Release);
        }
        return Err(TerminalSignalHookError::InstallAction {
            signal,
            source,
            rollback_failures,
        });
    }

    SIGNAL_HOOK_INSTALLED.store(true, Ordering::Release);
    Ok(TerminalSignalHookInstallation::Installed)
}

pub(super) fn install_actions_with(
    previous: &[libc::sigaction],
    mut replace: impl FnMut(libc::c_int, &libc::sigaction) -> io::Result<()>,
) -> Result<(), InstallAttemptFailure> {
    for (index, signal) in SUPERVISED_SIGNALS.into_iter().enumerate() {
        let wrapped = wrapped_action(&previous[index]);
        if let Err(source) = replace(signal, &wrapped) {
            let rollback_failures = rollback_actions_with(previous, index, &mut replace);
            return Err(InstallAttemptFailure {
                signal,
                source,
                rollback_failures,
            });
        }
    }
    Ok(())
}

pub(super) struct InstallAttemptFailure {
    pub(super) signal: libc::c_int,
    pub(super) source: io::Error,
    pub(super) rollback_failures: Vec<TerminalSignalRollbackFailure>,
}

fn rollback_actions_with(
    previous: &[libc::sigaction],
    installed: usize,
    replace: &mut impl FnMut(libc::c_int, &libc::sigaction) -> io::Result<()>,
) -> Vec<TerminalSignalRollbackFailure> {
    let mut failures = Vec::new();
    for index in (0..installed).rev() {
        let signal = SUPERVISED_SIGNALS[index];
        if let Err(error) = replace(signal, &previous[index]) {
            failures.push(TerminalSignalRollbackFailure { signal, error });
        }
    }
    failures
}

pub(super) fn inspect_action(
    signal: libc::c_int,
) -> Result<libc::sigaction, TerminalSignalHookError> {
    let mut action = MaybeUninit::<libc::sigaction>::zeroed();
    // SAFETY: a null replacement only queries the action and `action` points to
    // writable storage for the complete result.
    if unsafe { libc::sigaction(signal, ptr::null(), action.as_mut_ptr()) } != 0 {
        return Err(TerminalSignalHookError::InspectAction {
            signal,
            source: io::Error::last_os_error(),
        });
    }
    // SAFETY: successful `sigaction` initialized the output action.
    Ok(unsafe { action.assume_init() })
}

fn wrapped_action(previous: &libc::sigaction) -> libc::sigaction {
    // SAFETY: `sigaction` is a plain C value whose bytes were initialized by
    // the operating system. Copying retains its mask and platform-only fields.
    let mut wrapped = unsafe { ptr::read(previous) };
    wrapped.sa_sigaction = terminal_signal_handler as *const () as usize;
    // `sa_flags` differs in signedness and width on supported libc targets.
    #[allow(unused_assignments)]
    let mut siginfo = wrapped.sa_flags;
    siginfo = libc::SA_SIGINFO as _;
    wrapped.sa_flags |= siginfo;
    wrapped
}

fn replace_action(signal: libc::c_int, action: &libc::sigaction) -> io::Result<()> {
    // SAFETY: `action` is initialized for the current platform; a null output
    // pointer intentionally discards the action replaced by this call.
    if unsafe { libc::sigaction(signal, action, ptr::null_mut()) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub(super) unsafe extern "C" fn terminal_signal_handler(
    signal: libc::c_int,
    information: *mut libc::siginfo_t,
    context: *mut libc::c_void,
) {
    let Some(index) = signal_index(signal) else {
        return;
    };
    // SAFETY: installation rejects targets without a known thread-local errno
    // accessor, and each supported libc returns a live slot for this thread.
    let errno = unsafe { errno_location() };
    // SAFETY: `errno` is non-null on every platform accepted at installation.
    let saved_errno = unsafe { *errno };
    compiler_fence(Ordering::Acquire);
    // SAFETY: the previous slot is initialized before this signal's wrapper is
    // installed and is never subsequently mutated.
    let previous = unsafe { PREVIOUS_ACTIONS.read(index) };

    restore_owned_presentation_modes(emergency_presentation_modes(), emergency_write_stdout);
    let _ = emergency_restore_raw_mode();

    // Restoration must be invisible to the chained action. That action retains
    // its own errno discipline, exactly as it would under direct invocation.
    // SAFETY: `errno` remains this thread's live errno slot.
    unsafe { *errno = saved_errno };

    // SAFETY: `previous` came from `sigaction` for this exact signal. Dispatch
    // uses its original ABI flag and the kernel-provided pointers unchanged.
    unsafe { chain_previous_action(signal, information, context, previous) };
}

unsafe fn chain_previous_action(
    signal: libc::c_int,
    information: *mut libc::siginfo_t,
    context: *mut libc::c_void,
    previous: &libc::sigaction,
) {
    let handler = previous.sa_sigaction;
    if handler == libc::SIG_IGN {
        return;
    }
    if handler == libc::SIG_DFL {
        // SAFETY: the saved action belongs to `signal`; both `sigaction` and
        // `raise` are async-signal-safe. The signal becomes pending when the
        // current action masks itself and takes effect immediately otherwise.
        if unsafe { libc::sigaction(signal, previous, ptr::null_mut()) } != 0
            || unsafe { libc::raise(signal) } != 0
        {
            // SAFETY: `abort` is async-signal-safe and guarantees termination
            // if the exact default disposition could not be restored.
            unsafe { libc::abort() };
        }
        return;
    }

    let handler = handler as *mut ();
    #[allow(unused_assignments)]
    let mut siginfo = previous.sa_flags;
    siginfo = libc::SA_SIGINFO as _;
    if previous.sa_flags & siginfo == 0 {
        // SAFETY: absence of `SA_SIGINFO` declares the one-argument C ABI.
        let action = unsafe { mem::transmute::<*mut (), extern "C" fn(libc::c_int)>(handler) };
        action(signal);
    } else {
        type SignalAction = extern "C" fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void);
        // SAFETY: `SA_SIGINFO` declares the three-argument C ABI.
        let action = unsafe { mem::transmute::<*mut (), SignalAction>(handler) };
        action(signal, information, context);
    }
}

const fn signal_index(signal: libc::c_int) -> Option<usize> {
    if signal == libc::SIGINT {
        Some(0)
    } else if signal == libc::SIGTERM {
        Some(1)
    } else if signal == libc::SIGHUP {
        Some(2)
    } else if signal == libc::SIGQUIT {
        Some(3)
    } else {
        None
    }
}

struct PreviousActions(UnsafeCell<[MaybeUninit<libc::sigaction>; SUPERVISED_SIGNALS.len()]>);

impl PreviousActions {
    const fn new() -> Self {
        Self(UnsafeCell::new(
            [const { MaybeUninit::uninit() }; SUPERVISED_SIGNALS.len()],
        ))
    }

    fn write(&self, index: usize, action: &libc::sigaction) {
        // SAFETY: process-global installation is serialized, the slot is not
        // published to a handler until after this write, and the source is an
        // initialized C action that does not own Rust resources.
        unsafe { (*self.0.get())[index].write(ptr::read(action)) };
    }

    unsafe fn read(&self, index: usize) -> &'static libc::sigaction {
        // SAFETY: the caller established that this signal's slot was published
        // before the corresponding handler could run. Slots are never moved or
        // mutated after publication and the static storage lives forever.
        unsafe { (*self.0.get())[index].assume_init_ref() }
    }
}

// SAFETY: installation writes are serialized and finish before a matching
// kernel handler can read; published slots are immutable for process lifetime.
unsafe impl Sync for PreviousActions {}
