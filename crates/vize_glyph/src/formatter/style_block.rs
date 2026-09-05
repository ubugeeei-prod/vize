use super::{block_indent, opening_tag, write_attr, write_remaining_attrs};
use crate::{error::FormatError, options::FormatOptions, style};
use vize_s0::ToCompactString;

pub(super) fn write_style_block(
    output: &mut Vec<u8>,
    block: &vize_atelier_sfc::SfcStyleBlock<'_>,
    options: &FormatOptions,
    source: &str,
) -> Result<(), FormatError> {
    let is_plain_css = block
        .lang
        .as_ref()
        .is_none_or(|lang| lang.as_ref() == "css");
    let opening_tag = opening_tag::raw_templated_opening_tag(source, &block.loc);
    let content = opening_tag::content_after_opening_tag(
        source,
        &block.loc,
        block.content.as_ref(),
        opening_tag.as_ref(),
    );
    let formatted_content = if is_plain_css {
        style::format_style_content(content, options)
            .unwrap_or_else(|_| content.trim().to_compact_string())
    } else {
        content.trim().to_compact_string()
    };

    if let Some(opening_tag) = &opening_tag {
        opening_tag::write_raw(output, opening_tag);
    } else {
        output.extend_from_slice(b"<style");
        if block.scoped {
            write_attr(output, "scoped", None);
        }
        if let Some(lang) = &block.lang {
            write_attr(output, "lang", Some(lang));
        }
        write_remaining_attrs(output, &block.attrs, &["scoped", "lang"]);
        output.push(b'>');
    }
    output.extend_from_slice(options.newline_bytes());

    if options.vue_indent_script_and_style {
        block_indent::write_indented_block(
            output,
            formatted_content.as_str(),
            options.indent_bytes(),
            options.newline_bytes(),
        );
    } else {
        output.extend_from_slice(formatted_content.as_bytes());
        if !formatted_content.ends_with('\n') {
            output.extend_from_slice(options.newline_bytes());
        }
    }

    output.extend_from_slice(b"</style>");
    Ok(())
}
