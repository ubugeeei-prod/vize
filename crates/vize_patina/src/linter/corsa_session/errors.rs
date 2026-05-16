use std::path::Path;
use vize_carton::{String, ToCompactString};

pub(super) fn io_error_message(prefix: &str, path: &Path, error: &std::io::Error) -> String {
    let mut message = prefix.to_compact_string();
    message.push_str(": ");
    message.push_str(path.to_string_lossy().as_ref());
    message.push_str(": ");
    let detail = error.to_compact_string();
    message.push_str(detail.as_str());
    message
}

pub(super) fn compact_error(prefix: &str, detail: &str) -> String {
    let mut message = prefix.to_compact_string();
    if !detail.is_empty() {
        message.push_str(": ");
        message.push_str(detail);
    }
    message
}
