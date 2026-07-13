//! Minimal Source Map v3 VLQ reader/writer used by SFC map composition.

use vize_carton::String;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct OriginalPosition {
    pub(crate) source: i64,
    pub(crate) line: i64,
    pub(crate) column: i64,
    pub(crate) name: Option<i64>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct MappingSegment {
    pub(crate) generated_line: usize,
    pub(crate) generated_column: usize,
    pub(crate) original: Option<OriginalPosition>,
}

pub(super) fn decode_mappings(mappings: &str) -> Option<Vec<MappingSegment>> {
    let mut output = Vec::new();
    let mut source = 0i64;
    let mut source_line = 0i64;
    let mut source_column = 0i64;
    let mut name = 0i64;

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

pub(crate) fn encode_mappings(segments: &[MappingSegment]) -> String {
    let mut output = String::default();
    let mut current_line = 0usize;
    let mut generated_column = 0i64;
    let mut source = 0i64;
    let mut source_line = 0i64;
    let mut source_column = 0i64;
    let mut name = 0i64;
    let mut first_on_line = true;

    for segment in segments {
        while current_line < segment.generated_line {
            output.push(';');
            current_line += 1;
            generated_column = 0;
            first_on_line = true;
        }
        if !first_on_line {
            output.push(',');
        }
        first_on_line = false;

        let next_column = segment.generated_column as i64;
        encode_value(next_column - generated_column, &mut output);
        generated_column = next_column;

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
    let mut cursor = 0usize;
    let mut values = Vec::with_capacity(5);
    while cursor < bytes.len() {
        let mut value = 0u64;
        let mut shift = 0u32;
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
        let negative = value & 1 == 1;
        let magnitude = (value >> 1) as i64;
        values.push(if negative { -magnitude } else { magnitude });
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
    fn vlq_round_trip_preserves_absolute_segments() {
        let mappings = "AAAAA,MAAM;EACNC,IAAI";
        let decoded = decode_mappings(mappings).expect("valid mappings");
        let encoded = encode_mappings(&decoded);
        assert_eq!(decode_mappings(encoded.as_str()), Some(decoded));
    }
}
