//! Unix `sigaction` installation and async-signal-safe dispatch.
//!
//! The `unsafe` left here is the `sigaction` query and install, the handler
//! entry and its chaining to the previous action's C ABI, and the slot table
//! the handler reads without a lock. Errno is saved and restored through the
//! `errno` crate's safe accessors, which signal-hook-registry already links.

use std::{
    io,
    mem::{self, MaybeUninit},
    ptr,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering, compiler_fence},
    },
};

mod previous_actions;
use previous_actions::{PreviousAction, PreviousActions};

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

pub(super) fn install() -> Result<TerminalSignalHookInstallation, TerminalSignalHookError> {
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

    PREVIOUS_ACTIONS.publish(&previous);
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
    // `previous` holds one saved action per supervised signal.
    for (index, (signal, saved)) in SUPERVISED_SIGNALS.into_iter().zip(previous).enumerate() {
        let wrapped = wrapped_action(saved);
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
    let installed_pairs = SUPERVISED_SIGNALS.into_iter().zip(previous).take(installed);
    for (signal, saved) in installed_pairs.rev() {
        if let Err(error) = replace(signal, saved) {
            failures.push(TerminalSignalRollbackFailure { signal, error });
        }
    }
    failures
}

pub(super) fn inspect_action(
    signal: libc::c_int,
) -> Result<libc::sigaction, TerminalSignalHookError> {
    let mut action = MaybeUninit::<libc::sigaction>::zeroed();
    // SAFETY: POSIX `sigaction` with a null `act` changes nothing and, on
    // success, stores the current action in `*oact`, here the live local
    // `action`. Whether or not the call succeeds, every byte of `action` is
    // initialized: it starts zeroed, and zero is a valid `sigaction` (a null
    // handler word, an empty mask, no flags and, where present, a `None`
    // restorer).
    let (status, action) = unsafe {
        (
            libc::sigaction(signal, ptr::null(), action.as_mut_ptr()),
            action.assume_init(),
        )
    };
    if status != 0 {
        return Err(TerminalSignalHookError::InspectAction {
            signal,
            source: io::Error::last_os_error(),
        });
    }
    Ok(action)
}

fn wrapped_action(previous: &libc::sigaction) -> libc::sigaction {
    // `sigaction` is a `Copy` C value. Copying retains its mask and
    // platform-only fields.
    let mut wrapped = *previous;
    wrapped.sa_sigaction = terminal_signal_handler as *const () as usize;
    // `sa_flags` differs in signedness and width on supported libc targets.
    #[expect(
        unused_assignments,
        reason = "the dead initializer infers sa_flags' platform integer type"
    )]
    let mut siginfo = wrapped.sa_flags;
    siginfo = libc::SA_SIGINFO as _;
    wrapped.sa_flags |= siginfo;
    wrapped
}

/// Install `action` for `signal`. Async-signal-safe: one `sigaction` call, and
/// `io::Error::last_os_error` stores only the error code.
fn replace_action(signal: libc::c_int, action: &libc::sigaction) -> io::Result<()> {
    // SAFETY: POSIX `sigaction` reads one complete action from `act`, which
    // borrows an initialized `sigaction` for the call, and a null `oact`
    // discards the replaced action. Every action this module installs is
    // either one `sigaction` reported for the same signal or that action with
    // the handler word set to `terminal_signal_handler` and `SA_SIGINFO` added,
    // which matches that handler's three-argument signature.
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
    // `errno::errno` and `errno::set_errno` read and write this thread's errno
    // slot through libc's accessor; neither allocates nor locks.
    let saved_errno = errno::errno();
    compiler_fence(Ordering::Acquire);
    let previous = PREVIOUS_ACTIONS.read(index);

    restore_owned_presentation_modes(emergency_presentation_modes(), emergency_write_stdout);
    let _ = emergency_restore_raw_mode();

    // Restoration must be invisible to the chained action. That action retains
    // its own errno discipline, exactly as it would under direct invocation.
    errno::set_errno(saved_errno);

    if let Some(previous) = previous {
        // SAFETY: `previous` came from `sigaction` for this exact signal (the
        // slot table only stores actions `inspect_action` reported for the
        // signal at that index). The caller is the kernel, so `information`
        // and `context` are the pointers it passed for this delivery, and
        // dispatch hands them on unchanged.
        unsafe { chain_previous_action(signal, information, context, previous) };
    }
}

unsafe fn chain_previous_action(
    signal: libc::c_int,
    information: *mut libc::siginfo_t,
    context: *mut libc::c_void,
    previous: PreviousAction,
) {
    let (handler, siginfo) = match previous {
        PreviousAction::Ignore => return,
        PreviousAction::Default(previous) => {
            // Reinstate the default disposition, then re-raise. The signal becomes
            // pending when the current action masks itself and takes effect
            // immediately otherwise. `sigaction`, `raise` and `abort` are POSIX
            // async-signal-safe, and `std::process::abort` is libc's `abort`.
            // SAFETY: POSIX `raise` takes the signal number by value and
            // dereferences no pointer.
            if replace_action(signal, &previous).is_err() || unsafe { libc::raise(signal) } != 0 {
                // Termination is guaranteed if the exact default disposition could
                // not be restored.
                std::process::abort();
            }
            return;
        }
        PreviousAction::Simple(handler) => (handler, false),
        PreviousAction::Siginfo(handler) => (handler, true),
    };

    let handler = handler as *mut ();
    if !siginfo {
        // SAFETY: POSIX: without `SA_SIGINFO` the handler word is a
        // `void (*)(int)` handler, and it is neither `SIG_DFL` nor `SIG_IGN`
        // here, so it is a function with the one-argument C ABI.
        let action = unsafe { mem::transmute::<*mut (), extern "C" fn(libc::c_int)>(handler) };
        action(signal);
    } else {
        type SignalAction = extern "C" fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void);
        // SAFETY: POSIX: with `SA_SIGINFO` the handler word is a
        // `void (*)(int, siginfo_t *, void *)` handler with the three-argument
        // C ABI, called with the kernel's pointers.
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
