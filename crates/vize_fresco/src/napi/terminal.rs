//! Terminal NAPI bindings.
//!
//! Note: `format!` is used throughout this module because napi `Error::new`
//! requires `std::string::String`.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

use crate::terminal::{Backend, TerminalCapabilities, TerminalOptions};

use super::{
    frame_output::FrameOutputTelemetryNapi,
    terminal_types::{TerminalInfoNapi, TerminalOptionsNapi},
};

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
    init_terminal_with_options(TerminalOptionsNapi {
        raw_mode: Some(true),
        alternate_screen: Some(true),
        mouse: Some(true),
        bracketed_paste: Some(true),
        hide_cursor: Some(true),
    })
}

/// Initialize terminal with explicit TUI mode options.
#[napi(js_name = "initTerminalWithOptions")]
#[allow(clippy::disallowed_macros)]
pub fn init_terminal_with_options(options: TerminalOptionsNapi) -> Result<()> {
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

    backend
        .init_with_options(TerminalOptions {
            raw_mode: options.raw_mode.unwrap_or(true),
            alternate_screen: options.alternate_screen.unwrap_or(false),
            mouse_capture: options.mouse.unwrap_or(false),
            bracketed_paste: options.bracketed_paste.unwrap_or(true),
            hide_cursor: options.hide_cursor.unwrap_or(true),
        })
        .map_err(|e| {
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

/// Resolve the complete standard-output capability profile.
///
/// Size discovery falls back to positive `COLUMNS` and `LINES` values, then
/// 80x24. The result therefore remains available for redirected output.
#[napi(js_name = "getTerminalInfo")]
#[allow(clippy::disallowed_macros)]
pub fn get_terminal_info() -> Result<TerminalInfoNapi> {
    Ok(terminal_info_from(TerminalCapabilities::detect_stdout()))
}

fn terminal_info_from(capabilities: TerminalCapabilities) -> TerminalInfoNapi {
    let color = capabilities.color();
    let unicode = capabilities.unicode();
    let interactive = capabilities.interactive();
    TerminalInfoNapi {
        width: i32::from(capabilities.width()),
        height: i32::from(capabilities.height()),
        colors: color.value().is_color(),
        true_color: color.value().is_true_color(),
        color_depth: color.value().as_str().to_owned(),
        color_reason: color.reason().as_str().to_owned(),
        unicode: unicode.value(),
        unicode_reason: unicode.reason().as_str().to_owned(),
        interactive: interactive.value(),
        interactive_reason: interactive.reason().as_str().to_owned(),
        redirected: capabilities.is_redirected(),
        narrow: capabilities.is_narrow(),
        narrow_width: i32::from(capabilities.narrow_width()),
    }
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

/// Flush the terminal buffer and return exact presentation telemetry.
#[napi(js_name = "flushTerminalMeasured")]
#[allow(clippy::disallowed_macros)]
pub fn flush_terminal_measured() -> Result<FrameOutputTelemetryNapi> {
    let mut guard = BACKEND
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Lock error: {}", e)))?;

    let Some(backend) = guard.as_mut() else {
        return Ok(crate::terminal::FrameOutputTelemetry::default().into());
    };
    backend
        .flush_measured()
        .map(Into::into)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to flush: {}", e)))
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

#[cfg(test)]
mod tests {
    use crate::terminal::{TerminalCapabilityProbe, TerminalProfileOptions};

    use super::*;

    #[test]
    fn terminal_info_exposes_the_complete_resolved_profile() {
        let capabilities = TerminalCapabilities::resolve(
            &TerminalCapabilityProbe::new(42, 18, false)
                .with_no_color(true)
                .with_locale("C"),
            TerminalProfileOptions::default(),
        );
        let info = terminal_info_from(capabilities);

        assert_eq!((info.width, info.height), (42, 18));
        assert!(!info.colors);
        assert!(!info.true_color);
        assert_eq!(info.color_depth, "monochrome");
        assert_eq!(info.color_reason, "no-color-environment");
        assert!(!info.unicode);
        assert_eq!(info.unicode_reason, "non-utf8-locale");
        assert!(!info.interactive);
        assert_eq!(info.interactive_reason, "redirected-output");
        assert!(info.redirected);
        assert!(info.narrow);
        assert_eq!(info.narrow_width, 60);
    }
}
