//! Template-side static `ref="…"` scan for `script/prefer-use-template-ref`.
//!
//! Patina's template and script passes are separate, so a script rule that has
//! to know which names the template binds as template refs reads the raw
//! `<template>` source, the same cross-block trade-off
//! [`crate::rules::script::props_emits::template_emits`] makes for emit calls.
//!
//! Direction of error matters here, and it is the opposite of the emits scan: a
//! recorded name *enables* a report rather than suppressing one. That is still
//! safe relative to the rule's previous behaviour, which reported every
//! `ref(null)` declaration unconditionally — over-matching can therefore only
//! leave a pre-existing report in place, never invent one at a new location.
//! Under-matching (an exotic attribute spelling we do not recognize) drops a
//! report that upstream `vue/prefer-use-template-ref` would make.
//!
//! Only *static* `ref` attributes are collected. `:ref` / `v-bind:ref` bind an
//! expression rather than a name, and upstream skips them for the same reason.

use vize_s0::{CompactString, FxHashSet};

/// Collect the value of every static `ref="name"` attribute in `template`.
pub(super) fn collect_template_ref_names(template: &str) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();
    let bytes = template.as_bytes();
    for (index, _) in template.match_indices("ref") {
        // `ref` has to stand alone as an attribute name: the preceding byte may
        // not extend the name (`myref`, `data-ref`) and may not turn it into a
        // bound or namespaced attribute (`:ref`, `v-bind:ref`, `.ref`).
        let before = index.checked_sub(1).map(|i| bytes[i]);
        if before.is_some_and(|byte| is_attribute_name_byte(byte) || byte == b':' || byte == b'.') {
            continue;
        }
        if let Some(name) = attribute_value(template, index + "ref".len()) {
            names.insert(CompactString::new(name));
        }
    }
    names
}

/// Parse `= "value"` / `= 'value'` / `=value` starting at byte `from`,
/// returning the attribute value.
///
/// A value containing a backslash is skipped rather than unescaped, so a
/// mangled name is never recorded.
fn attribute_value(template: &str, from: usize) -> Option<&str> {
    let bytes = template.as_bytes();
    let mut cursor = skip_ascii_whitespace(bytes, from);
    if bytes.get(cursor) != Some(&b'=') {
        return None;
    }
    cursor = skip_ascii_whitespace(bytes, cursor + 1);
    let quote = match bytes.get(cursor) {
        Some(&byte @ (b'\'' | b'"')) => byte,
        // Unquoted HTML attribute value: runs until whitespace or the end of
        // the tag.
        _ => return unquoted_value(template, cursor),
    };
    let start = cursor + 1;
    let mut end = start;
    while let Some(&byte) = bytes.get(end) {
        if byte == quote {
            // Quote bytes are ASCII, so slicing at them stays on char
            // boundaries even in non-ASCII templates.
            return non_empty(&template[start..end]);
        }
        if byte == b'\\' {
            return None;
        }
        end += 1;
    }
    None
}

fn unquoted_value(template: &str, start: usize) -> Option<&str> {
    let bytes = template.as_bytes();
    let mut end = start;
    while bytes.get(end).copied().is_some_and(is_unquoted_value_byte) {
        end += 1;
    }
    non_empty(&template[start..end])
}

fn non_empty(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}

/// The index of the first non-whitespace byte at or after `cursor`.
#[inline]
fn skip_ascii_whitespace(bytes: &[u8], mut cursor: usize) -> usize {
    while bytes
        .get(cursor)
        .copied()
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        cursor += 1;
    }
    cursor
}

/// A byte that can appear in an unquoted HTML attribute value; the terminators
/// mirror the HTML tokenizer (whitespace, `>`, `/`, quotes, `=`, `` ` ``).
#[inline]
fn is_unquoted_value_byte(byte: u8) -> bool {
    !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'/' | b'"' | b'\'' | b'=' | b'`' | b'<')
}

/// A byte that can appear inside an HTML attribute name (ASCII subset; a
/// non-ASCII byte is treated as a boundary, the same approximation
/// `vue/no-unused-refs` uses for its raw-text token test).
#[inline]
fn is_attribute_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte == b'-'
}
