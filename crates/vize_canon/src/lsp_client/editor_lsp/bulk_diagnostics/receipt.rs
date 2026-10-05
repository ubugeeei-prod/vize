//! Test-only custody from the actual successful category requests.

use serde_json::Value;
use std::cell::RefCell;

std::thread_local! {
    static OBSERVED: RefCell<Option<Value>> = const { RefCell::new(None) };
}

pub(in crate::lsp_client) fn reset() {
    OBSERVED.with(|value| *value.borrow_mut() = None);
}

pub(super) fn record(receipt: Value) {
    OBSERVED.with(|value| *value.borrow_mut() = Some(receipt));
}

pub(super) fn finish(outcome: &super::BulkDiagnostics) {
    match outcome {
        super::BulkDiagnostics::Refused => reset(),
        super::BulkDiagnostics::Complete(_) => OBSERVED.with(|value| {
            if let Some(receipt) = value.borrow_mut().as_mut() {
                receipt["outcome"] = serde_json::json!("complete");
                receipt["release"] = serde_json::json!("acknowledged");
            }
        }),
        super::BulkDiagnostics::RetireOwner {
            request_error,
            release_error,
        } => OBSERVED.with(|value| {
            if let Some(receipt) = value.borrow_mut().as_mut() {
                receipt["outcome"] = serde_json::json!("retire-owner");
                receipt["requestError"] = request_error
                    .as_ref()
                    .map(|error| serde_json::json!(vize_l0::cstr!("{error:?}")))
                    .unwrap_or(Value::Null);
                receipt["releaseError"] = release_error
                    .as_ref()
                    .map(|error| serde_json::json!(vize_l0::cstr!("{error:?}")))
                    .unwrap_or(Value::Null);
            }
        }),
    }
}

pub(in crate::lsp_client::editor_lsp) fn take() -> Option<Value> {
    OBSERVED.with(|value| value.borrow_mut().take())
}

pub(in crate::lsp_client) fn fallback(
    cause: &str,
    retirement: Option<&vize_l0::String>,
    error: Option<&vize_l0::String>,
) {
    OBSERVED.with(|value| {
        let mut value = value.borrow_mut();
        let receipt = value.get_or_insert_with(|| serde_json::json!({}));
        receipt["cause"] = serde_json::json!(cause);
        receipt["ownerRetirementError"] = serde_json::json!(retirement);
        receipt["wholeFallbackError"] = serde_json::json!(error);
        receipt["outcome"] = serde_json::json!("whole-original-fallback");
    });
}
