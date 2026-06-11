//! Ultra-fast `<template>` block extraction using memchr for
//! SIMD-accelerated search.

use vize_carton::String;
use vize_carton::ToCompactString;

use super::tag_scan::{closing_tag_name_at, find_closing_tag, find_tag_end, tag_name_at};

/// Ultra-fast template extraction using memchr for SIMD-accelerated search.
#[inline]
pub(crate) fn extract_template_fast(source: &str) -> Option<(String, u32)> {
    let bytes = source.as_bytes();

    let (_, content_start) = find_template_block_start(bytes)?;

    // Find matching </template> - handle nesting with simple depth tracking
    let mut depth = 1u32;
    let mut pos = content_start;

    while pos < bytes.len() && depth > 0 {
        // Find next < character
        let next_lt = match memchr::memchr(b'<', &bytes[pos..]) {
            Some(p) => pos + p,
            None => break,
        };

        // Skip HTML comments wholesale. A comment can legitimately contain
        // `</template>` or `<template>` text (e.g. a commented-out template
        // fragment); counting those as real tags would close the block early
        // and truncate the extracted template. An unterminated comment runs to
        // EOF, leaving no trustworthy closing tag, so the scan ends.
        if bytes[next_lt..].starts_with(b"<!--") {
            pos = memchr::memmem::find(&bytes[next_lt + 4..], b"-->")
                .map_or(bytes.len(), |offset| next_lt + 4 + offset + 3);
            continue;
        }

        // Check if it's <template or </template
        if tag_name_at(bytes, next_lt)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(b"template"))
        {
            // Check if self-closing
            if let Some(gt) = memchr::memchr(b'>', &bytes[next_lt..]) {
                let tag_end_pos = next_lt + gt;
                if tag_end_pos > 0 && bytes[tag_end_pos - 1] != b'/' {
                    depth += 1;
                }
                pos = tag_end_pos + 1;
            } else {
                pos = next_lt + 9;
            }
        } else if closing_tag_name_at(bytes, next_lt)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(b"template"))
        {
            depth -= 1;
            if depth == 0 {
                let content = std::str::from_utf8(&bytes[content_start..next_lt]).ok()?;
                return Some((content.to_compact_string(), content_start as u32));
            }
            pos = find_tag_end(bytes, next_lt).map_or(next_lt + 11, |gt| gt + 1);
        } else {
            pos = next_lt + 1;
        }
    }

    None
}

fn find_template_block_start(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut pos = 0;

    while pos < bytes.len() {
        let next_lt = match memchr::memchr(b'<', &bytes[pos..]) {
            Some(offset) => pos + offset,
            None => return None,
        };

        if bytes[next_lt..].starts_with(b"<!--") {
            pos = memchr::memmem::find(&bytes[next_lt + 4..], b"-->")
                .map_or(next_lt + 4, |offset| next_lt + 4 + offset + 3);
            continue;
        }

        let Some((tag_name, _)) = tag_name_at(bytes, next_lt) else {
            pos = next_lt + 1;
            continue;
        };

        let tag_end = find_tag_end(bytes, next_lt)?;
        if tag_name.eq_ignore_ascii_case(b"template") {
            return Some((next_lt, tag_end + 1));
        }

        if tag_end > next_lt && bytes[tag_end - 1] == b'/' {
            pos = tag_end + 1;
            continue;
        }

        pos = find_closing_tag(bytes, tag_name, tag_end + 1)
            .and_then(|close_idx| find_tag_end(bytes, close_idx))
            .map_or(tag_end + 1, |close_end| close_end + 1);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::extract_template_fast;

    fn extract(source: &str) -> Option<vize_carton::String> {
        extract_template_fast(source).map(|(content, _)| content)
    }

    #[test]
    fn extract_template_fast_skips_template_prefix_custom_blocks() {
        let source = "<template-card></template-card><template><div /></template>";

        assert_eq!(extract(source).as_deref(), Some("<div />"));
    }

    #[test]
    fn extract_template_fast_skips_template_strings_in_script_blocks() {
        let source =
            "<script>const tag = '<template></template>';</script><template><span /></template>";

        assert_eq!(extract(source).as_deref(), Some("<span />"));
    }

    #[test]
    fn extract_template_fast_handles_nested_template_tags() {
        let source = "<template><template #default><slot /></template></template>";

        assert_eq!(
            extract(source).as_deref(),
            Some("<template #default><slot /></template>")
        );
    }

    #[test]
    fn extract_template_fast_ignores_closing_tag_inside_comment() {
        // A commented-out `</template>` must not close the block early.
        let source = "<template><div /><!-- </template> --><span /></template>";

        assert_eq!(
            extract(source).as_deref(),
            Some("<div /><!-- </template> --><span />")
        );
    }

    #[test]
    fn extract_template_fast_ignores_opening_tag_inside_comment() {
        // A commented-out `<template>` must not inflate the nesting depth and
        // swallow the real closing tag.
        let source = "<template><!-- <template> --><div /></template><script>x</script>";

        assert_eq!(
            extract(source).as_deref(),
            Some("<!-- <template> --><div />")
        );
    }

    #[test]
    fn extract_template_fast_handles_unterminated_comment() {
        // An unterminated comment leaves no trustworthy closing tag; extraction
        // bails rather than returning a truncated or corrupt template.
        let source = "<template><div /><!-- no end";

        assert_eq!(extract(source), None);
    }
}
