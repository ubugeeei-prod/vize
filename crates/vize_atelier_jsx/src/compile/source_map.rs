//! Relocate one VDOM component map into its assembled JSX module.

use serde_json::{Value, json};
use vize_carton::String;

#[derive(Clone, Copy)]
struct OriginalPosition {
    source: i64,
    line: i64,
    column: i64,
    name: Option<i64>,
}

#[derive(Clone, Copy)]
struct MappingSegment {
    generated_line: usize,
    generated_column: usize,
    original: Option<OriginalPosition>,
}

pub(super) fn relocate_fragment_map(
    fragment: &str,
    module_code: &str,
    generated_start: usize,
    filename: &str,
    source: &str,
) -> Option<String> {
    let mut document: Value = serde_json::from_str(fragment).ok()?;
    let mappings = document.get("mappings")?.as_str()?;
    let mut segments = decode_mappings(mappings)?;
    let (line_offset, column_offset) = line_column_at(module_code, generated_start);
    for segment in &mut segments {
        if segment.generated_line == 0 {
            segment.generated_column += column_offset;
        }
        segment.generated_line += line_offset;
    }
    document["file"] = json!(filename);
    document["sources"] = json!([filename]);
    document["sourcesContent"] = json!([source]);
    document["mappings"] = json!(encode_mappings(segments.as_slice()).as_str());
    serde_json::to_string(&document).ok().map(String::from)
}

fn line_column_at(source: &str, offset: usize) -> (usize, usize) {
    let mut offset = offset.min(source.len());
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line = source[..line_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    (line, source[line_start..offset].encode_utf16().count())
}

fn decode_mappings(mappings: &str) -> Option<Vec<MappingSegment>> {
    let mut output = Vec::new();
    let (mut source, mut source_line, mut source_column, mut name) = (0, 0, 0, 0);
    for (generated_line, line) in mappings.split(';').enumerate() {
        let mut generated_column = 0i64;
        for encoded in line.split(',').filter(|segment| !segment.is_empty()) {
            let fields = decode_segment(encoded)?;
            if !matches!(fields.len(), 1 | 4 | 5) {
                return None;
            }
            generated_column += fields[0];
            if generated_column < 0 {
                return None;
            }
            let original = if fields.len() == 1 {
                None
            } else {
                source += fields[1];
                source_line += fields[2];
                source_column += fields[3];
                if source < 0 || source_line < 0 || source_column < 0 {
                    return None;
                }
                let mapped_name = if fields.len() == 5 {
                    name += fields[4];
                    (name >= 0).then_some(name)?
                } else {
                    -1
                };
                Some(OriginalPosition {
                    source,
                    line: source_line,
                    column: source_column,
                    name: (mapped_name >= 0).then_some(mapped_name),
                })
            };
            output.push(MappingSegment {
                generated_line,
                generated_column: generated_column as usize,
                original,
            });
        }
    }
    Some(output)
}

fn encode_mappings(segments: &[MappingSegment]) -> String {
    let mut output = String::default();
    let (mut line, mut column, mut source, mut source_line, mut source_column, mut name) =
        (0, 0, 0, 0, 0, 0);
    let mut first = true;
    for segment in segments {
        while line < segment.generated_line {
            output.push(';');
            line += 1;
            column = 0;
            first = true;
        }
        if !first {
            output.push(',');
        }
        first = false;
        let next_column = segment.generated_column as i64;
        encode_value(next_column - column, &mut output);
        column = next_column;
        if let Some(original) = segment.original {
            encode_value(original.source - source, &mut output);
            encode_value(original.line - source_line, &mut output);
            encode_value(original.column - source_column, &mut output);
            source = original.source;
            source_line = original.line;
            source_column = original.column;
            if let Some(next_name) = original.name {
                encode_value(next_name - name, &mut output);
                name = next_name;
            }
        }
    }
    output
}

fn decode_segment(encoded: &str) -> Option<Vec<i64>> {
    let bytes = encoded.as_bytes();
    let mut cursor = 0;
    let mut values = Vec::with_capacity(5);
    while cursor < bytes.len() {
        let (mut value, mut shift) = (0u64, 0u32);
        loop {
            let digit = decode_digit(*bytes.get(cursor)?)?;
            cursor += 1;
            value |= u64::from(digit & 31) << shift;
            if digit & 32 == 0 {
                break;
            }
            shift += 5;
            if shift >= 64 {
                return None;
            }
        }
        let magnitude = (value >> 1) as i64;
        values.push(if value & 1 == 1 {
            -magnitude
        } else {
            magnitude
        });
    }
    Some(values)
}

fn decode_digit(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn encode_value(value: i64, output: &mut String) {
    let mut encoded = value.unsigned_abs() << 1;
    if value < 0 {
        encoded |= 1;
    }
    loop {
        let mut digit = (encoded & 31) as u8;
        encoded >>= 5;
        if encoded != 0 {
            digit |= 32;
        }
        output.push(encode_digit(digit) as char);
        if encoded == 0 {
            break;
        }
    }
}

fn encode_digit(digit: u8) -> u8 {
    match digit {
        0..=25 => b'A' + digit,
        26..=51 => b'a' + digit - 26,
        52..=61 => b'0' + digit - 52,
        62 => b'+',
        _ => b'/',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relocates_first_line_columns_after_a_non_newline_prefix() {
        let map = serde_json::to_string(&json!({
            "version": 3,
            "mappings": "AAAA",
            "sources": ["template.vue"],
        }))
        .unwrap();
        let relocated = relocate_fragment_map(&map, "prefix code", 7, "App.jsx", "<App />")
            .expect("valid source map");
        let document: Value = serde_json::from_str(relocated.as_str()).unwrap();
        assert_eq!(document["mappings"], "OAAA");
    }
}
