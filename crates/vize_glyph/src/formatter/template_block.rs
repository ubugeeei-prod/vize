//! Template block emission for the SFC formatter.
//!
//! Native HTML templates are reformatted by the HTML template formatter and
//! re-indented one level from the `<template>` tag. Every other template
//! language is opaque: its bytes are preserved and only the common outer
//! indentation is rebased.

use super::{opening_tag, template_indent, write_attr, write_remaining_attrs};
use crate::error::FormatError;
use crate::options::FormatOptions;
use crate::template;

pub(super) fn write_template_block(
    output: &mut Vec<u8>,
    block: &vize_atelier_sfc::SfcTemplateBlock<'_>,
    options: &FormatOptions,
    source: &str,
) -> Result<(), FormatError> {
    // The HTML template formatter owns HTML syntax only. Feeding Pug,
    // Haml, Markdown, or another preprocessor language through it turns
    // indentation-sensitive source into unrelated HTML text nodes and can
    // flatten the program while still reaching a formatter fixed point.
    let native_html = block
        .lang
        .as_deref()
        .is_none_or(|lang| lang.eq_ignore_ascii_case("html"));
    let opening_tag = opening_tag::raw_templated_opening_tag(source, &block.loc);
    let content = opening_tag::content_after_opening_tag(
        source,
        &block.loc,
        block.content.as_ref(),
        opening_tag.as_ref(),
    );
    let formatted_content = native_html
        .then(|| template::format_template_content(content, options))
        .transpose()?;

    if let Some(opening_tag) = &opening_tag {
        opening_tag::write_raw(output, opening_tag);
    } else {
        // Build the opening tag
        output.extend_from_slice(b"<template");
        if let Some(lang) = &block.lang {
            write_attr(output, "lang", Some(lang));
        }
        write_remaining_attrs(output, &block.attrs, &["lang"]);
        output.push(b'>');
    }
    output.extend_from_slice(options.newline_bytes());

    let indent = options.indent_bytes();
    if let Some(formatted_content) = formatted_content {
        // Native HTML is always indented by one level from the template
        // tag — except inside whitespace-significant regions (`<pre>`,
        // `<textarea>`, `v-pre`) where the inner content must round-trip
        // byte-for-byte. (#963)
        let trimmed = formatted_content
            .trim_end_matches('\n')
            .trim_end_matches('\r');
        template_indent::write_indented_template(output, trimmed, indent, options.newline_bytes());
    } else {
        // A preprocessor language owns every byte inside its block. Rebase
        // only the common outer indentation so the SFC stays canonical;
        // relative indentation and all line content remain source-owned.
        template_indent::write_rebased_opaque_template(
            output,
            content,
            indent,
            options.newline_bytes(),
        );
    }

    output.extend_from_slice(b"</template>");

    Ok(())
}
