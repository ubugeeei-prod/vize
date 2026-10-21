//! Project only geometry admitted by the canonical original Vue descriptor.

use vize_l1::container::Vue;

use crate::types::{BlockLocation, PadOption, SfcParseOptions};

pub(super) struct NativeTemplateProjection<'a> {
    pub(super) loc: BlockLocation,
    pub(super) lang: Option<&'a str>,
}

pub(super) fn try_parse<'a>(
    source: &'a str,
    options: &SfcParseOptions,
) -> Option<NativeTemplateProjection<'a>> {
    if options.pad != PadOption::None
        || options.ignore_empty
        || options.template_parse_options.is_some()
    {
        return None;
    }
    let observed = Vue.observe_template_only(source)?;
    let span = observed.block().span();
    // The returned public value borrows the caller's source, never arena data.
    // All coordinates come from this exact source's genuine original owner.
    let start = span.start as usize;
    let end = span.end as usize;
    let content = source.get(start..end)?;
    let prefix = source.get(..start)?;
    let start_line = memchr::memchr_iter(b'\n', prefix.as_bytes()).count() + 1;
    let start_column = prefix.rfind('\n').map_or(start + 1, |at| start - at);
    let end_line = start_line + memchr::memchr_iter(b'\n', content.as_bytes()).count();
    let end_column = content
        .rfind('\n')
        .map_or(start_column + content.len(), |at| content.len() - at);
    Some(NativeTemplateProjection {
        loc: BlockLocation {
            start,
            end,
            tag_start: observed.open_tag().start as usize,
            tag_end: observed.close_tag().end as usize,
            start_line,
            start_column,
            end_line,
            end_column,
        },
        lang: observed.lang(),
    })
}
