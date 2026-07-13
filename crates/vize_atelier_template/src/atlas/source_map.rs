use serde_json::{Value, json};
use vize_carton::String;

use super::TemplateOutputMapping;

pub(super) fn source_map(
    filename: &str,
    source: &str,
    code: &str,
    mappings: &[TemplateOutputMapping],
) -> Value {
    let mut segments = mappings
        .iter()
        .map(|mapping| {
            let (generated_line, generated_column) = line_column(code, mapping.generated_start);
            let (source_line, source_column) = line_column(source, mapping.source_start as usize);
            Segment {
                generated_line,
                generated_column,
                source_line,
                source_column,
            }
        })
        .collect::<Vec<_>>();
    segments.sort_by_key(|segment| (segment.generated_line, segment.generated_column));
    segments.dedup_by_key(|segment| (segment.generated_line, segment.generated_column));
    json!({
        "version": 3,
        "file": filename,
        "sources": [filename],
        "sourcesContent": [source],
        "names": [],
        "mappings": encode(&segments),
    })
}

#[derive(Clone, Copy)]
struct Segment {
    generated_line: usize,
    generated_column: usize,
    source_line: usize,
    source_column: usize,
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut offset = offset.min(source.len());
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line = source[..line_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    (line, offset - line_start)
}

fn encode(segments: &[Segment]) -> String {
    let mut output = String::default();
    let mut line = 0;
    let mut previous_source_line = 0_i64;
    let mut previous_source_column = 0_i64;
    let mut previous_generated_column = 0_i64;
    let mut first_on_line = true;
    for segment in segments {
        while line < segment.generated_line {
            output.push(';');
            line += 1;
            first_on_line = true;
            previous_generated_column = 0;
        }
        if !first_on_line {
            output.push(',');
        }
        first_on_line = false;
        let generated_column = segment.generated_column as i64;
        push_vlq(&mut output, generated_column - previous_generated_column);
        previous_generated_column = generated_column;
        push_vlq(&mut output, 0);
        let source_line = segment.source_line as i64;
        push_vlq(&mut output, source_line - previous_source_line);
        previous_source_line = source_line;
        let source_column = segment.source_column as i64;
        push_vlq(&mut output, source_column - previous_source_column);
        previous_source_column = source_column;
    }
    output
}

fn push_vlq(output: &mut String, value: i64) {
    const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut value = if value < 0 {
        ((-value) as u64) << 1 | 1
    } else {
        (value as u64) << 1
    };
    loop {
        let mut digit = (value & 31) as usize;
        value >>= 5;
        if value != 0 {
            digit |= 32;
        }
        output.push(BASE64[digit] as char);
        if value == 0 {
            break;
        }
    }
}
