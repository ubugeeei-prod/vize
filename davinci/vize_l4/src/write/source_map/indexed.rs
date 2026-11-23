//! Reusable authored coordinates for maps emitted from one immutable source.
use super::{ResolvedSegment, SourceMapBuilder, encode_mappings, floor_char_boundary};
use crate::write::EmitDocument;
use alloc::vec::Vec;
use vize_l0::{
    SmallVec, String,
    line_index::{LineBreaks, utf16_len},
};

mod geometry;
use geometry::{STRIDE, checkpoint, prefix_units};

/// Coordinates shared by all emitted documents from the same authored text.
pub struct SourceMapIndex<'a> {
    source: &'a str,
    lines: SmallVec<[usize; 8]>,
    excess: SmallVec<[usize; 8]>,
    fields: serde_json::Value,
}

impl<'a> SourceMapIndex<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut excess = SmallVec::new();
        if !source.is_ascii() {
            let (mut before, mut saved) = (0, 0);
            for byte in (0..=source.len()).step_by(STRIDE) {
                let end = floor_char_boundary(source, byte);
                let part = source.get(before..end).unwrap_or_default();
                saved += part.len() - utf16_len(part);
                excess.push(saved);
                before = end;
            }
        }
        Self {
            source,
            lines: LineBreaks::TypeScript.line_starts(source).collect(),
            excess,
            // Match SourceMapBuilder::finish in both sorted and insertion-ordered
            // serde_json builds. Only the keys are owned by this small template.
            fields: serde_json::json!({
                "version": 3, "file": null, "sources": null,
                "sourcesContent": null, "names": null, "mappings": null,
            }),
        }
    }

    // offset is a valid scalar boundary. The difference between two prefixes
    // gives the exact UTF-16 column even if a checkpoint bisects a surrogate pair's UTF-8.
    fn prefix(&self, offset: usize) -> usize {
        if self.excess.is_empty() {
            return offset;
        }
        let slot = offset / STRIDE;
        let start = floor_char_boundary(self.source, checkpoint(offset));
        prefix_units(
            start,
            self.excess.get(slot).copied().unwrap_or(0),
            utf16_len(self.source.get(start..offset).unwrap_or_default()),
        )
    }

    fn position(&self, offset: u32) -> (u32, u32) {
        let end = floor_char_boundary(self.source, (offset as usize).min(self.source.len()));
        let line = self
            .lines
            .partition_point(|start| *start <= end)
            .saturating_sub(1);
        let start = self.lines.get(line).copied().unwrap_or(0);
        let col = if self.excess.is_empty() {
            end - start
        } else if end - start < STRIDE {
            utf16_len(self.source.get(start..end).unwrap_or_default())
        } else {
            self.prefix(end) - self.prefix(start)
        };
        (line as u32, col as u32)
    }

    /// Serialize the same v3 document and intern names in original insertion
    /// order. Several targets can share this source index across their maps.
    pub fn source_map(&self, document: &EmitDocument, filename: &str) -> String {
        let mut builder = SourceMapBuilder::new();
        document.write_segments(&mut builder);
        builder.segments.sort_by_key(|s| s.generated_offset);
        let code = document.as_str();
        let mut breaks = LineBreaks::TypeScript.line_starts(code).skip(1).peekable();
        let (mut byte, mut line, mut column) = (0, 0, 0);
        let mut rows = Vec::with_capacity(builder.segments.len());
        for segment in builder.segments {
            let target =
                floor_char_boundary(code, (segment.generated_offset as usize).min(code.len()));
            while let Some(start) = breaks.next_if(|at| *at <= target) {
                byte = start;
                line += 1;
                column = 0;
            }
            column += utf16_len(code.get(byte..target).unwrap_or_default()) as u32;
            byte = target;
            let (source_line, source_column) = self.position(segment.source_offset);
            rows.push(ResolvedSegment {
                generated_line: line,
                generated_column: column,
                source_line,
                source_column,
                name: segment.name,
            });
        }
        let mappings = encode_mappings(&rows);
        // Write borrowed strings directly to the output. A large shared
        // sourcesContent would otherwise be cloned into a temporary JSON tree.
        let encoded = || -> serde_json::Result<String> {
            let mut out = Vec::with_capacity(self.source.len() + mappings.len() + 128);
            out.push(b'{');
            if let Some(fields) = self.fields.as_object() {
                for (i, (key, placeholder)) in fields.iter().enumerate() {
                    if i != 0 {
                        out.push(b',');
                    }
                    serde_json::to_writer(&mut out, key)?;
                    out.push(b':');
                    match key.as_str() {
                        "file" => serde_json::to_writer(&mut out, filename)?,
                        "sources" => serde_json::to_writer(&mut out, &[filename])?,
                        "sourcesContent" => serde_json::to_writer(&mut out, &[self.source])?,
                        "mappings" => serde_json::to_writer(&mut out, mappings.as_str())?,
                        "names" => {
                            out.push(b'[');
                            for (i, name) in builder.names.iter().enumerate() {
                                if i != 0 {
                                    out.push(b',');
                                }
                                serde_json::to_writer(&mut out, name.as_str())?;
                            }
                            out.push(b']');
                        }
                        _ => serde_json::to_writer(&mut out, placeholder)?,
                    }
                }
            }
            out.push(b'}');
            Ok(String::from_utf8(out).unwrap_or_default())
        };
        encoded().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
