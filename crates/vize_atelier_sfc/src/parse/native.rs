//! Project only geometry admitted by the canonical original Vue descriptor.

use vize_l0::{
    config::{VueDialect, VueVersion},
    pool,
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
};

use crate::types::{BlockLocation, PadOption, SfcParseOptions};

pub(super) fn try_parse(source: &str, options: &SfcParseOptions) -> Option<BlockLocation> {
    // Rejection only: avoid observing ordinary script/style inputs twice. The
    // original descriptor below remains the sole authority for admission.
    let envelope = source.trim_ascii();
    if !envelope.starts_with("<template>")
        || !envelope.ends_with("</template>")
        || options.pad != PadOption::None
        || options.ignore_empty
        || options.template_parse_options.is_some()
    {
        return None;
    }
    let arena = pool::acquire();
    let observed = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = observed.admitted().ok()?;
    let [block] = observed.container().blocks.as_slice() else {
        return None;
    };
    if !block.attrs.is_empty()
        || admitted.ordinary().is_some()
        || admitted.setup().is_some()
        || admitted.styles().len() != 0
    {
        return None;
    }
    let template = admitted.template()?;
    let span = template.block().span();
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
    Some(BlockLocation {
        start,
        end,
        tag_start: block.open_tag.start as usize,
        tag_end: block.close_tag?.end as usize,
        start_line,
        start_column,
        end_line,
        end_column,
    })
}
