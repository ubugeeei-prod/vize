//! Independent Source Map v3 decoder for TS-31.
//!
//! Deliberately not shared with the encoder under test: a map that decodes to
//! the wrong position is exactly the failure TS-31 measures, so the harness
//! decodes every segment itself and resolves both sides back to byte offsets.

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// One decoded segment, both sides resolved to byte offsets.
///
/// A side is `None` when its (line, UTF-16 column) does not land on a
/// character boundary inside the text it indexes; such a segment can never be
/// byte-exact and is not attributed to any authored span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    pub generated: Option<usize>,
    pub source: Option<usize>,
}

/// Validate the map document and decode its mappings against `generated` and
/// `source`.
///
/// The document must be a single-source v3 map whose `sources` names
/// `filename` and whose `sourcesContent` embeds the authored source exactly.
pub fn decode(
    map: &serde_json::Value,
    generated: &str,
    source: &str,
    filename: &str,
) -> Vec<Segment> {
    assert_eq!(
        (&map["version"], &map["sources"], &map["sourcesContent"]),
        (
            &serde_json::json!(3),
            &serde_json::json!([filename]),
            &serde_json::json!([source]),
        ),
        "TS-31 maps are single-source v3 documents embedding the authored text"
    );
    let mappings = map["mappings"].as_str().expect("`mappings` is a string");
    let generated_lines = line_starts(generated);
    let source_lines = line_starts(source);

    let mut segments = Vec::new();
    let (mut source_index, mut source_line, mut source_column) = (0i64, 0i64, 0i64);
    for (generated_line, group) in mappings.split(';').enumerate() {
        let mut generated_column = 0i64;
        for field in group.split(',').filter(|field| !field.is_empty()) {
            let values = decode_field(field);
            generated_column += values[0];
            if values.len() == 1 {
                continue;
            }
            assert!(
                values.len() == 4 || values.len() == 5,
                "segment `{field}` has {} fields",
                values.len()
            );
            source_index += values[1];
            source_line += values[2];
            source_column += values[3];
            assert_eq!(source_index, 0, "single-source maps only");
            segments.push(Segment {
                generated: resolve(
                    generated,
                    &generated_lines,
                    generated_line as i64,
                    generated_column,
                ),
                source: resolve(source, &source_lines, source_line, source_column),
            });
        }
    }
    segments
}

fn decode_field(field: &str) -> Vec<i64> {
    let mut values = Vec::with_capacity(5);
    let mut result: u64 = 0;
    let mut shift = 0u32;
    for byte in field.bytes() {
        let digit = BASE64
            .iter()
            .position(|&candidate| candidate == byte)
            .unwrap_or_else(|| panic!("invalid base64 VLQ digit in `{field}`"))
            as u64;
        result |= (digit & 0b1_1111) << shift;
        shift += 5;
        if digit & 0b10_0000 == 0 {
            let magnitude = (result >> 1) as i64;
            values.push(if result & 1 == 1 {
                -magnitude
            } else {
                magnitude
            });
            result = 0;
            shift = 0;
        }
    }
    assert_eq!(shift, 0, "truncated VLQ in `{field}`");
    values
}

fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(index, _)| index + 1))
        .collect()
}

/// Resolve a 0-based line and UTF-16 column to a byte offset.
fn resolve(text: &str, starts: &[usize], line: i64, column: i64) -> Option<usize> {
    let line_start = *starts.get(usize::try_from(line).ok()?)?;
    let line_end = text[line_start..]
        .find('\n')
        .map_or(text.len(), |index| line_start + index);
    let mut units = 0i64;
    for (offset, ch) in text[line_start..line_end].char_indices() {
        if units == column {
            return Some(line_start + offset);
        }
        units += ch.len_utf16() as i64;
    }
    (units == column).then_some(line_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_absolute_segments_and_skips_generated_only_fields() {
        let map = serde_json::json!({
            "version": 3,
            "sources": ["a.vue"],
            "sourcesContent": ["ab\ncd"],
            "names": [],
            // gen (0,0)->src(0,1); gen (0,2) generated-only; gen (1,1)->src(1,0)
            "mappings": "AAAC,E;CACD",
        });
        assert_eq!(
            decode(&map, "xyz\nuvw", "ab\ncd", "a.vue"),
            vec![
                Segment {
                    generated: Some(0),
                    source: Some(1)
                },
                Segment {
                    generated: Some(5),
                    source: Some(3)
                },
            ]
        );
    }

    #[test]
    fn utf16_columns_resolve_to_char_boundaries_or_none() {
        let text = "a\u{1D7D8}b";
        let starts = line_starts(text);
        assert_eq!(
            (0..=4)
                .map(|column| resolve(text, &starts, 0, column))
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1), None, Some(5), Some(6)]
        );
    }
}
