//! Final dispatch completion for the exact authenticated native provider.

use corsa::runtime::BroadcastReceiver;
use corsa_lsp::{CorsaError, jsonrpc::InboundEvent};
use serde_json::Value;
use std::{
    sync::mpsc::RecvTimeoutError,
    time::{Duration, Instant},
};

// The pinned SDK's existing process-shutdown bound; this does not extend it.
const TIMEOUT: Duration = Duration::from_secs(2);
const PREFIX: &str = "handled method 'shutdown' (";

pub(super) fn drain(events: &BroadcastReceiver<InboundEvent>) -> corsa_lsp::Result<()> {
    let deadline = Instant::now() + TIMEOUT;
    vize_l0::profile!("canon.editor_lsp.shutdown_completion", {
        loop {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return Err(CorsaError::timeout("native shutdown completion", TIMEOUT));
            };
            let event = events
                .recv_timeout(remaining)
                .map_err(|error| match error {
                    RecvTimeoutError::Timeout => {
                        CorsaError::timeout("native shutdown completion", TIMEOUT)
                    }
                    RecvTimeoutError::Disconnected => {
                        CorsaError::Closed("native shutdown completion")
                    }
                })?;
            if let InboundEvent::Notification { method, params } = event
                && method == "window/logMessage"
                && completion(&params)?
            {
                return Ok(());
            }
        }
    })
}

fn completion(params: &Value) -> corsa_lsp::Result<bool> {
    let Some(message) = params.get("message").and_then(Value::as_str) else {
        return Ok(false);
    };
    // Other methods, levels and server messages remain unrelated events.
    if !message.starts_with("handled method 'shutdown'")
        || params.get("type").and_then(Value::as_u64) != Some(3)
    {
        return Ok(false);
    }
    let valid = params.as_object().is_some_and(|object| object.len() == 2)
        && message
            .strip_prefix(PREFIX)
            .and_then(|suffix| suffix.split_once(") in "))
            .is_some_and(|(id, elapsed)| {
                !id.is_empty()
                    && !id.starts_with('0')
                    && id.bytes().all(|byte| byte.is_ascii_digit())
                    && id.parse::<i64>().is_ok_and(|id| id > 0)
                    && go_duration(elapsed)
            });
    if !valid {
        return Err(CorsaError::Protocol(
            vize_l0::cstr!("Malformed authenticated native shutdown completion: {params}")
                .as_str()
                .into(),
        ));
    }
    Ok(true)
}

fn go_duration(mut value: &str) -> bool {
    if value == "0s" {
        return true;
    }
    let mut previous_rank = 7;
    let mut components = 0;
    while !value.is_empty() {
        let digits = value.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return false;
        }
        let Some(rest) = value.get(digits..) else {
            return false;
        };
        let number_end = if let Some(fraction) = rest.strip_prefix('.') {
            let fractional_digits = fraction.bytes().take_while(u8::is_ascii_digit).count();
            if fractional_digits == 0 {
                return false;
            }
            digits + 1 + fractional_digits
        } else {
            digits
        };
        let Some(after_number) = value.get(number_end..) else {
            return false;
        };
        let Some((unit, rank)) = [
            ("h", 6),
            ("m", 5),
            ("s", 4),
            ("ms", 3),
            ("µs", 2),
            ("ns", 1),
        ]
        .into_iter()
        .filter(|(_, rank)| *rank < previous_rank)
        .find(|(unit, _)| {
            after_number.starts_with(unit) && (*unit != "m" || !after_number.starts_with("ms"))
        }) else {
            return false;
        };
        let Some(rest) = after_number.strip_prefix(unit) else {
            return false;
        };
        value = rest;
        previous_rank = rank;
        components += 1;
    }
    components > 0
}

#[cfg(test)]
mod tests;
