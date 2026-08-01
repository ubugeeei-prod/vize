//! Tag lexing for the whitespace-significant regions of the raw-line mask.
//!
//! `<pre>` and `<textarea>` are whitespace-significant by name, so the mask can
//! find them with fixed open/close needles. `v-pre` makes *any* element
//! whitespace-significant — `is_whitespace_significant_element` treats an
//! element carrying it exactly like `<pre>` and the template formatter copies
//! its content verbatim — so for those the mask has to read the element's own
//! name out of the source to know which `</tag>` ends the region. (#3379)

/// A whitespace-significant region the mask is currently inside.
#[derive(Clone, Copy)]
pub(super) struct RawRegion<'a> {
    /// The element's tag name, as spelled in the source.
    pub(super) tag: &'a [u8],
    /// Whether the region was opened by a `v-pre` attribute rather than by the
    /// tag name itself. `<pre>`/`<textarea>` regions are closed by their fixed
    /// close needles and `v-pre` regions by a tag-name comparison, so the two
    /// mechanisms must never pop each other's entries.
    pub(super) v_pre: bool,
}

/// The tag name starting at `start`, or `None` when the bytes there do not
/// begin an ASCII-alphabetic tag name.
///
/// Vue element names carry more than letters — `<my-el>`, `<Foo.Bar>`,
/// `<svg:rect>` — so the scan only stops at a byte that cannot appear in a
/// name. An unterminated name (the tag continues on the next line) runs to the
/// end of the line, which is the whole name in that case.
pub(super) fn tag_name_at(bytes: &[u8], start: usize) -> Option<&[u8]> {
    if !bytes.get(start)?.is_ascii_alphabetic() {
        return None;
    }
    let end = bytes[start..]
        .iter()
        .position(|byte| !is_tag_name_byte(*byte))
        .map_or(bytes.len(), |offset| start + offset);
    Some(&bytes[start..end])
}

fn is_tag_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b':')
}

/// Whether a bare `v-pre` attribute starts at `cursor` inside an opening tag.
///
/// Vue ignores the directive's value, and so does
/// `is_whitespace_significant_element`, so `=` terminates the name alongside
/// whitespace, `/`, `>` and the end of the line — an opening tag may be split
/// over several lines, which is exactly how the shape that motivated this
/// appears in the wild (`<code`, then `v-pre`, then `class=…>`).
///
/// The byte before the name must be a separator, so `data-v-pre` and
/// `:title="v-pre"` never match. A cursor at column 0 is a separator too: the
/// attribute then opens a continuation line of the tag.
pub(in crate::formatter) fn starts_v_pre_attribute(bytes: &[u8], cursor: usize) -> bool {
    const NAME: &[u8] = b"v-pre";
    if !bytes[cursor..]
        .get(..NAME.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(NAME))
    {
        return false;
    }
    let terminated = bytes
        .get(cursor + NAME.len())
        .is_none_or(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'=' | b'/' | b'>'));
    let separated = cursor == 0 || matches!(bytes[cursor - 1], b' ' | b'\t' | b'\n');
    terminated && separated
}

#[cfg(test)]
mod tests {
    use super::{starts_v_pre_attribute, tag_name_at};

    #[test]
    fn tag_name_reads_component_and_namespaced_names() {
        assert_eq!(tag_name_at(b"<my-el a>", 1), Some(&b"my-el"[..]));
        assert_eq!(tag_name_at(b"<Foo.Bar>", 1), Some(&b"Foo.Bar"[..]));
        assert_eq!(tag_name_at(b"<svg:rect/>", 1), Some(&b"svg:rect"[..]));
        assert_eq!(tag_name_at(b"<code", 1), Some(&b"code"[..]));
    }

    #[test]
    fn tag_name_rejects_a_non_alphabetic_start() {
        assert_eq!(tag_name_at(b"</div>", 1), None);
        assert_eq!(tag_name_at(b"<3>", 1), None);
        assert_eq!(tag_name_at(b"<", 1), None);
    }

    #[test]
    fn v_pre_attribute_is_recognized_in_every_bare_position() {
        assert!(starts_v_pre_attribute(b"<div v-pre>", 5));
        assert!(starts_v_pre_attribute(b"<div v-pre />", 5));
        assert!(starts_v_pre_attribute(b"<div v-pre class=\"a\">", 5));
        assert!(starts_v_pre_attribute(b"<div v-pre", 5));
        assert!(
            starts_v_pre_attribute(b"v-pre", 0),
            "a tag split over lines"
        );
        assert!(
            starts_v_pre_attribute(b"<div V-PRE>", 5),
            "case-insensitive"
        );
        assert!(
            starts_v_pre_attribute(b"<div v-pre=\"\">", 5),
            "valued form"
        );
    }

    #[test]
    fn v_pre_attribute_rejects_look_alikes() {
        assert!(!starts_v_pre_attribute(b"<div data-v-pre>", 10));
        assert!(!starts_v_pre_attribute(b"<div v-precise>", 5));
        assert!(!starts_v_pre_attribute(b"<div :t=\"v-pre\">", 9));
        assert!(!starts_v_pre_attribute(b"<div v-show>", 5));
    }
}
