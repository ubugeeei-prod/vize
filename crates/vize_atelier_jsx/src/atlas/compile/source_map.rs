//! Source Map v3 assembly from graph-backend Rendu mappings.

use serde_json::json;
use vize_atelier_dom::RenduDomMapping;
use vize_carton::String;

pub(super) fn from_dom_mappings(
    code: &str,
    body_start: usize,
    mappings: &[RenduDomMapping],
    filename: &str,
    source: &str,
) -> String {
    let mut segments = mappings
        .iter()
        .filter(|mapping| mapping.generated_start >= body_start)
        .map(|mapping| {
            let generated = line_column_at(code, mapping.generated_start - body_start);
            Segment {
                generated_line: generated.0,
                generated_column: generated.1,
                source_line: mapping.source.start.line.saturating_sub(1) as usize,
                source_column: mapping.source.start.column.saturating_sub(1) as usize,
            }
        })
        .collect::<Vec<_>>();
    segments.sort_by_key(|segment| (segment.generated_line, segment.generated_column));
    segments.dedup_by_key(|segment| (segment.generated_line, segment.generated_column));
    serde_json::to_string(&json!({
        "version": 3,
        "file": filename,
        "sources": [filename],
        "sourcesContent": [source],
        "names": [],
        "mappings": encode(&segments).as_str(),
    }))
    .unwrap_or_default()
    .into()
}

struct Segment {
    generated_line: usize,
    generated_column: usize,
    source_line: usize,
    source_column: usize,
}

fn encode(segments: &[Segment]) -> String {
    let mut output = String::default();
    let (mut line, mut column, mut source_line, mut source_column) = (0, 0, 0, 0);
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
        encode_vlq(segment.generated_column as i64 - column, &mut output);
        encode_vlq(0, &mut output);
        encode_vlq(segment.source_line as i64 - source_line, &mut output);
        encode_vlq(segment.source_column as i64 - source_column, &mut output);
        column = segment.generated_column as i64;
        source_line = segment.source_line as i64;
        source_column = segment.source_column as i64;
    }
    output
}

fn line_column_at(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());
    let start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line = source[..start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    (line, source[start..offset].encode_utf16().count())
}

fn encode_vlq(value: i64, output: &mut String) {
    let mut encoded = value.unsigned_abs() << 1 | u64::from(value < 0);
    loop {
        let mut digit = (encoded & 31) as u8;
        encoded >>= 5;
        if encoded != 0 {
            digit |= 32;
        }
        output.push(base64(digit) as char);
        if encoded == 0 {
            break;
        }
    }
}

fn base64(value: u8) -> u8 {
    match value {
        0..=25 => b'A' + value,
        26..=51 => b'a' + value - 26,
        52..=61 => b'0' + value - 52,
        62 => b'+',
        _ => b'/',
    }
}
