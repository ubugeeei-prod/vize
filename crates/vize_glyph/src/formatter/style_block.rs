use super::{block_indent, opening_tag, write_attr, write_authored_attrs, write_remaining_attrs};
use crate::{error::FormatError, options::FormatOptions, style};
use vize_l0::ToCompactString;

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
    let kept = || {
        if options.vue_indent_script_and_style {
            block_indent::kept_block_body(content)
        } else {
            content.trim().to_compact_string()
        }
    };
    let formatted_content = if is_plain_css {
        // Comments keep their authored indentation, so an indented body is
        // formatted from its dedented form.
        let kept = kept();
        let source = if options.vue_indent_script_and_style {
            kept.as_str()
        } else {
            content
        };
        style::format_style_content(source, options).unwrap_or(kept)
    } else if scss_or_less(block.lang.as_deref()) {
        let kept = kept();
        let content = kept.as_str();
        let requoted = options
            .normalize_css_quotes
            .then(|| style::requote_preprocessor_strings(content, options));
        let content = requoted.as_deref().unwrap_or(content);
        options
            .reindent_preprocessor_styles
            .then(|| style::reindent_preprocessor(content, options))
            .flatten()
            .unwrap_or_else(|| content.to_compact_string())
    } else {
        kept()
    };

    if let Some(opening_tag) = &opening_tag {
        opening_tag::write_raw(output, opening_tag);
    } else {
        output.extend_from_slice(b"<style");
        if !write_authored_attrs(output, &block.attrs, source, &block.loc, options) {
            if block.scoped {
                write_attr(output, "scoped", None);
            }
            if let Some(lang) = &block.lang {
                write_attr(output, "lang", Some(lang));
            }
            write_remaining_attrs(output, &block.attrs, &["scoped", "lang"]);
        }
        output.push(b'>');
    }
    // An empty block (for example one that only carries `src`) stays empty on
    // one line, as Prettier prints it.
    if content.trim().is_empty() {
        output.extend_from_slice(b"</style>");
        return Ok(());
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
        if !formatted_content.ends_with(options.newline_string()) {
            output.extend_from_slice(options.newline_bytes());
        }
    }

    output.extend_from_slice(b"</style>");
    Ok(())
}

/// Sass (SCSS syntax) and Less share CSS string and comment syntax; the
/// indented Sass and Stylus dialects are left alone, as Prettier does.
fn scss_or_less(lang: Option<&str>) -> bool {
    lang.is_some_and(|lang| lang.eq_ignore_ascii_case("scss") || lang.eq_ignore_ascii_case("less"))
}
