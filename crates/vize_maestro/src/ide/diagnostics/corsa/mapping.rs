//! Translation of Corsa diagnostic ranges back into SFC source coordinates.

use vize_canon::ImportSourceMap;

pub(super) type LspRangeParts = (u32, u32, u32, u32);

#[allow(clippy::too_many_arguments)]
pub(super) fn map_diagnostic_with_source_mappings(
    virtual_ts: &str,
    source: &str,
    mappings: &[vize_canon::virtual_ts::VizeMapping],
    import_source_map: &ImportSourceMap,
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
) -> Option<LspRangeParts> {
    // Diagnostics come back from Corsa in coordinates of the *rewritten*
    // virtual TS (the one we sent). The byte-range mappings, however, were
    // produced before the `.vue` → `.vue.ts` rewrite. Translate first.
    let start_offset_post = line_character_to_byte_offset(virtual_ts, start_line, start_character)?;
    let end_offset_post = line_character_to_byte_offset(virtual_ts, end_line, end_character)
        .unwrap_or(start_offset_post.saturating_add(1));
    let start_offset = import_source_map.get_original_offset(start_offset_post as u32) as usize;
    let end_offset = import_source_map.get_original_offset(end_offset_post as u32) as usize;
    let start_mapping = mapping_for_generated_offset(mappings, start_offset)?;
    let src_start = map_generated_offset_to_source(start_mapping, start_offset);
    let src_end = mapping_for_generated_offset(mappings, end_offset)
        .map(|mapping| map_generated_offset_to_source(mapping, end_offset))
        .unwrap_or_else(|| {
            let generated_len = end_offset.saturating_sub(start_offset);
            src_start
                .saturating_add(generated_len)
                .min(start_mapping.src_range.end)
        })
        .max(src_start.saturating_add(1));

    let (start_line, start_char) = source_offset_to_position(source, src_start);
    let (end_line, end_char) = source_offset_to_position(source, src_end.min(source.len()));
    Some((start_line, end_line, start_char, end_char))
}

fn mapping_for_generated_offset(
    mappings: &[vize_canon::virtual_ts::VizeMapping],
    offset: usize,
) -> Option<&vize_canon::virtual_ts::VizeMapping> {
    mappings
        .iter()
        .find(|mapping| offset >= mapping.gen_range.start && offset <= mapping.gen_range.end)
}

fn map_generated_offset_to_source(
    mapping: &vize_canon::virtual_ts::VizeMapping,
    generated_offset: usize,
) -> usize {
    let generated_relative = generated_offset.saturating_sub(mapping.gen_range.start);
    let source_len = mapping
        .src_range
        .end
        .saturating_sub(mapping.src_range.start);
    mapping
        .src_range
        .start
        .saturating_add(generated_relative.min(source_len.saturating_sub(1)))
}

pub(super) fn line_character_to_byte_offset(
    text: &str,
    line: u32,
    character: u32,
) -> Option<usize> {
    let mut current_line = 0u32;
    let mut line_start = 0usize;

    for (offset, ch) in text.char_indices() {
        if current_line == line {
            break;
        }
        if ch == '\n' {
            current_line += 1;
            line_start = offset + ch.len_utf8();
        }
    }

    if current_line != line {
        return None;
    }

    let line_text = text[line_start..]
        .split_once('\n')
        .map(|(line, _)| line)
        .unwrap_or(&text[line_start..]);
    let mut utf16_units = 0u32;
    for (relative_offset, ch) in line_text.char_indices() {
        if utf16_units == character {
            return Some(line_start + relative_offset);
        }

        let next_utf16_units = utf16_units + ch.len_utf16() as u32;
        if character < next_utf16_units {
            return None;
        }
        utf16_units = next_utf16_units;
    }

    (utf16_units == character).then_some(line_start + line_text.len())
}

pub(super) fn source_offset_to_position(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut character = 0u32;
    let target = offset.min(source.len());

    for (current, ch) in source.char_indices() {
        if current >= target {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
    }

    (line, character)
}
