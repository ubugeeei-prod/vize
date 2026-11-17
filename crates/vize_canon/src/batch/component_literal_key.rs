//! Authentication for component props-literal keys shared by checker paths.

use std::ops::Range;

use crate::virtual_ts::{ProjectionFeatures, VizeMapping};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct AuthenticatedComponentLiteralKey {
    pub(super) authored: Range<usize>,
    pub(super) generated: Range<usize>,
}

/// Authenticate the canonical, editor-owned component props-literal key.
///
/// Inference-only copies fail the `ALL` feature gate. The generated row must
/// start with a quoted property key and carry an exact sub-span from that key
/// to an authored attribute name inside a component start tag (never a
/// `<slot>` outlet or a `<template>`).
pub(super) fn authenticate_component_literal_key(
    source: &str,
    generated: &str,
    mapping: &VizeMapping,
    features: ProjectionFeatures,
    authored_base: usize,
) -> Option<AuthenticatedComponentLiteralKey> {
    if features != ProjectionFeatures::ALL {
        return None;
    }
    let entry = generated.get(mapping.gen_range.clone())?;
    let key_length = quoted_key_length(entry)?;
    let generated = mapping.gen_range.start..mapping.gen_range.start + key_length;
    let authored = mapping
        .sub_spans
        .iter()
        .find(|sub_span| sub_span.gen_range == generated)
        .map(|sub_span| {
            sub_span.src_range.start + authored_base..sub_span.src_range.end + authored_base
        })?;
    is_authored_attribute_key(source, authored.clone()).then_some(
        AuthenticatedComponentLiteralKey {
            authored,
            generated,
        },
    )
}

/// Length of an emitted quoted property key, including both quotes.
fn quoted_key_length(entry: &str) -> Option<usize> {
    let bytes = entry.as_bytes();
    if bytes.first() != Some(&b'"') {
        return None;
    }
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' => escaped = true,
            b'"' if entry
                .get(index + 1..)
                .is_some_and(|tail| tail.starts_with(": ")) =>
            {
                return Some(index + 1);
            }
            _ => {}
        }
    }
    None
}

fn is_authored_attribute_key(source: &str, key: Range<usize>) -> bool {
    let Some(token) = source.get(key.clone()) else {
        return false;
    };
    if token.is_empty()
        || token.bytes().any(|byte| {
            byte.is_ascii_whitespace() || matches!(byte, b'"' | b'\'' | b'<' | b'>' | b'=')
        })
    {
        return false;
    }
    let Some(before) = source.get(..key.start) else {
        return false;
    };
    let Some(tag_start) = before.rfind('<') else {
        return false;
    };
    if before.rfind('>').is_some_and(|end| end > tag_start) {
        return false;
    }
    let Some(tag_head) = source.get(tag_start + 1..key.start) else {
        return false;
    };
    if tag_head.starts_with(['/', '!', '?']) || !ends_outside_quotes(tag_head) {
        return false;
    }
    // Vue's compiler decides these by tag name: `<slot>` is a slot outlet
    // whose payload literal has the same shape as a props literal (#4979),
    // and `<template>` is a fragment. Neither passes component props.
    let tag_name = tag_head
        .split(|byte: char| byte.is_ascii_whitespace() || byte == '/')
        .next()
        .unwrap_or_default();
    if matches!(tag_name, "slot" | "template") {
        return false;
    }
    source.get(key.end..).is_some_and(|after| {
        after.is_empty() || after.starts_with(['=', '.', ' ', '\t', '\r', '\n', '/', '>'])
    })
}

fn ends_outside_quotes(tag_head: &str) -> bool {
    let mut quote = None;
    for byte in tag_head.bytes() {
        match quote {
            Some(open) if byte == open => quote = None,
            Some(_) => {}
            None if matches!(byte, b'"' | b'\'') => quote = Some(byte),
            None => {}
        }
    }
    quote.is_none()
}
