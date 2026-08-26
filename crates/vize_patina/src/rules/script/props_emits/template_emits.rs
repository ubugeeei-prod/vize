//! Template-side emit usage scan shared by the `script/*` emits rules.
//!
//! Top-level `<script setup>` bindings are visible to the template, so a
//! captured emit function may be called from directive handlers
//! (`@click='emit("save")'`), inline handler statements inside `v-for` /
//! `v-if`, or interpolations — call sites the script AST never contains. This
//! module resolves those calls from the **raw template source**: patina's
//! template and script passes are separate, and handler expressions are opaque
//! strings even in the template AST, so raw-text matching is the same
//! cross-block trade-off `vue/no-unused-refs` makes in the other direction.
//!
//! The scan is one-directional by design: a recorded name only ever
//! *suppresses* an "unused" report, never creates one, so over-matching (a
//! call inside an HTML comment, a coincidental attribute value) can at worst
//! hide a report. Recognized call forms mirror the script-side collector — a
//! whole-identifier callee (`emit(...)`, never `bus.emit(...)`) invoked with a
//! string-literal first argument — plus the template-only `$emit` helper,
//! which dispatches the same declared events. Dynamic event names
//! (`emit(name)`) record nothing, matching the script-side treatment.

use vize_s0::{CompactString, FxHashSet};

/// Record into `used` every event name emitted from `template` through a call
/// of `binding` (the captured `defineEmits` function, template-visible as a
/// top-level `<script setup>` binding) or the built-in `$emit` helper.
pub(super) fn collect_template_emitted_events(
    template: &str,
    binding: &str,
    used: &mut FxHashSet<CompactString>,
) {
    collect_callee_events(template, binding, used);
    collect_callee_events(template, "$emit", used);
}

/// Record the string-literal first argument of every `callee(...)` call in
/// `template`.
///
/// A call site counts only when `callee` stands alone as an identifier token:
/// the byte before it must be neither an identifier byte (so `myemit(` and
/// `$emit(` never match a plain `emit` binding) nor `.` (member calls such as
/// `child.$emit('x')` dispatch on another instance, and the script-side
/// collector likewise tracks only identifier callees), and the byte after it
/// must not extend the identifier. Between the token and its argument list,
/// whitespace and an optional `?.` are accepted, mirroring how the script AST
/// treats `emit?.('save')` as a call of the `emit` identifier.
fn collect_callee_events(template: &str, callee: &str, used: &mut FxHashSet<CompactString>) {
    if callee.is_empty() {
        return;
    }
    let bytes = template.as_bytes();
    for (index, _) in template.match_indices(callee) {
        let before = index.checked_sub(1).map(|i| bytes[i]);
        if before.is_some_and(|byte| is_identifier_byte(byte) || byte == b'.') {
            continue;
        }
        let after = index + callee.len();
        if bytes.get(after).copied().is_some_and(is_identifier_byte) {
            continue;
        }
        if let Some(name) = string_literal_argument(template, after) {
            used.insert(CompactString::new(name));
        }
    }
}

/// Parse `( 'name'` — optionally preceded by `?.` — starting at byte `from`,
/// returning the quoted name.
///
/// Only single- and double-quoted literals are recognized, matching the
/// script-side collector, which counts string-literal arguments only. A
/// literal containing a backslash is skipped rather than unescaped so a
/// mangled name is never recorded; skipping can at worst leave a report in
/// place, never invent one.
fn string_literal_argument(template: &str, from: usize) -> Option<&str> {
    let bytes = template.as_bytes();
    let mut cursor = skip_ascii_whitespace(bytes, from);
    if bytes.get(cursor) == Some(&b'?') && bytes.get(cursor + 1) == Some(&b'.') {
        cursor = skip_ascii_whitespace(bytes, cursor + 2);
    }
    if bytes.get(cursor) != Some(&b'(') {
        return None;
    }
    cursor = skip_ascii_whitespace(bytes, cursor + 1);
    let quote = match bytes.get(cursor) {
        Some(&byte @ (b'\'' | b'"')) => byte,
        _ => return None,
    };
    let start = cursor + 1;
    let mut end = start;
    while let Some(&byte) = bytes.get(end) {
        if byte == quote {
            // Quote bytes are ASCII, so slicing at them stays on char
            // boundaries even in non-ASCII templates.
            return Some(&template[start..end]);
        }
        if byte == b'\\' {
            return None;
        }
        end += 1;
    }
    None
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

/// A byte that can appear in a JS identifier (ASCII subset; a non-ASCII byte
/// is treated as a boundary, the same approximation `vue/no-unused-refs` uses
/// for its raw-text token test).
#[inline]
fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}
