//! Input NAPI bindings.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::types::{ImeStateNapi, InputEventNapi};
use crate::input;

/// Poll for input events.
///
/// Returns an event if available within the timeout, or null if no event.
#[napi(js_name = "pollEvent")]
pub fn poll_event(timeout_ms: u32) -> Result<Option<InputEventNapi>> {
    let event = input::poll(timeout_ms as u64)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Poll error: {}", e)))?;

    Ok(event.map(InputEventNapi::from))
}

/// Poll for input events without blocking.
#[napi(js_name = "pollEventNonBlocking")]
pub fn poll_event_non_blocking() -> Result<Option<InputEventNapi>> {
    let event = input::poll_nonblocking()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Poll error: {}", e)))?;

    Ok(event.map(InputEventNapi::from))
}

/// Read an input event, blocking until one is available.
#[napi(js_name = "readEvent")]
pub fn read_event() -> Result<InputEventNapi> {
    let event = input::read_event()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Read error: {}", e)))?;

    Ok(InputEventNapi::from(event))
}

/// Get current IME state.
#[napi(js_name = "getImeState")]
pub fn get_ime_state() -> Result<ImeStateNapi> {
    // For now, return a default inactive state
    // In a full implementation, this would query the platform IME
    Ok(ImeStateNapi {
        active: false,
        mode: "direct".to_string(),
        composing: false,
        preedit: None,
        preedit_cursor: None,
        candidates: None,
        selected: None,
    })
}

/// Enable IME.
#[napi(js_name = "enableIme")]
pub fn enable_ime() -> Result<bool> {
    // Terminal IME is always "enabled" in the sense that
    // we can receive composed text via paste events
    Ok(true)
}

/// Disable IME.
#[napi(js_name = "disableIme")]
pub fn disable_ime() -> Result<bool> {
    Ok(true)
}

/// Set IME input mode.
#[napi(js_name = "setImeMode")]
pub fn set_ime_mode(mode: String) -> Result<()> {
    // In a terminal context, we can't directly control the system IME
    // This is a placeholder for future platform-specific implementations
    let _ = mode;
    Ok(())
}
