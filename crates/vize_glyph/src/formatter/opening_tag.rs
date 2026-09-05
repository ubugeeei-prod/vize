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
