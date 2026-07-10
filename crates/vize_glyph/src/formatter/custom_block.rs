use crate::{error::FormatError, options::FormatOptions};

pub(super) fn format(
    output: &mut Vec<u8>,
    block: &vize_atelier_sfc::SfcCustomBlock<'_>,
    options: &FormatOptions,
) -> Result<(), FormatError> {
    output.push(b'<');
    output.extend_from_slice(block.block_type.as_bytes());
    super::write_remaining_attrs(output, &block.attrs, &[]);
    output.push(b'>');
    output.extend_from_slice(options.newline_bytes());

    if block.block_type.as_ref() == "art" {
        write_art_content(output, block.content.as_ref(), options)?;
    } else {
        output.extend_from_slice(block.content.trim().as_bytes());
        output.extend_from_slice(options.newline_bytes());
    }

    output.extend_from_slice(b"</");
    output.extend_from_slice(block.block_type.as_bytes());
    output.push(b'>');
    Ok(())
}

fn write_art_content(
    output: &mut Vec<u8>,
    content: &str,
    options: &FormatOptions,
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
                write_art_chunk(output, &chunk_lines, options)?;
                chunk_lines.clear();
                wrote_chunk = true;
            }
            pending_blank_lines += 1;
            continue;
        }

        if pending_blank_lines > 0 {
            if wrote_chunk {
                for _ in 0..pending_blank_lines {
                    output.extend_from_slice(options.newline_bytes());
                }
            }
            pending_blank_lines = 0;
        }
        chunk_lines.push(line);
    }

    if !chunk_lines.is_empty() {
        if pending_blank_lines > 0 && wrote_chunk {
            for _ in 0..pending_blank_lines {
                output.extend_from_slice(options.newline_bytes());
            }
        }
        write_art_chunk(output, &chunk_lines, options)?;
    }
    Ok(())
}

fn write_art_chunk(
    output: &mut Vec<u8>,
    lines: &[&str],
    options: &FormatOptions,
) -> Result<(), FormatError> {
    let chunk = lines.join("\n");
    let formatted = crate::template::format_template_content(chunk.trim(), options)?;
    let indent = options.indent_bytes();
    for line in formatted.lines() {
        output.extend_from_slice(indent);
        output.extend_from_slice(line.as_bytes());
        output.extend_from_slice(options.newline_bytes());
    }
    Ok(())
}
