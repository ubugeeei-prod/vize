//! Verbatim emission of whitespace-significant elements (`<pre>`,
//! `<textarea>`, and any element carrying `v-pre`). Split into its own file so
//! the already-large `formatter.rs` stays within the source-file-length budget
//! (#3251).

use super::{TemplateFormatter, find_matching_close_tag};

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
