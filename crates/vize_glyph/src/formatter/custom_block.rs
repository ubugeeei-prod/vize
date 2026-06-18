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
        write_art_content(output, block.content.as_ref(), options);
    } else {
        output.extend_from_slice(block.content.trim().as_bytes());
        output.extend_from_slice(options.newline_bytes());
    }

    output.extend_from_slice(b"</");
    output.extend_from_slice(block.block_type.as_bytes());
    output.push(b'>');
    Ok(())
}

fn write_art_content(output: &mut Vec<u8>, content: &str, options: &FormatOptions) {
    let lines: Vec<&str> = content.lines().collect();
    let start = lines
        .iter()
        .position(|line| !line.trim().is_empty())
        .unwrap_or(lines.len());
    let end = lines
        .iter()
        .rposition(|line| !line.trim().is_empty())
        .map_or(start, |index| index + 1);
    let indent = options.indent_bytes();
    let mut depth = 1usize;

    for line in &lines[start..end] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            output.extend_from_slice(options.newline_bytes());
            continue;
        }
        if trimmed.starts_with("</") {
            depth = depth.saturating_sub(1);
        }
        for _ in 0..depth {
            output.extend_from_slice(indent);
        }
        output.extend_from_slice(trimmed.as_bytes());
        output.extend_from_slice(options.newline_bytes());
        if opens_block(trimmed) {
            depth += 1;
        }
    }
}

fn opens_block(line: &str) -> bool {
    line.starts_with('<')
        && !line.starts_with("</")
        && !line.starts_with("<!--")
        && !line.ends_with("/>")
        && !line.contains("</")
        && line.contains('>')
}
