//! Verbatim emission of whitespace-significant elements (`<pre>`,
//! `<textarea>`, and any element carrying `v-pre`). Split into its own file so
//! the already-large `formatter.rs` stays within the source-file-length budget
//! (#3251).

use super::TemplateFormatter;
use crate::template::{
    WHITESPACE_SIGNIFICANT_NATIVE_ELEMENTS,
    attributes::ParsedAttribute,
    helpers::{find_bytes, is_void_element_str},
};

impl TemplateFormatter<'_> {
    /// Emit a whitespace-significant element verbatim, returning the source
    /// offset to resume formatting from.
    ///
    /// The opening tag has already been rendered up to (but not including) its
    /// `>`; `end_pos` points just past that `>` in `source`. The element's
    /// content must be copied byte-for-byte because a formatter must never
    /// change rendered output. (#963)
    pub(super) fn copy_whitespace_significant_element(
        &self,
        source: &[u8],
        end_pos: usize,
        tag_name: &str,
        len: usize,
        output: &mut Vec<u8>,
    ) -> usize {
        output.push(b'>');
        let Some(close_start) = find_matching_close_tag(source, end_pos, tag_name) else {
            // Unclosed — copy the rest and stop.
            output.extend_from_slice(&source[end_pos..]);
            return len;
        };
        output.extend_from_slice(&source[end_pos..close_start]);
        // If the closing tag is incomplete (no `>`, e.g. an unterminated LSP
        // buffer like `<pre>body</pre\n`), preserve the remaining source
        // verbatim instead of fabricating a `>` and dropping the tail.
        let Some(close_offset) = memchr::memchr(b'>', &source[close_start..]) else {
            output.extend_from_slice(&source[close_start..]);
            return len;
        };
        output.extend_from_slice(b"</");
        output.extend_from_slice(tag_name.as_bytes());
        output.push(b'>');
        output.extend_from_slice(self.newline);
        // The closing tag may carry whitespace before `>` (the Prettier
        // `</pre\n  >` trick that keeps the trailing newline out of `<pre>`
        // content), so scan to the actual `>` rather than assuming a bare
        // `</tag_name>`. Skipping only the bare length would leave `  >` behind
        // as a stray text node and change the rendered output. (#3249)
        close_start + close_offset + 1
    }
}

/// Returns true if the element's content must be preserved byte-for-byte:
/// `<pre>`, `<textarea>`, `<listing>`, or any element with the `v-pre`
/// directive.
/// Whitespace and interpolations inside these regions are rendered as-is
/// at runtime, so the formatter must not touch them. (#963)
pub(super) fn is_whitespace_significant_element(tag_name: &str, attrs: &[ParsedAttribute]) -> bool {
    // Vue resolves tags containing uppercase ASCII as components. Only the
    // lowercase native spelling owns HTML whitespace-significant semantics.
    if WHITESPACE_SIGNIFICANT_NATIVE_ELEMENTS.contains(&tag_name) {
        return true;
    }
    attrs
        .iter()
        .any(|attr| attr.name.eq_ignore_ascii_case("v-pre"))
}

/// Find the start of the matching `</tag_name>` for a content region that
/// begins at `start` in `source`. Returns the byte index of the `<` of the
/// closing tag, or `None` if no matching close is found.
///
/// This is a tag-name aware scan so nested elements with the same tag are
/// handled correctly (e.g. `<pre>...<pre>x</pre>...</pre>`).
fn find_matching_close_tag(source: &[u8], start: usize, tag_name: &str) -> Option<usize> {
    let len = source.len();
    let tag_bytes = tag_name.as_bytes();
    let mut pos = start;
    let mut depth: i32 = 1;
    while pos < len {
        let offset = memchr::memchr(b'<', &source[pos..])?;
        pos += offset;
        if pos + 1 >= len {
            return None;
        }
        // Skip comments and CDATA to avoid false matches inside them.
        if pos + 3 < len && &source[pos..pos + 4] == b"<!--" {
            if let Some(end) = find_bytes(&source[pos..], b"-->") {
                pos += end + 3;
                continue;
            }
            return None;
        }

        let is_closing = source[pos + 1] == b'/';
        let name_start = if is_closing { pos + 2 } else { pos + 1 };
        if name_start >= len {
            return None;
        }
        let name_bytes = &source[name_start..];
        if name_bytes.len() >= tag_bytes.len()
            && name_bytes[..tag_bytes.len()].eq_ignore_ascii_case(tag_bytes)
        {
            let after = name_bytes.get(tag_bytes.len()).copied().unwrap_or(0);
            let after_is_terminator = matches!(after, b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r');
            if after_is_terminator {
                if is_closing {
                    depth -= 1;
                    if depth == 0 {
                        return Some(pos);
                    }
                } else if !is_void_element_str(tag_name) {
                    // Treat self-closing forms (`<tag … />`) as not opening
                    // a new nesting level. Peek to the next `>` and check
                    // for a preceding `/`.
                    if let Some(gt) = memchr::memchr(b'>', &source[pos..]) {
                        let close_at = pos + gt;
                        if close_at > 0 && source[close_at - 1] != b'/' {
                            depth += 1;
                        }
                        pos = close_at + 1;
                        continue;
                    }
                }
            }
        }
        pos += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::options::FormatOptions;
    use crate::template::format_template_content;

    #[test]
    fn test_pre_split_closing_tag_leaves_no_stray_gt() {
        // A closing tag split across lines (`</pre\n  >`, the Prettier trick
        // that keeps a trailing newline out of `<pre>` content) must be
        // consumed whole. Leaving the trailing `>` behind reprinted it as a
        // stray text node and changed the rendered output. (#3249)
        let options = FormatOptions::default();

        let source = "<pre>\npage: {{ x }}</pre\n  >";
        let result = format_template_content(source, &options).unwrap();
        assert_eq!(result.as_str(), "<pre>\npage: {{ x }}</pre>");
        assert_eq!(
            format_template_content(&result, &options).unwrap(),
            result,
            "collapsed close tag must be idempotent"
        );

        // Same trick on `<textarea>`.
        let ta = "<textarea>value</textarea\n>";
        assert_eq!(
            format_template_content(ta, &options).unwrap().as_str(),
            "<textarea>value</textarea>"
        );

        // A whitespace-significant element whose split close tag has no inner
        // content still round-trips cleanly.
        let empty_pre = "<pre></pre\n>";
        assert_eq!(
            format_template_content(empty_pre, &options)
                .unwrap()
                .as_str(),
            "<pre></pre>"
        );
    }

    #[test]
    fn test_pre_ordinary_closing_tag_unaffected() {
        // Regression guard: a normal `</pre>` (no split) is unchanged and the
        // inner content stays byte-for-byte. (#3249)
        let options = FormatOptions::default();
        let source = "<pre>\n  a\n    b</pre>";
        assert_eq!(
            format_template_content(source, &options).unwrap().as_str(),
            source
        );
    }
}
