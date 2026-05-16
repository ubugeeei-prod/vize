//! Terminal NAPI bindings.
//!
//! Note: `format!` is used throughout this module because napi `Error::new`
//! requires `std::string::String`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

use crate::terminal::Backend;

use super::types::TerminalInfoNapi;

// Global terminal backend (lazy initialized)
static BACKEND: Mutex<Option<Backend>> = Mutex::new(None);

/// Initialize terminal for TUI mode.
#[napi(js_name = "initTerminal")]
#[allow(clippy::disallowed_macros)]
pub fn init_terminal() -> Result<()> {
    let mut guard = BACKEND
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Lock error: {}", e)))?;

    if guard.is_some() {
        return Err(Error::new(
            Status::GenericFailure,
            "Terminal already initialized",
        ));
    }

    let mut backend = Backend::new().map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to create backend: {}", e),
        )
    })?;

    backend.init().map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to init terminal: {}", e),
        )
    })?;

    *guard = Some(backend);
    Ok(())
}

/// Initialize terminal with mouse capture.
#[napi(js_name = "initTerminalWithMouse")]
#[allow(clippy::disallowed_macros)]
pub fn init_terminal_with_mouse() -> Result<()> {
    let mut guard = BACKEND
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Lock error: {}", e)))?;

    if guard.is_some() {
        return Err(Error::new(
            Status::GenericFailure,
            "Terminal already initialized",
        ));
    }

    let mut backend = Backend::new().map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to create backend: {}", e),
        )
    })?;

    backend.init_with_mouse().map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to init terminal: {}", e),
        )
    })?;

    *guard = Some(backend);
    Ok(())
}

/// Restore terminal to normal mode.
#[napi(js_name = "restoreTerminal")]
#[allow(clippy::disallowed_macros)]
pub fn restore_terminal() -> Result<()> {
    let mut guard = BACKEND
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Lock error: {}", e)))?;

    if let Some(ref mut backend) = *guard {
        backend.restore().map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to restore terminal: {}", e),
            )
        })?;
    }

    *guard = None;
    Ok(())
}

/// Get terminal info.
#[napi(js_name = "getTerminalInfo")]
#[allow(clippy::disallowed_macros)]
pub fn get_terminal_info() -> Result<TerminalInfoNapi> {
    let (width, height) = crossterm::terminal::size()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get size: {}", e)))?;

    Ok(TerminalInfoNapi {
        width: width as i32,
        height: height as i32,
        colors: true, // Assume colors are supported
        true_color: std::env::var("COLORTERM")
            .map(|v| v == "truecolor" || v == "24bit")
            .unwrap_or(false),
    })
}

/// Clear the screen.
#[napi(js_name = "clearScreen")]
#[allow(clippy::disallowed_macros)]
pub fn clear_screen() -> Result<()> {
    let mut guard = BACKEND
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Lock error: {}", e)))?;

    if let Some(ref mut backend) = *guard {
        backend
            .clear()
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to clear: {}", e)))?;
    }

    Ok(())
}

/// Flush the terminal buffer.
#[napi(js_name = "flushTerminal")]
#[allow(clippy::disallowed_macros)]
pub fn flush_terminal() -> Result<()> {
    let mut guard = BACKEND
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Lock error: {}", e)))?;

    if let Some(ref mut backend) = *guard {
        backend
            .flush()
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to flush: {}", e)))?;
    }

    Ok(())
}

/// Sync terminal size (call after resize events).
#[napi(js_name = "syncTerminalSize")]
#[allow(clippy::disallowed_macros)]
pub fn sync_terminal_size() -> Result<bool> {
    let mut guard = BACKEND
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Lock error: {}", e)))?;

    if let Some(ref mut backend) = *guard {
        let changed = backend.sync_size().map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to sync size: {}", e),
            )
        })?;
        Ok(changed)
    } else {
        Ok(false)
    }
}

/// Get access to backend (internal use).
#[allow(clippy::disallowed_macros)]
pub(crate) fn with_backend<T, F: FnOnce(&mut Backend) -> T>(f: F) -> Result<T> {
    let mut guard = BACKEND
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Lock error: {}", e)))?;

    if let Some(ref mut backend) = *guard {
        Ok(f(backend))
    } else {
        Err(Error::new(
            Status::GenericFailure,
            "Terminal not initialized",
        ))
    }
}
