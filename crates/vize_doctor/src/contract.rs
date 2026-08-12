//! Shared validation for stable public contract identifiers.

pub(crate) fn is_stable_id(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
        && !value.ends_with(['.', '-'])
        && !value.contains("..")
        && !value.contains("--")
}
