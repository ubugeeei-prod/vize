//! Relocate a template Atelier map into the final SFC module.

use serde_json::{Value, json};
use vize_carton::{FxHashMap, String, ToCompactString};

use super::super::super::{compile_template::TemplateBlockCompileResult, types::SfcTemplateBlock};
use super::super::output_module::OutputRange;
use super::vlq::{MappingSegment, decode_mappings, encode_mappings};

/// Owned source-map material retained while SFC script/template assembly runs.
#[derive(Debug, Clone)]
pub(crate) struct TemplateSourceMapArtifact {
    fragment: String,
    generated: String,
}

impl TemplateSourceMapArtifact {
    pub(crate) fn capture(output: &TemplateBlockCompileResult) -> Option<Self> {
        let fragment = output.source_map_fragment()?.to_compact_string();
        let generated = generated_fragment(output)?.to_compact_string();
        Some(Self {
            fragment,
            generated,
        })
    }

    /// Compose this template fragment into `final_code` and rebase its source
    /// positions from template-local coordinates to the original `.vue` file.
    pub(crate) fn compose(
        &self,
        final_code: &str,
        template: &SfcTemplateBlock<'_>,
        filename: &str,
        sfc_source: &str,
    ) -> Option<Value> {
        let mut document: Value = serde_json::from_str(self.fragment.as_str()).ok()?;
        let mappings = document.get("mappings")?.as_str()?;
        let decoded = decode_mappings(mappings)?;
        let mut relocated =
            GeneratedLocator::new(self.generated.as_str(), final_code).relocate(decoded.as_slice());
        if relocated.is_empty() {
            return None;
        }

        let (source_line, source_column) = line_column_at(sfc_source, template.loc.start);
        for segment in &mut relocated {
            let Some(original) = segment.original.as_mut() else {
                continue;
            };
            if original.source != 0 {
                continue;
            }
            if original.line == 0 {
                original.column += source_column as i64;
            }
            original.line += source_line as i64;
        }

        relocated.sort_by_key(|segment| (segment.generated_line, segment.generated_column));
        document["file"] = json!(filename);
        document["sources"] = json!([filename]);
        document["sourcesContent"] = json!([sfc_source]);
        document["mappings"] = json!(encode_mappings(relocated.as_slice()).as_str());
        Some(document)
    }
}

fn generated_fragment(output: &TemplateBlockCompileResult) -> Option<&str> {
    let range = output
        .module_sections
        .map(|sections| sections.functions)
        .unwrap_or(OutputRange::new(0, output.code.len()));
    output.code.get(range.start..range.end)
}

#[derive(Debug, Clone, Copy)]
struct LinePlacement {
    target_line: usize,
    source_indent: usize,
    target_indent: usize,
}

struct GeneratedLocator<'a> {
    source: &'a str,
    target: &'a str,
    source_lines: Vec<&'a str>,
    target_lines: Vec<&'a str>,
    target_line_starts: Vec<usize>,
    exact_lines: FxHashMap<usize, LinePlacement>,
    last_target_offset: usize,
}

impl<'a> GeneratedLocator<'a> {
    fn new(source: &'a str, target: &'a str) -> Self {
        Self {
            source,
            target,
            source_lines: source.split('\n').collect(),
            target_lines: target.split('\n').collect(),
            target_line_starts: line_starts(target),
            exact_lines: FxHashMap::default(),
            last_target_offset: 0,
        }
    }

    fn relocate(mut self, segments: &[MappingSegment]) -> Vec<MappingSegment> {
        let mut relocated = Vec::with_capacity(segments.len());
        for segment in segments {
            let Some((line, column, offset)) = self.locate(segment) else {
                continue;
            };
            self.last_target_offset = self.last_target_offset.max(offset);
            relocated.push(MappingSegment {
                generated_line: line,
                generated_column: column,
                original: segment.original,
            });
        }
        relocated
    }

    fn locate(&mut self, segment: &MappingSegment) -> Option<(usize, usize, usize)> {
        let source_line = *self.source_lines.get(segment.generated_line)?;
        if let Some(placement) = self.exact_line(segment.generated_line, source_line) {
            let relative = segment
                .generated_column
                .saturating_sub(placement.source_indent);
            let column = placement.target_indent + relative;
            let target_line = self.target_lines.get(placement.target_line)?;
            let byte_column = byte_at_utf16_column(target_line, column);
            let offset = self.target_line_starts[placement.target_line] + byte_column;
            return Some((placement.target_line, column, offset));
        }

        let source_byte = byte_at_utf16_column(source_line, segment.generated_column);
        let source_line_start = line_start_offset(self.source, segment.generated_line)?;
        let anchor_start = source_line_start + source_byte;
        let target_offset = find_anchor(
            self.source,
            anchor_start,
            self.target,
            self.last_target_offset,
        )?;
        let (line, column) = line_column_at(self.target, target_offset);
        Some((line, column, target_offset))
    }

    fn exact_line(&mut self, source_index: usize, source_line: &str) -> Option<LinePlacement> {
        if let Some(placement) = self.exact_lines.get(&source_index) {
            return Some(*placement);
        }
        let source_trimmed = source_line.trim();
        if source_trimmed.is_empty() {
            return None;
        }
        let minimum_line =
            line_at_offset(self.target_line_starts.as_slice(), self.last_target_offset);
        let target_index = self
            .target_lines
            .iter()
            .enumerate()
            .skip(minimum_line)
            .find_map(|(index, line)| (line.trim() == source_trimmed).then_some(index))?;
        let placement = LinePlacement {
            target_line: target_index,
            source_indent: utf16_len(source_line) - utf16_len(source_line.trim_start()),
            target_indent: utf16_len(self.target_lines[target_index])
                - utf16_len(self.target_lines[target_index].trim_start()),
        };
        self.exact_lines.insert(source_index, placement);
        Some(placement)
    }
}

fn find_anchor(source: &str, start: usize, target: &str, minimum: usize) -> Option<usize> {
    let tail = source.get(start..)?.split('\n').next()?;
    let mut boundaries: Vec<usize> = tail.char_indices().map(|(index, _)| index).collect();
    boundaries.push(tail.len());
    for &length in boundaries.iter().rev() {
        if !(8..=64).contains(&length) {
            continue;
        }
        let anchor = tail.get(..length)?.trim_end();
        if anchor.len() < 8 {
            continue;
        }
        if let Some(relative) = target.get(minimum..)?.find(anchor) {
            return Some(minimum + relative);
        }
    }
    None
}

fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    starts.extend(
        source
            .bytes()
            .enumerate()
            .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
    );
    starts
}

fn line_start_offset(source: &str, line: usize) -> Option<usize> {
    line_starts(source).get(line).copied()
}

fn line_at_offset(starts: &[usize], offset: usize) -> usize {
    starts
        .partition_point(|start| *start <= offset)
        .saturating_sub(1)
}

fn line_column_at(source: &str, offset: usize) -> (usize, usize) {
    let offset = floor_char_boundary(source, offset.min(source.len()));
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line = source[..line_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    (line, utf16_len(&source[line_start..offset]))
}

fn byte_at_utf16_column(line: &str, column: usize) -> usize {
    let mut utf16 = 0usize;
    for (index, ch) in line.char_indices() {
        if utf16 >= column {
            return index;
        }
        utf16 += ch.len_utf16();
        if utf16 > column {
            return index;
        }
    }
    line.len()
}

fn utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}

fn floor_char_boundary(value: &str, mut offset: usize) -> usize {
    while offset > 0 && !value.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::super::vlq::{OriginalPosition, encode_mappings};
    use super::*;
    use crate::types::BlockLocation;
    use std::borrow::Cow;

    #[test]
    fn relocates_render_lines_and_rebases_to_the_sfc() {
        let generated = "export function render() {\n  return _ctx.msg\n}";
        let segment = MappingSegment {
            generated_line: 1,
            generated_column: 9,
            original: Some(OriginalPosition {
                source: 0,
                line: 0,
                column: 7,
                name: None,
            }),
        };
        let artifact = TemplateSourceMapArtifact {
            fragment: json!({
                "version": 3,
                "sources": ["template.vue"],
                "sourcesContent": ["<p>{{ msg }}</p>"],
                "names": [],
                "mappings": encode_mappings(&[segment]).as_str(),
            })
            .to_string()
            .into(),
            generated: generated.into(),
        };
        let source = "<script>export default {}</script>\n<template><p>{{ msg }}</p></template>";
        let content_start = source.find("<p>").unwrap();
        let template = SfcTemplateBlock {
            content: Cow::Borrowed("<p>{{ msg }}</p>"),
            loc: BlockLocation {
                start: content_start,
                end: content_start + 18,
                ..Default::default()
            },
            lang: None,
            src: None,
            attrs: Default::default(),
        };
        let final_code = "const _sfc_main = {}\nfunction _sfc_render() {\n  return _ctx.msg\n}";
        let map = artifact
            .compose(final_code, &template, "Probe.vue", source)
            .unwrap();
        let relocated = decode_mappings(map["mappings"].as_str().unwrap()).unwrap();

        assert_eq!(
            (relocated[0].generated_line, relocated[0].generated_column),
            (2, 9)
        );
        assert_eq!(relocated[0].original.unwrap().line, 1);
        assert_eq!(map["sources"], json!(["Probe.vue"]));
        assert_eq!(map["sourcesContent"], json!([source]));
    }
}

#[cfg(test)]
#[path = "composition_tests.rs"]
mod integration_tests;
