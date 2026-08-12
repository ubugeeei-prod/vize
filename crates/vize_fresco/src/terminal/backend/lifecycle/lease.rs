//! Exclusive ownership of the process terminal.

use std::{
    error::Error,
    fmt,
    io::{self, Write},
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};

use super::super::{Backend, TerminalOptions};

static PROCESS_TERMINAL_OWNED: AtomicBool = AtomicBool::new(false);
static PROCESS_TERMINAL_MODES: AtomicU8 = AtomicU8::new(0);

/// Reason a backend could not acquire terminal-session ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminalSessionAcquireError {
    /// Another process-terminal backend still owns terminal presentation.
    ProcessTerminalAlreadyOwned,
    /// An injected writer requested process-global raw mode.
    RawModeRequiresProcessTerminal,
}

impl TerminalSessionAcquireError {
    const fn kind(self) -> io::ErrorKind {
        match self {
            Self::ProcessTerminalAlreadyOwned => io::ErrorKind::AlreadyExists,
            Self::RawModeRequiresProcessTerminal => io::ErrorKind::InvalidInput,
        }
    }

    fn into_io_error(self) -> io::Error {
        io::Error::new(self.kind(), self)
    }
}

impl fmt::Display for TerminalSessionAcquireError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessTerminalAlreadyOwned => formatter.write_str(
                "the process terminal is already owned by another active Fresco backend",
            ),
            Self::RawModeRequiresProcessTerminal => formatter.write_str(
                "raw mode is process-global and cannot be enabled for an injected writer",
            ),
        }
    }
}

impl Error for TerminalSessionAcquireError {}

impl<W: Write> Backend<W> {
    /// Return whether this backend holds the exclusive process-terminal lease.
    ///
    /// Injected writers always return `false`: their output and lifecycle are
    /// isolated, and raw mode is rejected instead of mutating process state.
    pub const fn holds_process_terminal_lease(&self) -> bool {
        self.process_lease
    }

    pub(super) fn prepare_session(&mut self, options: TerminalOptions) -> io::Result<()> {
        if !self.process_terminal {
            return if options.raw_mode {
                Err(TerminalSessionAcquireError::RawModeRequiresProcessTerminal.into_io_error())
            } else {
                Ok(())
            };
        }

        if options.requests_terminal_control() {
            self.acquire_process_lease()?;
        }
        Ok(())
    }

    #[inline]
    pub(in crate::terminal::backend) fn acquire_process_lease(&mut self) -> io::Result<()> {
        if !self.process_terminal || self.process_lease {
            return Ok(());
        }

        PROCESS_TERMINAL_OWNED
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| {
                TerminalSessionAcquireError::ProcessTerminalAlreadyOwned.into_io_error()
            })?;
        PROCESS_TERMINAL_MODES.store(0, Ordering::Release);
        self.process_lease = true;
        Ok(())
    }

    pub(in crate::terminal::backend) fn publish_process_session_state(&self) {
        if self.process_lease {
            PROCESS_TERMINAL_MODES.store(self.session.bits(), Ordering::Release);
        }
    }

    pub(super) fn release_process_lease_if_inactive(&mut self) {
        if !self.process_lease || !self.session.is_inactive() {
            return;
        }
        PROCESS_TERMINAL_MODES.store(0, Ordering::Release);
        self.process_lease = false;
        PROCESS_TERMINAL_OWNED.store(false, Ordering::Release);
    }
}

pub(super) fn emergency_presentation_modes() -> u8 {
    PROCESS_TERMINAL_MODES.load(Ordering::Acquire)
}
