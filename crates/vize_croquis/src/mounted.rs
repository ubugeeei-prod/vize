//! Mount-resource notes for `onMounted` callbacks.
//!
//! The call walker does not scan hook bodies. That scan pushed croquis
//! analyze over its instruction ceilings. Cross-file registration runs this
//! after analysis, outside that measurement.

use crate::{Croquis, ScopeData};

/// Mark `onMounted` calls whose callback acquires a listener, timer, socket,
/// or observer. The marker is an empty browser-global name so
/// `ClientOnlyScopeData` stays the published shape.
pub fn note_mounted_resources(source: &str, mut analysis: Croquis) -> Croquis {
    let offsets: Vec<u32> = analysis
        .scopes
        .iter()
        .filter_map(|scope| {
            let ScopeData::ClientOnly(data) = scope.data() else {
                return None;
            };
            if data.hook_name.as_str() != "onMounted" {
                return None;
            }
            callback_acquires_resource(source, scope.span.start, scope.span.end)
                .then_some(scope.span.start)
        })
        .collect();
    for offset in offsets {
        analysis.setup_context.note_mounted_resource(offset);
    }
    analysis
}

fn callback_acquires_resource(source: &str, start: u32, end: u32) -> bool {
    let Some(text) = source.get(start as usize..end as usize) else {
        return false;
    };
    const NAMES: &[&str] = &[
        "addEventListener",
        "setInterval",
        "setTimeout",
        "requestAnimationFrame",
        "subscribe",
    ];
    NAMES.iter().any(|name| contains_ident(text, name))
        || text.contains("new WebSocket")
        || text.contains("new EventSource")
        || text.contains("new Worker")
        || text.contains("new IntersectionObserver")
        || text.contains("new ResizeObserver")
        || text.contains("new MutationObserver")
        || text.contains(".observe(")
}

fn contains_ident(haystack: &str, ident: &str) -> bool {
    let bytes = haystack.as_bytes();
    let needle = ident.as_bytes();
    if needle.is_empty() {
        return false;
    }
    let mut start = 0;
    while start + needle.len() <= bytes.len() {
        if bytes.get(start..start + needle.len()) == Some(needle) {
            let before_ok = start == 0
                || bytes
                    .get(start - 1)
                    .is_some_and(|byte| !is_ident_byte(*byte));
            let after = start + needle.len();
            let after_ok = bytes.get(after).is_none_or(|byte| !is_ident_byte(*byte));
            if before_ok && after_ok {
                return true;
            }
        }
        start += 1;
    }
    false
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}
