use vize_atelier_sfc::BlockLocation;

pub(super) struct RawOpeningTag<'a> {
    text: &'a str,
    content_start: usize,
}

pub(super) fn raw_templated_opening_tag<'a>(
    source: &'a str,
    loc: &BlockLocation,
) -> Option<RawOpeningTag<'a>> {
    let raw_end = templated_opening_tag_end(source, loc)?;
    Some(RawOpeningTag {
        text: source.get(loc.tag_start..raw_end)?,
        content_start: raw_end,
    })
}

pub(super) fn content_after_opening_tag<'a>(
    source: &'a str,
    loc: &BlockLocation,
    fallback: &'a str,
    opening_tag: Option<&RawOpeningTag<'_>>,
) -> &'a str {
    let start = opening_tag.map_or(loc.start, |tag| tag.content_start);
    source.get(start..loc.end).unwrap_or(fallback)
}

pub(super) fn write_raw(output: &mut Vec<u8>, opening_tag: &RawOpeningTag<'_>) {
    output.extend_from_slice(opening_tag.text.as_bytes());
}

fn templated_opening_tag_end(source: &str, loc: &BlockLocation) -> Option<usize> {
    let raw = source.get(loc.tag_start..loc.start)?;
    let contains_marker = raw
        .as_bytes()
        .windows(2)
        .any(|pair| pair == b"<%" || pair == b"%>");
    contains_marker.then(|| {
        loc.start
            + usize::from(
                source
                    .as_bytes()
                    .get(loc.start)
                    .is_some_and(|byte| *byte == b'>'),
            )
    })
}

/// Names of the block's attributes in authored order, first occurrence wins.
///
/// Returns `None` unless the scanned names are exactly the parser's attribute
/// keys, so a caller can fall back to the canonical order whenever the raw tag
/// disagrees with what the SFC parser recorded.
pub(super) fn authored_attr_order<'s>(
    source: &'s str,
    loc: &BlockLocation,
    attr_count: usize,
    has_attr: impl Fn(&str) -> bool,
) -> Option<Vec<&'s str>> {
    let raw = source.get(loc.tag_start..loc.start)?;
    let bytes = raw.as_bytes();
    let is_space = |b: u8| matches!(b, b' ' | b'\t' | b'\n' | b'\r');
    let mut pos = 1;
    while bytes
        .get(pos)
        .is_some_and(|&b| !is_space(b) && b != b'>' && b != b'/')
    {
        pos += 1;
    }
    let mut names: Vec<&str> = Vec::with_capacity(attr_count);
    loop {
        while bytes.get(pos).is_some_and(|&b| is_space(b)) {
            pos += 1;
        }
        match bytes.get(pos) {
            None | Some(b'>' | b'/') => break,
            Some(_) => {}
        }
        let name_start = pos;
        while bytes
            .get(pos)
            .is_some_and(|&b| !is_space(b) && !matches!(b, b'=' | b'>' | b'/'))
        {
            pos += 1;
        }
        if pos == name_start {
            pos += 1;
            continue;
        }
        let name = raw.get(name_start..pos)?;
        while matches!(bytes.get(pos), Some(b' ' | b'\t')) {
            pos += 1;
        }
        if bytes.get(pos) == Some(&b'=') {
            pos += 1;
            while matches!(bytes.get(pos), Some(b' ' | b'\t')) {
                pos += 1;
            }
            if let Some(&quote) = bytes.get(pos).filter(|&&b| b == b'"' || b == b'\'') {
                let close = bytes.get(pos + 1..)?.iter().position(|&b| b == quote)?;
                pos += close + 2;
            } else {
                while bytes
                    .get(pos)
                    .is_some_and(|&b| !is_space(b) && b != b'>' && b != b'/')
                {
                    pos += 1;
                }
            }
        }
        if !names.contains(&name) {
            names.push(name);
        }
    }
    (names.len() == attr_count && names.iter().all(|name| has_attr(name))).then_some(names)
}
