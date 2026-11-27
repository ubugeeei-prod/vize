use super::{block_indent, opening_tag, write_attr, write_authored_attrs, write_remaining_attrs};
use crate::{error::FormatError, options::FormatOptions, script};
use vize_l0::string_builder::TextBuf;
use vize_l0::{Allocator, ToCompactString};

pub(super) fn write_script_block(
    output: &mut TextBuf,
    block: &super::SfcBlock<'_>,
    options: &FormatOptions,
    allocator: &Allocator,
    source: &str,
    sort_imports: Option<&crate::ImportSortOptions>,
) -> Result<(), FormatError> {
    let opening_tag = opening_tag::raw_templated_opening_tag(source, &block.loc);
    let content = opening_tag::content_after_opening_tag(
        source,
        &block.loc,
        block.content.as_ref(),
        opening_tag.as_ref(),
    );
    let trimmed = content.trim();
    let source_type = script::source_type_for_script_lang(block.lang());
    let formatted_content = script::format_sfc_script_content_stable(
        trimmed,
        options,
        allocator,
        source_type,
        sort_imports,
    )
    .unwrap_or_else(|_| {
        if options.vue_indent_script_and_style {
            block_indent::kept_block_body(content)
        } else {
            trimmed.to_compact_string()
        }
    });

    if let Some(opening_tag) = &opening_tag {
        opening_tag::write_raw(output, opening_tag);
    } else {
        output.push_str("<script");
        if !write_authored_attrs(output, &block.attrs, source, &block.loc, options) {
            if block.has_attr("setup") {
                write_attr(output, "setup", None);
            }
            if let Some(lang) = block.lang() {
                write_attr(output, "lang", Some(lang));
            }
            write_remaining_attrs(output, &block.attrs, &["setup", "lang"]);
        }
        output.push('>');
    }
    // An empty block (for example one that only carries `src`) stays empty on
    // one line, as Prettier prints it.
    if trimmed.is_empty() {
        output.push_str("</script>");
        return Ok(());
    }
    output.push_str(options.newline_string());

    if options.vue_indent_script_and_style {
        block_indent::write_indented_block(
            output,
            &formatted_content,
            options.indent_str(),
            options.newline_string(),
        );
    } else {
        output.push_str(&formatted_content);
        if !formatted_content.ends_with(options.newline_string()) {
            output.push_str(options.newline_string());
        }
    }

    output.push_str("</script>");
    Ok(())
}
