//! Format an original block under the full document's line-ending policy.
use super::{BlockAttrs, BlockKind, BlockLocation, SfcBlock};
use crate::{
    EndOfLine, FormatError, FormatOptions,
    formatter::{FormatResult, GlyphFormatter, style_block},
};
use std::borrow::Cow;
use vize_l0::String;
use vize_l1::container::vue::StyleView;

impl GlyphFormatter<'_> {
    /// The replacement covers the selected authored tag and its body. The
    /// enclosing document owns all whitespace before and after that tag.
    pub fn format_observed_style(
        &self,
        style: StyleView<'_, '_>,
    ) -> Result<FormatResult, FormatError> {
        let source = style.source();
        let whole = style.original_block().authored_span();
        let content = style.block();
        let span = content.span();
        let attrs: BlockAttrs<'_> = style
            .attrs()
            .iter()
            .map(|attr| {
                (
                    Cow::Borrowed(attr.name),
                    Cow::Borrowed(attr.value.unwrap_or_default()),
                )
            })
            .collect();
        let block = SfcBlock {
            kind: BlockKind::Style,
            name: "style",
            attrs,
            content: Cow::Borrowed(content.source()),
            loc: BlockLocation {
                start: span.start as usize,
                end: span.end as usize,
                tag_start: whole.start as usize,
                tag_end: whole.end as usize,
            },
        };
        let write = |options: &FormatOptions| {
            let mut output = Vec::with_capacity((whole.end - whole.start) as usize);
            style_block::write_style_block(&mut output, &block, options, source)?;
            let code = String::from_utf8(output).map_err(|_| {
                FormatError::StyleFormatError("invalid style output encoding".into())
            })?;
            let changed =
                source.get(whole.start as usize..whole.end as usize) != Some(code.as_str());
            Ok(FormatResult { code, changed })
        };
        if self.options.end_of_line == EndOfLine::Auto {
            self.options.format_with_source_line_ending(source, write)
        } else {
            write(self.options)
        }
    }
}
