//! Shared helpers for the source-map test modules (#3399).
//!
//! The decoder is a deliberate mirror of the v3 `mappings` encoding rather than
//! a dependency: a map that exists but decodes to the wrong position is the
//! exact failure mode this feature has to rule out, so the assertions decode
//! every segment and compare full values.

use crate::parse_sfc;
use crate::types::SfcParseOptions;

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// One decoded Source Map v3 segment, with every field absolute.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Segment {
    pub(super) generated_line: usize,
    pub(super) generated_column: i64,
    pub(super) source_index: i64,
    pub(super) source_line: i64,
    pub(super) source_column: i64,
}

fn decode_vlq(bytes: &[u8], cursor: &mut usize) -> i64 {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    loop {
        let digit = BASE64.iter().position(|&c| c == bytes[*cursor]).unwrap() as u64;
        *cursor += 1;
        result |= (digit & 0b1_1111) << shift;
        shift += 5;
        if digit & 0b10_0000 == 0 {
            break;
        }
    }
    let magnitude = (result >> 1) as i64;
    if result & 1 != 0 {
        -magnitude
    } else {
        magnitude
    }
}

pub(super) fn decode_mappings(mappings: &str) -> Vec<Segment> {
    let mut decoded = Vec::new();
    let (mut source_index, mut source_line, mut source_column) = (0i64, 0i64, 0i64);

    for (generated_line, group) in mappings.split(';').enumerate() {
        let mut generated_column = 0i64;
        for field in group.split(',').filter(|field| !field.is_empty()) {
            let bytes = field.as_bytes();
            let mut cursor = 0usize;
            generated_column += decode_vlq(bytes, &mut cursor);
            source_index += decode_vlq(bytes, &mut cursor);
            source_line += decode_vlq(bytes, &mut cursor);
            source_column += decode_vlq(bytes, &mut cursor);
            decoded.push(Segment {
                generated_line,
                generated_column,
                source_index,
                source_line,
                source_column,
            });
        }
    }

    decoded
}

pub(super) fn descriptor_of(source: &str) -> crate::types::SfcDescriptor<'_> {
    parse_sfc(
        source,
        SfcParseOptions {
            filename: "/app/src/Counter.vue".into(),
            ..Default::default()
        },
    )
    .expect("fixture parses")
}
