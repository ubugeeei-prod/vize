//! Process signal supervision for terminal presentation modes.

use std::{error::Error, fmt, io};

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(all(test, unix))]
use unix::{
    SUPERVISED_SIGNALS, errno_location, inspect_action, install_actions_with,
    terminal_signal_handler,
};

/// Result of installing Fresco's process signal hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminalSignalHookInstallation {
    /// Fresco installed restoration around the existing process actions.
    Installed,
    /// Fresco's signal hook was already installed, so process state was unchanged.
    AlreadyInstalled,
}

/// One existing signal action that could not be restored after a partial install.
#[derive(Debug)]
pub struct TerminalSignalRollbackFailure {
    signal: i32,
    error: io::Error,
}

impl TerminalSignalRollbackFailure {
    /// Return the signal number whose previous action was not restored.
    pub const fn signal(&self) -> i32 {
        self.signal
    }

    /// Return the operating-system failure reported by `sigaction`.
    pub const fn error(&self) -> &io::Error {
        &self.error
    }
}

impl fmt::Display for TerminalSignalRollbackFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", signal_name(self.signal), self.error)
    }
}

impl Error for TerminalSignalRollbackFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

/// Reason Fresco could not install process signal restoration.
#[derive(Debug)]
#[non_exhaustive]
pub enum TerminalSignalHookError {
    /// The current platform has no native termination-handler implementation.
    UnsupportedPlatform,
    /// A prior panic poisoned the process-global installation lock.
    InstallationPoisoned,
    /// A failed rollback left at least one process signal action uncertain.
    InstallationStateUncertain,
    /// Fresco could not inspect the action already registered for a signal.
    InspectAction {
        /// Signal whose existing action could not be read.
        signal: i32,
        /// Operating-system error returned by `sigaction`.
        source: io::Error,
    },
    /// Fresco could not install its wrapper and attempted transactional rollback.
    InstallAction {
        /// Signal whose wrapper could not be installed.
        signal: i32,
        /// Operating-system error returned by `sigaction`.
        source: io::Error,
        /// Previous actions that could not be restored during rollback.
        rollback_failures: Vec<TerminalSignalRollbackFailure>,
    },
}

impl fmt::Display for TerminalSignalHookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(
                formatter,
                "terminal signal restoration is not supported on {}",
                std::env::consts::OS
            ),
            Self::InstallationPoisoned => {
                formatter.write_str("terminal signal hook installation was poisoned")
            }
            Self::InstallationStateUncertain => formatter.write_str(
                "terminal signal hook installation state is uncertain after a failed rollback",
            ),
            Self::InspectAction { signal, source } => write!(
                formatter,
                "cannot inspect the existing {} action: {source}",
                signal_name(*signal)
            ),
            Self::InstallAction {
                signal,
                source,
                rollback_failures,
            } => {
                write!(
                    formatter,
                    "cannot install terminal restoration for {}: {source}",
                    signal_name(*signal)
                )?;
                if !rollback_failures.is_empty() {
                    formatter.write_str("; previous actions also failed to restore for ")?;
                    for (index, failure) in rollback_failures.iter().enumerate() {
                        if index > 0 {
                            formatter.write_str(", ")?;
                        }
                        write!(formatter, "{failure}")?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl Error for TerminalSignalHookError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InspectAction { source, .. } | Self::InstallAction { source, .. } => Some(source),
            Self::UnsupportedPlatform
            | Self::InstallationPoisoned
            | Self::InstallationStateUncertain => None,
        }
    }
}

/// Install restoration before existing handlers for interactive termination events.
///
/// On Unix, Fresco supervises `SIGINT`, `SIGTERM`, `SIGHUP`, and `SIGQUIT`. Its
/// handler restores every presentation mode owned by the active process-terminal
/// backend and the exact terminal attributes captured before raw mode, then
/// delegates to the action that was active at installation time. One-argument
/// handlers, `SA_SIGINFO` handlers, ignored actions, default termination, signal
/// masks, and behavioral flags are retained. Default termination is re-raised
/// after restoration so the process still reports the original signal.
///
/// The handler calls only POSIX async-signal-safe operations (`write`,
/// `tcsetattr`, `sigaction`, and `raise`) plus lock-free atomics and fixed-memory
/// operations. It performs no allocation, locking, formatting, or buffered I/O.
/// Installation itself is process-global, thread-safe, idempotent, and
/// transactional: a partial failure restores every wrapper installed by that
/// attempt and reports all rollback failures.
///
/// Like Rust panic hooks, POSIX signal actions are process-global. Fresco's
/// wrapper is permanent; an action installed afterward must preserve the usual
/// query-and-chain discipline. Applications should install this hook during
/// single-threaded startup, before unrelated threads mutate signal actions.
///
/// On Windows, Fresco supervises console control events for Ctrl+C, Ctrl+Break,
/// console close, logoff, and shutdown. The handler restores the same
/// presentation and raw-input state, then returns `FALSE` so Windows can invoke
/// the next application handler or the default disposition. Windows does not
/// expose the existing handler chain for inspection, so Fresco cannot report or
/// restore prior handlers during installation; later handlers must still follow
/// Windows console-control chaining rules.
///
/// Windows delivery of these events is narrower than the Unix signal set, so
/// the console path supervises rather than guarantees restoration. Windows
/// delivers `CTRL_LOGOFF_EVENT` and `CTRL_SHUTDOWN_EVENT` only to services, and
/// it stops delivering them at all once `user32.dll` or `gdi32.dll` is loaded
/// into the process, so a typical console application receives neither. While
/// raw mode is active, Ctrl+C is delivered to the application as an input
/// record instead of a control event, because raw mode clears
/// `ENABLE_PROCESSED_INPUT`; that is the intended raw-mode contract, and
/// restoration for that path runs through the application's normal shutdown or
/// the panic hook. `CTRL_CLOSE_EVENT` also caps handler execution at a short
/// system timeout before the process is terminated.
///
/// Platforms without a native hook return
/// [`TerminalSignalHookError::UnsupportedPlatform`] without changing process
/// state.
pub fn install_terminal_signal_hook()
-> Result<TerminalSignalHookInstallation, TerminalSignalHookError> {
    #[cfg(not(any(unix, windows)))]
    {
        Err(TerminalSignalHookError::UnsupportedPlatform)
    }

    #[cfg(unix)]
    {
        unix::install()
    }

    #[cfg(windows)]
    {
        windows::install()
    }
}

fn signal_name(signal: i32) -> &'static str {
    #[cfg(unix)]
    {
        if signal == libc::SIGINT {
            return "SIGINT";
        }
        if signal == libc::SIGTERM {
            return "SIGTERM";
        }
        if signal == libc::SIGHUP {
            return "SIGHUP";
        }
        if signal == libc::SIGQUIT {
            return "SIGQUIT";
        }
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::{
            CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT,
            CTRL_SHUTDOWN_EVENT,
        };
        if signal == CTRL_C_EVENT as i32 {
            return "CTRL_C_EVENT";
        }
        if signal == CTRL_BREAK_EVENT as i32 {
            return "CTRL_BREAK_EVENT";
        }
        if signal == CTRL_CLOSE_EVENT as i32 {
            return "CTRL_CLOSE_EVENT";
        }
        if signal == CTRL_LOGOFF_EVENT as i32 {
            return "CTRL_LOGOFF_EVENT";
        }
        if signal == CTRL_SHUTDOWN_EVENT as i32 {
            return "CTRL_SHUTDOWN_EVENT";
        }
    }
    "unknown signal"
}

#[cfg(test)]
mod tests;
