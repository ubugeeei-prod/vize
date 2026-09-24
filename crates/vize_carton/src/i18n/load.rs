//! The embedded flat-JSON message loader.

use rustc_hash::FxHashMap;

/// Parse JSON and load into message map
pub(super) fn load_json(map: &mut FxHashMap<&'static str, &'static str>, json: &'static str) {
    // Fast JSON parsing for flat key-value objects
    // Format: { "key": "value", "key2": "value2", ... }
    let json = json.trim();
    if json.len() < 2 || !json.starts_with('{') || !json.ends_with('}') {
        return;
    }

    let content = json.get(1..json.len() - 1).unwrap_or_default();
    let bytes = content.as_bytes();
    let at = |index: usize| bytes.get(index).copied();
    let mut idx = 0;

    while idx < content.len() {
        // Skip whitespace
        while at(idx).is_some_and(|byte| byte.is_ascii_whitespace()) {
            idx += 1;
        }

        if idx >= content.len() {
            break;
        }

        // Expect opening quote for key
        if at(idx) != Some(b'"') {
            idx += 1;
            continue;
        }
        idx += 1;

        // Parse key
        let key_start = idx;
        while let Some(byte) = at(idx) {
            match byte {
                b'"' => break,
                b'\\' => idx += 2,
                _ => idx += 1,
            }
        }
        let key_end = idx;
        idx += 1; // Skip closing quote

        // Skip to colon
        while at(idx).is_some_and(|byte| byte != b':') {
            idx += 1;
        }
        idx += 1; // Skip colon

        // Skip whitespace
        while at(idx).is_some_and(|byte| byte.is_ascii_whitespace()) {
            idx += 1;
        }

        // Expect opening quote for value
        if at(idx) != Some(b'"') {
            continue;
        }
        idx += 1;

        // Parse value (handle escaped quotes)
        let value_start = idx;
        while let Some(byte) = at(idx) {
            match byte {
                b'"' => break,
                b'\\' => idx += 2,
                _ => idx += 1,
            }
        }
        let value_end = idx;
        idx += 1; // Skip closing quote

        // Skip to comma or end
        while at(idx).is_some_and(|byte| byte != b',') {
            idx += 1;
        }
        idx += 1; // Skip comma

        // Extract key and value; an escape at the very end leaves no slice.
        let (Some(key), Some(value)) = (
            content.get(key_start..key_end),
            content.get(value_start..value_end),
        ) else {
            continue;
        };

        // Unescape and leak to get 'static lifetime
        // This is safe because we only load once at startup
        let key: &'static str = Box::leak(key.to_string().into_boxed_str());
        let value: &'static str = Box::leak(unescape_json_string(value).into_boxed_str());

        map.insert(key, value);
    }
}

/// Unescape JSON string escape sequences
#[inline]
pub(super) fn unescape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('u') => {
                    // Unicode escape: \uXXXX
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(cp) = u32::from_str_radix(&hex, 16)
                        && let Some(c) = char::from_u32(cp)
                    {
                        result.push(c);
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}
