//! Process panic supervision for terminal presentation modes.

use std::{error::Error, fmt};

#[cfg(unix)]
use std::{
    panic,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

#[cfg(any(unix, test))]
use super::TerminalMode;
#[cfg(unix)]
use super::lease::emergency_presentation_modes;
#[cfg(unix)]
use super::raw_mode::emergency_restore_raw_mode;

#[cfg(unix)]
static PANIC_HOOK_INSTALLATION: Mutex<()> = Mutex::new(());
#[cfg(unix)]
static PANIC_HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

#[cfg(any(unix, test))]
const DISABLE_MOUSE_CAPTURE: &[u8] = b"\x1b[?1006l\x1b[?1015l\x1b[?1003l\x1b[?1002l\x1b[?1000l";
#[cfg(any(unix, test))]
const DISABLE_BRACKETED_PASTE: &[u8] = b"\x1b[?2004l";
#[cfg(any(unix, test))]
const LEAVE_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049l";
#[cfg(any(unix, test))]
const RESET_CURSOR_SHAPE: &[u8] = b"\x1b[0 q";
#[cfg(any(unix, test))]
const SHOW_CURSOR: &[u8] = b"\x1b[?25h";

#[cfg(any(unix, test))]
pub(super) const PRESENTATION_RESETS: [(TerminalMode, &[u8]); 5] = [
    (TerminalMode::MouseCapture, DISABLE_MOUSE_CAPTURE),
    (TerminalMode::BracketedPaste, DISABLE_BRACKETED_PASTE),
    (TerminalMode::AlternateScreen, LEAVE_ALTERNATE_SCREEN),
    (TerminalMode::CursorShape, RESET_CURSOR_SHAPE),
    (TerminalMode::CursorVisibility, SHOW_CURSOR),
];

/// Result of installing Fresco's process panic hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminalPanicHookInstallation {
    /// Fresco installed its hook around the previously configured process hook.
    Installed,
    /// Fresco's hook was already installed, so process state was unchanged.
    AlreadyInstalled,
}

/// Reason Fresco could not install its process panic hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminalPanicHookError {
    /// The current platform lacks Fresco's native emergency-output path.
    UnsupportedPlatform,
    /// Rust forbids changing the process panic hook from a panicking thread.
    PanickingThread,
    /// A prior panic poisoned the installation lock.
    InstallationPoisoned,
}

impl fmt::Display for TerminalPanicHookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(
                formatter,
                "terminal panic restoration is not supported on {}",
                std::env::consts::OS
            ),
            Self::PanickingThread => formatter
                .write_str("the terminal panic hook cannot be installed from a panicking thread"),
            Self::InstallationPoisoned => {
                formatter.write_str("terminal panic hook installation was poisoned")
            }
        }
    }
}

impl Error for TerminalPanicHookError {}

/// Install process-wide restoration before Rust invokes the existing panic hook.
///
/// On Unix, Fresco restores every tracked ANSI presentation mode directly to
/// standard output, then restores the exact native terminal attributes captured
/// before raw mode, before delegating to the hook that was active at installation
/// time. This path does not allocate, format, acquire a standard-output lock, or
/// depend on [`Drop`], so it also runs before a `panic = "abort"` process exits.
///
/// Installation is process-global, thread-safe, and idempotent. The wrapper is
/// permanent because stable Rust cannot determine whether a later application
/// hook still chains it; hooks installed afterward must preserve Rust's usual
/// take-and-chain ownership discipline. A caught panic should be followed by
/// [`Backend::restore`](super::super::Backend::restore) before reusing the backend.
///
/// Non-Unix platforms return [`TerminalPanicHookError::UnsupportedPlatform`]
/// without changing the process hook. Their console restoration requires native
/// platform state rather than assuming ANSI escape-sequence support.
pub fn install_terminal_panic_hook() -> Result<TerminalPanicHookInstallation, TerminalPanicHookError>
{
    #[cfg(not(unix))]
    {
        Err(TerminalPanicHookError::UnsupportedPlatform)
    }

    #[cfg(unix)]
    {
        if thread::panicking() {
            return Err(TerminalPanicHookError::PanickingThread);
        }
        if PANIC_HOOK_INSTALLED.load(Ordering::Acquire) {
            return Ok(TerminalPanicHookInstallation::AlreadyInstalled);
        }

        let _installation = PANIC_HOOK_INSTALLATION
            .lock()
            .map_err(|_| TerminalPanicHookError::InstallationPoisoned)?;
        if PANIC_HOOK_INSTALLED.load(Ordering::Acquire) {
            return Ok(TerminalPanicHookInstallation::AlreadyInstalled);
        }

        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |information| {
            restore_owned_presentation_modes(
                emergency_presentation_modes(),
                emergency_write_stdout,
            );
            let _ = emergency_restore_raw_mode();
            previous(information);
        }));
        PANIC_HOOK_INSTALLED.store(true, Ordering::Release);
        Ok(TerminalPanicHookInstallation::Installed)
    }
}

#[inline]
#[cfg(any(unix, test))]
pub(super) fn restore_owned_presentation_modes(
    owned_modes: u8,
    mut write: impl FnMut(&[u8]) -> bool,
) {
    for (mode, reset) in PRESENTATION_RESETS {
        if owned_modes & mode.bit() != 0 {
            // Cleanup is best-effort: one rejected mode must not prevent the
            // remaining independent resets from reaching the terminal.
            let _ = write(reset);
        }
    }
}

#[cfg(unix)]
pub(super) fn emergency_write_stdout(mut bytes: &[u8]) -> bool {
    while !bytes.is_empty() {
        // SAFETY: `bytes` remains valid for the duration of the call, its exact
        // length is supplied, and `STDOUT_FILENO` is not borrowed or closed.
        // POSIX `write` avoids Rust's reentrant standard-output lock and heap.
        let written = unsafe {
            libc::write(
                libc::STDOUT_FILENO,
                bytes.as_ptr().cast::<libc::c_void>(),
                bytes.len(),
            )
        };
        if written <= 0 || written as usize > bytes.len() {
            return false;
        }
        bytes = &bytes[written as usize..];
    }
    true
}

#[cfg(test)]
mod tests;
