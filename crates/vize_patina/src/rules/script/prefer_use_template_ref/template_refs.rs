//! Template-side static `ref="…"` collection for `script/prefer-use-template-ref`.
//!
//! A recorded name *creates* a report, so an over-match is a false positive:
//! `ref="x"` inside an HTML comment, a text node, an interpolation string or
//! another attribute's value must not count. The names therefore come from the
//! parsed template ([`SfcScriptContext::template_surface`]), where those are
//! structurally `Comment`, `Text`, `Interpolation` nodes and attribute values.
//!
//! Without a surface (a template in another language such as pug, or one that
//! failed to parse), the raw text is scanned instead. For an HTML-like
//! template, the scan only reads attribute names inside tag headers, skipping
//! comments, text and interpolations. For anything else, the scan keeps the
//! name-based scan, which reads any `ref=` it finds.
//!
//! Only *static* `ref` attributes are collected. `:ref` / `v-bind:ref` bind an
//! expression rather than a name.
//!
//! [`SfcScriptContext::template_surface`]: crate::rules::script::SfcScriptContext::template_surface

use vize_l0::{CompactString, FxHashSet};
use vize_l1::markup::{SurfaceChild, SurfaceTree};

/// Collect the value of every static `ref="name"` attribute of the parsed
/// template.
pub(super) fn surface_ref_names(tree: &SurfaceTree<'_>) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();
    collect_surface(&tree.children, &mut names);
    names
}

fn collect_surface(children: &[SurfaceChild<'_>], names: &mut FxHashSet<CompactString>) {
    for child in children {
        let SurfaceChild::Element(element) = child else {
            continue;
        };
        for attribute in &element.open.attrs {
            if attribute.name.text == "ref"
                && let Some(value) = &attribute.value
                && let Some(name) = non_empty(value.content.text)
            {
                names.insert(CompactString::new(name));
            }
        }
        collect_surface(&element.children, names);
    }
}

/// Collect static `ref="name"` attributes from the raw template text, for a
/// template without a parsed surface.
pub(super) fn source_ref_names(template: &str) -> FxHashSet<CompactString> {
    if !template.trim_start().starts_with('<') {
        return name_scan(template);
    }
    let mut names = FxHashSet::default();
    let bytes = template.as_bytes();
    let mut cursor = 0;
    while let Some(&byte) = bytes.get(cursor) {
        let rest = template.get(cursor..).unwrap_or_default();
        cursor = if rest.starts_with("<!--") {
            skip_past(template, cursor + 4, "-->")
        } else if rest.starts_with("{{") {
            skip_past(template, cursor + 2, "}}")
        } else if byte == b'<' && bytes.get(cursor + 1).is_some_and(u8::is_ascii_alphabetic) {
            scan_tag_header(template, cursor + 1, &mut names)
        } else {
            cursor + 1
        };
    }
    names
}

/// The index just past the next `terminator` at or after `from`, or the end.
fn skip_past(template: &str, from: usize, terminator: &str) -> usize {
    template
        .get(from..)
        .and_then(|rest| rest.find(terminator))
        .map_or(template.len(), |index| from + index + terminator.len())
}

/// Read one open-tag header starting at the tag name, recording a static
/// `ref` attribute. Returns the index just past the header.
fn scan_tag_header(template: &str, from: usize, names: &mut FxHashSet<CompactString>) -> usize {
    let bytes = template.as_bytes();
    let mut cursor = from;
    while bytes
        .get(cursor)
        .is_some_and(|&byte| !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'/'))
    {
        cursor += 1;
    }
    loop {
        while bytes
            .get(cursor)
            .is_some_and(|&byte| byte.is_ascii_whitespace() || byte == b'/')
        {
            cursor += 1;
        }
        match bytes.get(cursor) {
            None => return cursor,
            Some(b'>') => return cursor + 1,
            Some(_) => {}
        }
        let name_start = cursor;
        while bytes
            .get(cursor)
            .is_some_and(|&byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'>' | b'/'))
        {
            cursor += 1;
        }
        let name = template.get(name_start..cursor).unwrap_or_default();
        let value_from = skip_ascii_whitespace(bytes, cursor);
        if bytes.get(value_from) != Some(&b'=') {
            continue;
        }
        cursor = skip_ascii_whitespace(bytes, value_from + 1);
        let (value, end) = match bytes.get(cursor) {
            Some(&quote @ (b'\'' | b'"')) => {
                let start = cursor + 1;
                let end = template
                    .get(start..)
                    .and_then(|rest| rest.find(char::from(quote)))
                    .map_or(template.len(), |index| start + index);
                (template.get(start..end), end + 1)
            }
            _ => {
                let start = cursor;
                let mut end = start;
                while bytes.get(end).copied().is_some_and(is_unquoted_value_byte) {
                    end += 1;
                }
                (template.get(start..end), end)
            }
        };
        if name == "ref"
            && let Some(value) = value.and_then(non_empty)
            && !value.contains('\\')
        {
            names.insert(CompactString::new(value));
        }
        cursor = end;
    }
}

/// The name-based scan for templates in another language: every standalone
/// `ref=` attribute spelling.
fn name_scan(template: &str) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();
    let bytes = template.as_bytes();
    for (index, _) in template.match_indices("ref") {
        // `ref` has to stand alone as an attribute name: the preceding byte may
        // not extend the name (`myref`, `data-ref`) and may not turn it into a
        // bound or namespaced attribute (`:ref`, `v-bind:ref`, `.ref`).
        let before = index.checked_sub(1).and_then(|i| bytes.get(i).copied());
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
            return template.get(start..end).and_then(non_empty);
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
    template.get(start..end).and_then(non_empty)
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
/// non-ASCII byte is treated as a boundary).
#[inline]
fn is_attribute_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte == b'-'
}
