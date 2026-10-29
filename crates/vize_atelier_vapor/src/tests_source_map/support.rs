//! Independent source-map decoder and compilation fixtures.

use crate::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor_with_experimental_options,
};
use std::{
    path::Path,
    process::{Command, Stdio},
};
use vize_carton::Allocator;

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Which Vapor lane a test compile must take.
#[derive(Clone, Copy)]
pub(super) enum Lane {
    /// Whatever the compile selects (the native L3 lane when it admits the
    /// template).
    Selected,
    /// The legacy lowering lane: binding metadata keeps the L3 bridge out.
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

fn parsed_map(map: &str) -> serde_json::Value {
    serde_json::from_str(map).unwrap()
}

fn authored_anchors(map: &str) -> std::vec::Vec<(std::string::String, i64, i64)> {
    let map = parsed_map(map);
    let names = map["names"].as_array().unwrap();
    decode(map["mappings"].as_str().unwrap())
        .into_iter()
        .filter_map(|(_, _, source_line, source_column, name)| {
            name.map(|index| {
                (
                    names[index as usize].as_str().unwrap().into(),
                    source_line,
                    source_column,
                )
            })
        })
        .collect()
}

fn render(code: &str, source: &str, map: &str) -> std::vec::Vec<std::string::String> {
    let map = parsed_map(map);
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
    let (code, segments, _) = mapped_with_anchors_on(source, lane);
    (code, segments)
}

pub(super) fn mapped_with_anchors_on(
    source: &str,
    lane: Lane,
) -> (
    std::string::String,
    std::vec::Vec<std::string::String>,
    std::vec::Vec<(std::string::String, i64, i64)>,
) {
    let without_map = compile_on(source, false, lane);
    let with_map = compile_on(source, true, lane);
    assert_eq!(
        (&with_map.code, &with_map.templates),
        (&without_map.code, &without_map.templates)
    );
    let map = with_map.map.expect("source_map should attach Vapor map");
    let segments = render(&with_map.code, source, &map);
    let anchors = authored_anchors(&map);
    (with_map.code.as_str().into(), segments, anchors)
}

pub(super) fn official_vapor_anchors(
    source: &str,
) -> std::vec::Vec<(std::string::String, i64, i64)> {
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/davinci-upstream-vapor-map.mjs");
    let mut child = Command::new("node")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect(
            "start official compiler-vapor map oracle (install workspace JS dependencies first)",
        );
    serde_json::to_writer(
        child.stdin.take().unwrap(),
        &serde_json::json!({"source": source, "filename": "Foo.vue"}),
    )
    .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "official compiler-vapor map oracle failed with {}:\n{}",
        output.status,
        std::string::String::from_utf8_lossy(&output.stderr)
    );
    authored_anchors(&std::string::String::from_utf8(output.stdout).unwrap())
}
