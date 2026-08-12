//! Process signal supervision for terminal presentation modes.

use std::{error::Error, fmt, io};

#[cfg(unix)]
mod unix;

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
    /// The current platform has no POSIX signal-action implementation.
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

/// Install restoration before existing handlers for interactive termination signals.
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
/// Non-Unix platforms return [`TerminalSignalHookError::UnsupportedPlatform`]
/// without changing process state.
pub fn install_terminal_signal_hook()
-> Result<TerminalSignalHookInstallation, TerminalSignalHookError> {
    #[cfg(not(unix))]
    {
        Err(TerminalSignalHookError::UnsupportedPlatform)
    }

    #[cfg(unix)]
    {
        unix::install()
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
    "unknown signal"
}

#[cfg(test)]
mod tests;
