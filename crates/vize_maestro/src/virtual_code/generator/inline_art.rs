//! Inline `<art>` variant scanning for regular Vue SFC custom blocks.

/// Byte ranges for a `<variant>` inside an inline `<art>` custom block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InlineArtVariant {
    /// Start offset of the `<variant...>` opening tag in the original file.
    pub variant_start: usize,
    /// End offset of the `</variant>` closing tag in the original file.
    pub variant_end: usize,
    /// Offset immediately after the opening tag.
    pub body_start: usize,
    /// Offset immediately before the closing tag.
    pub body_end: usize,
    /// Start offset of non-whitespace template content.
    pub template_start: usize,
    /// End offset of non-whitespace template content.
    pub template_end: usize,
}

pub(crate) fn inline_art_variants(content: &str, content_start: usize) -> Vec<InlineArtVariant> {
    let mut variants = Vec::new();
    let mut cursor = 0usize;

    while let Some(open_start) = find_next_variant_open(content, cursor) {
        let Some(open_tag_end) = find_tag_end(content, open_start).map(|end| end + 1) else {
            break;
        };

        let (body_end, variant_end) =
            if let Some((close_start, close_tag_end)) = find_variant_close(content, open_tag_end) {
                (close_start, close_tag_end)
            } else {
                (content.len(), content.len())
            };

        let body_start = open_tag_end;
        let (template_start, template_end) = trim_template_range(content, body_start, body_end);
        variants.push(InlineArtVariant {
            variant_start: content_start + open_start,
            variant_end: content_start + variant_end,
            body_start: content_start + body_start,
            body_end: content_start + body_end,
            template_start: content_start + template_start,
            template_end: content_start + template_end,
        });

        if variant_end <= open_start {
            break;
        }
        cursor = variant_end;
    }

    variants
}

fn find_next_variant_open(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut cursor = start;
    while let Some(relative) = content.get(cursor..)?.find('<') {
        let candidate = cursor + relative;
        let name_end = candidate + "<variant".len();
        if bytes
            .get(candidate..name_end)?
            .eq_ignore_ascii_case(b"<variant")
            && bytes
                .get(name_end)
                .is_none_or(|byte| is_tag_name_boundary(*byte))
        {
            return Some(candidate);
        }
        cursor = candidate + 1;
    }
    None
}

fn find_variant_close(content: &str, start: usize) -> Option<(usize, usize)> {
    let bytes = content.as_bytes();
    let mut cursor = start;
    while let Some(relative) = content.get(cursor..)?.find("</") {
        let candidate = cursor + relative;
        let name_end = candidate + "</variant".len();
        if bytes
            .get(candidate..name_end)?
            .eq_ignore_ascii_case(b"</variant")
            && bytes
                .get(name_end)
                .is_none_or(|byte| is_tag_name_boundary(*byte))
        {
            let close_tag_end = find_tag_end(content, candidate)? + 1;
            return Some((candidate, close_tag_end));
        }
        cursor = candidate + 2;
    }
    None
}

fn find_tag_end(content: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (relative, byte) in content.as_bytes().get(start..)?.iter().copied().enumerate() {
        match quote {
            Some(current) if byte == current => quote = None,
            Some(_) => {}
            None if byte == b'\'' || byte == b'"' => quote = Some(byte),
            None if byte == b'>' => return Some(start + relative),
            None => {}
        }
    }
    None
}

fn is_tag_name_boundary(byte: u8) -> bool {
    byte.is_ascii_whitespace() || byte == b'>' || byte == b'/'
}

fn trim_template_range(content: &str, start: usize, end: usize) -> (usize, usize) {
    if start >= end || end > content.len() {
        return (end, end);
    }

    let segment = &content[start..end];
    let Some((first, _)) = segment.char_indices().find(|(_, ch)| !ch.is_whitespace()) else {
        return (end, end);
    };
    let last_end = segment
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(first);

    (start + first, start + last_end)
}
