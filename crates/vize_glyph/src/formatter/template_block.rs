//! Template block emission for the SFC formatter.
//!
//! Native HTML templates are reformatted by the HTML template formatter and
//! re-indented one level from the `<template>` tag. Every other template
//! language is opaque: its bytes are preserved and only the common outer
//! indentation is rebased. Markdown is stricter: Ox Content owns its complete
//! authored payload, so the formatter does not even rebase indentation or line
//! endings between the template tags.

use super::{
    opening_tag, template_indent, write_attr, write_authored_attrs, write_remaining_attrs,
};
use crate::error::FormatError;
use crate::options::FormatOptions;
use crate::template;
use vize_l0::string_builder::TextBuf;

pub(super) fn write_template_block(
    output: &mut TextBuf,
    block: &super::SfcBlock<'_>,
    options: &FormatOptions,
    source: &str,
    vue_version: crate::VueVersion,
    preserve_whitespace: bool,
) -> Result<(), FormatError> {
    // The HTML template formatter owns HTML syntax only. Feeding Pug,
    // Haml, Markdown, or another preprocessor language through it turns
    // indentation-sensitive source into unrelated HTML text nodes and can
    // flatten the program while still reaching a formatter fixed point.
    let language = block.lang();
    let native_html = vize_l1::container::lang::is_html_template(language);
    let authored_markdown = language.is_some_and(|lang| lang.eq_ignore_ascii_case("md"));
    let opening_tag = opening_tag::raw_templated_opening_tag(source, &block.loc);
    let content = opening_tag::content_after_opening_tag(
        source,
        &block.loc,
        block.content.as_ref(),
        opening_tag.as_ref(),
    );
    let formatted_content = (native_html && !preserve_whitespace)
        .then(|| {
            template::format_template_content_with_base_depth(content, options, vue_version, 1)
        })
        .transpose()?;

    if let Some(opening_tag) = &opening_tag {
        opening_tag::write_raw(output, opening_tag);
    } else {
        // Build the opening tag
        output.push_str("<template");
        if !write_authored_attrs(output, &block.attrs, source, &block.loc, options) {
            if let Some(lang) = block.lang() {
                write_attr(output, "lang", Some(lang));
            }
            write_remaining_attrs(output, &block.attrs, &["lang"]);
        }
        output.push('>');
    }
    if authored_markdown {
        // Frontmatter delimiters, code fences and their indentation are all
        // Markdown syntax owned by Ox Content. Until explicit Markdown
        // formatting exists, copying this slice is the only semantics-safe
        // operation, including when the SFC formatter uses another EOL mode.
        output.push_str(content);
        output.push_str("</template>");
        return Ok(());
    }
    // An empty block stays empty on one line, as Prettier prints it.
    if content.trim().is_empty() {
        output.push_str("</template>");
        return Ok(());
    }
    if native_html && preserve_whitespace {
        // Layout-looking newlines and indentation can belong to rendered text
        // under compiler.whitespace=preserve. Format tag syntax while leaving
        // all bytes between tags and interpolation delimiters with their owner.
        let formatted =
            template::format_template_content_preserving_text(content, options, vue_version)?;
        output.push_str(&formatted);
        output.push_str("</template>");
        return Ok(());
    }
    output.push_str(options.newline_string());

    let indent = options.indent_str();
    if let Some(formatted_content) = formatted_content {
        // Native HTML is always indented by one level from the template
        // tag — except inside whitespace-significant regions (`<pre>`,
        // `<textarea>`, `v-pre`) where the inner content must round-trip
        // byte-for-byte. (#963)
        let trimmed = formatted_content
            .trim_end_matches('\n')
            .trim_end_matches('\r');
        template_indent::write_indented_template(output, trimmed, indent, options.newline_string());
    } else {
        // A preprocessor language owns every byte inside its block. Rebase
        // only the common outer indentation so the SFC stays canonical;
        // relative indentation and all line content remain source-owned.
        template_indent::write_rebased_opaque_template(
            output,
            content,
            indent,
            options.newline_string(),
        );
    }

    output.push_str("</template>");

    Ok(())
}
