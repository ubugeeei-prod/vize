//! Independent source-map decoder and compilation fixtures.

use crate::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor_with_experimental_options,
};
use vize_carton::Allocator;

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Which Vapor lane a test compile must take.
#[derive(Clone, Copy)]
pub(super) enum Lane {
    /// Whatever the compile selects (the native S3 lane when it admits the
    /// template).
    Selected,
    /// The legacy lowering lane: binding metadata keeps the S3 bridge out.
    Legacy,
}

fn compile_on(source: &str, source_map: bool, lane: Lane) -> crate::VaporCompileResult {
    let allocator = Allocator::new();
    let result = compile_vapor_with_experimental_options(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            binding_metadata: matches!(lane, Lane::Legacy).then(Default::default),
            ..VaporCompilerOptions::default()
        },
        VaporCompilerExperimentalOptions {
            source_map,
            source_map_filename: Some("Foo.vue".into()),
            ..VaporCompilerExperimentalOptions::default()
        },
    );
    assert_eq!(
        result.error_messages.len(),
        0,
        "{:?}",
        result.error_messages
    );
    result
}

/// Absolute `(generated line, generated column, source line, source column,
/// name index)` for every segment.
fn decode(mappings: &str) -> std::vec::Vec<(usize, i64, i64, i64, Option<i64>)> {
    let mut segments = std::vec::Vec::new();
    let (mut source_line, mut source_column, mut name) = (0i64, 0i64, 0i64);
    for (line, group) in mappings.split(';').enumerate() {
        let mut column = 0i64;
        for field in group.split(',').filter(|field| !field.is_empty()) {
            let mut values = std::vec::Vec::new();
            let (mut value, mut shift) = (0u64, 0u32);
            for byte in field.bytes() {
                let digit = BASE64.iter().position(|&c| c == byte).unwrap() as u64;
                value |= (digit & 31) << shift;
                shift += 5;
                if digit & 32 == 0 {
                    let magnitude = (value >> 1) as i64;
                    values.push(if value & 1 == 1 {
                        -magnitude
                    } else {
                        magnitude
                    });
                    (value, shift) = (0, 0);
                }
            }
            column += values[0];
            source_line += values[2];
            source_column += values[3];
            let named = values.get(4).map(|delta| {
                name += delta;
                name
            });
            segments.push((line, column, source_line, source_column, named));
        }
    }
    segments
}

fn offset(text: &str, line: usize, column: i64) -> usize {
    let start: usize = text.split_inclusive('\n').take(line).map(str::len).sum();
    start + column as usize
}

fn render(code: &str, source: &str, map: &str) -> std::vec::Vec<std::string::String> {
    let map: serde_json::Value = serde_json::from_str(map).unwrap();
    let names = map["names"].as_array().unwrap();
    let window = |text: &str, at: usize| text[at..(at + 8).min(text.len())].replace('\n', "⏎");
    decode(map["mappings"].as_str().unwrap())
        .into_iter()
        .map(|(line, column, source_line, source_column, name)| {
            let generated = window(code, offset(code, line, column));
            let authored = window(source, offset(source, source_line as usize, source_column));
            let name = name.map_or(std::string::String::new(), |index| {
                std::format!(" [{}]", names[index as usize].as_str().unwrap())
            });
            std::format!("{line}:{column} {generated:?} -> {authored:?}{name}")
        })
        .collect()
}

/// Compile with and without a map, require identical output, and render the
/// map's segments.
pub(super) fn mapped(source: &str) -> (std::string::String, std::vec::Vec<std::string::String>) {
    mapped_on(source, Lane::Selected)
}

pub(super) fn mapped_on(
    source: &str,
    lane: Lane,
) -> (std::string::String, std::vec::Vec<std::string::String>) {
    let without_map = compile_on(source, false, lane);
    let with_map = compile_on(source, true, lane);
    assert_eq!(
        (&with_map.code, &with_map.templates),
        (&without_map.code, &without_map.templates)
    );
    let map = with_map.map.expect("source_map should attach Vapor map");
    let segments = render(&with_map.code, source, &map);
    (with_map.code.as_str().into(), segments)
}
