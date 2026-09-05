use super::{block_indent, opening_tag, write_attr, write_remaining_attrs};
use crate::{error::FormatError, options::FormatOptions, script};
use vize_s0::{Allocator, ToCompactString};

pub(super) fn write_script_block(
    output: &mut Vec<u8>,
    block: &vize_atelier_sfc::SfcScriptBlock<'_>,
    options: &FormatOptions,
    allocator: &Allocator,
    source: &str,
) -> Result<(), FormatError> {
    let opening_tag = opening_tag::raw_templated_opening_tag(source, &block.loc);
    let content = opening_tag::content_after_opening_tag(
        source,
        &block.loc,
        block.content.as_ref(),
        opening_tag.as_ref(),
    );
    let trimmed = content.trim();
    let source_type =
        script::source_type_for_script_lang(block.lang.as_ref().map(|lang| lang.as_ref()));
    let formatted_content =
        script::format_sfc_script_content_stable(trimmed, options, allocator, source_type)
            .unwrap_or_else(|_| trimmed.to_compact_string());

    if let Some(opening_tag) = &opening_tag {
        opening_tag::write_raw(output, opening_tag);
    } else {
        output.extend_from_slice(b"<script");
        if block.setup {
            write_attr(output, "setup", None);
        }
        if let Some(lang) = &block.lang {
            write_attr(output, "lang", Some(lang));
        }
        write_remaining_attrs(output, &block.attrs, &["setup", "lang"]);
        output.push(b'>');
    }
    output.extend_from_slice(options.newline_bytes());

    if options.vue_indent_script_and_style {
        block_indent::write_indented_block(
            output,
            &formatted_content,
            options.indent_bytes(),
            options.newline_bytes(),
        );
    } else {
        output.extend_from_slice(formatted_content.as_bytes());
        if !formatted_content.ends_with('\n') {
            output.extend_from_slice(options.newline_bytes());
        }
    }

    output.extend_from_slice(b"</script>");
    Ok(())
}
