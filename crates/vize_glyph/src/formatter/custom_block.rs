use crate::{error::FormatError, options::FormatOptions};
use vize_l0::string_builder::TextBuf;

pub(super) fn format(
    output: &mut TextBuf,
    block: &super::SfcBlock<'_>,
    options: &FormatOptions,
    source: &str,
    vue_version: crate::VueVersion,
) -> Result<(), FormatError> {
    let opening_tag = super::opening_tag::raw_templated_opening_tag(source, &block.loc);
    if let Some(opening_tag) = &opening_tag {
        super::opening_tag::write_raw(output, opening_tag);
    } else {
        output.push('<');
        output.push_str(block.name);
        if !super::write_authored_attrs(output, &block.attrs, source, &block.loc, options) {
            super::write_remaining_attrs(output, &block.attrs, &[]);
        }
        output.push('>');
    }
    output.push_str(options.newline_string());

    let content = super::opening_tag::content_after_opening_tag(
        source,
        &block.loc,
        block.content.as_ref(),
        opening_tag.as_ref(),
    );
    let body_start = output.as_str().len();
    if block.name != "art" || write_art_content(output, content, options, vue_version).is_err() {
        // A body the template formatter declines is kept like any other
        // custom block's, without the chunks already laid out.
        output.truncate(body_start);
        output.push_str(content.trim());
        output.push_str(options.newline_string());
    }

    output.push_str("</");
    output.push_str(block.name);
    output.push('>');
    Ok(())
}

fn write_art_content(
    output: &mut TextBuf,
    content: &str,
    options: &FormatOptions,
    vue_version: crate::VueVersion,
) -> Result<(), FormatError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let mut chunk_lines: Vec<&str> = Vec::new();
    let mut pending_blank_lines = 0usize;
    let mut wrote_chunk = false;

    for line in trimmed.lines() {
        if line.trim().is_empty() {
            if !chunk_lines.is_empty() {
                write_art_chunk(output, &chunk_lines, options, vue_version)?;
                chunk_lines.clear();
                wrote_chunk = true;
            }
            pending_blank_lines += 1;
            continue;
        }

        if pending_blank_lines > 0 {
            if wrote_chunk {
                for _ in 0..pending_blank_lines {
                    output.push_str(options.newline_string());
                }
            }
            pending_blank_lines = 0;
        }
        chunk_lines.push(line);
    }

    if !chunk_lines.is_empty() {
        if pending_blank_lines > 0 && wrote_chunk {
            for _ in 0..pending_blank_lines {
                output.push_str(options.newline_string());
            }
        }
        write_art_chunk(output, &chunk_lines, options, vue_version)?;
    }
    Ok(())
}

fn write_art_chunk(
    output: &mut TextBuf,
    lines: &[&str],
    options: &FormatOptions,
    vue_version: crate::VueVersion,
) -> Result<(), FormatError> {
    let chunk = lines.join("\n");
    let formatted = crate::template::format_template_content_with_vue_version(
        chunk.trim(),
        options,
        vue_version,
    )?;
    let trimmed = formatted.trim_end_matches('\n').trim_end_matches('\r');
    super::template_indent::write_indented_template(
        output,
        trimmed,
        options.indent_str(),
        options.newline_string(),
    );
    Ok(())
}
