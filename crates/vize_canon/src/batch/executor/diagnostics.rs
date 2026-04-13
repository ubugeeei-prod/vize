use std::path::{Path, PathBuf};

use serde_json::Value;

use super::super::{Diagnostic, OriginalPosition, VirtualFile, VirtualProject};
use crate::corsa_client::LspDiagnostic;
use crate::file_uri::file_uri_to_path;
use vize_carton::{FxHashMap, String};

pub(super) fn map_batch_diagnostics(
    results: Vec<(String, Vec<LspDiagnostic>)>,
    project: &VirtualProject,
) -> Vec<Diagnostic> {
    let diagnostic_count = results
        .iter()
        .fold(0usize, |acc, (_, diagnostics)| acc + diagnostics.len());
    let mut diagnostics = Vec::with_capacity(diagnostic_count);
    let mut mapper = DiagnosticMapper::new(project);

    for (uri, lsp_diagnostics) in results {
        let virtual_path = uri_to_path(uri.as_str());
        for diagnostic in lsp_diagnostics {
            if let Some(diagnostic) = mapper.map_lsp_diagnostic(&virtual_path, diagnostic) {
                diagnostics.push(diagnostic);
            }
        }
    }

    diagnostics
}

struct DiagnosticMapper<'a> {
    project: &'a VirtualProject,
    original_sources: FxHashMap<PathBuf, CachedSource>,
    virtual_line_indexes: FxHashMap<PathBuf, LineIndex>,
}

impl<'a> DiagnosticMapper<'a> {
    fn new(project: &'a VirtualProject) -> Self {
        Self {
            project,
            original_sources: FxHashMap::default(),
            virtual_line_indexes: FxHashMap::default(),
        }
    }

    fn map_lsp_diagnostic(
        &mut self,
        virtual_path: &Path,
        diagnostic: LspDiagnostic,
    ) -> Option<Diagnostic> {
        let code = parse_diagnostic_code(diagnostic.code.as_ref());
        if should_skip_diagnostic(code) {
            return None;
        }

        let original = self.map_to_original(
            virtual_path,
            diagnostic.range.start.line,
            diagnostic.range.start.character,
        );

        if let Some(original) = original {
            return Some(Diagnostic {
                file: original.path,
                line: original.line,
                column: original.column,
                message: diagnostic.message,
                code,
                severity: parse_severity(diagnostic.severity),
                block_type: original.block_type,
            });
        }

        None
    }

    fn map_to_original(
        &mut self,
        virtual_path: &Path,
        line: u32,
        column: u32,
    ) -> Option<OriginalPosition> {
        let file = self.project.find_by_virtual(virtual_path)?;
        let virtual_offset = self.virtual_offset(file, line, column)?;
        let (original_offset, _, block_type) =
            file.source_map.get_original_position(virtual_offset)?;
        let cached = self.original_source(&file.original_path)?;
        let (original_line, original_column) = cached
            .line_index
            .offset_to_line_col(&cached.content, original_offset)?;

        Some(OriginalPosition {
            path: file.original_path.clone(),
            line: original_line,
            column: original_column,
            block_type,
        })
    }

    fn virtual_offset(&mut self, file: &VirtualFile, line: u32, column: u32) -> Option<u32> {
        if !self.virtual_line_indexes.contains_key(&file.virtual_path) {
            self.virtual_line_indexes
                .insert(file.virtual_path.clone(), LineIndex::new(&file.content));
        }
        let line_index = self.virtual_line_indexes.get(&file.virtual_path)?;
        line_index.line_col_to_offset(&file.content, line, column)
    }

    fn original_source(&mut self, path: &Path) -> Option<&CachedSource> {
        if !self.original_sources.contains_key(path) {
            let content: String = std::fs::read_to_string(path).ok()?.into();
            let line_index = LineIndex::new(&content);
            self.original_sources.insert(
                path.to_path_buf(),
                CachedSource {
                    content,
                    line_index,
                },
            );
        }

        self.original_sources.get(path)
    }
}

struct CachedSource {
    content: String,
    line_index: LineIndex,
}

struct LineIndex {
    starts: Vec<usize>,
    len: usize,
}

impl LineIndex {
    fn new(content: &str) -> Self {
        let mut starts = vec![0];
        for (index, byte) in content.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(index + 1);
            }
        }

        Self {
            starts,
            len: content.len(),
        }
    }

    fn line_col_to_offset(&self, content: &str, line: u32, col: u32) -> Option<u32> {
        let line = usize::try_from(line).ok()?;
        let start = *self.starts.get(line)?;
        let end = self.line_end(line);
        let mut current_col = 0u32;
        let mut offset = start;

        if col == 0 {
            return u32::try_from(offset).ok();
        }

        for ch in content[start..end].chars() {
            offset += ch.len_utf8();
            current_col += 1;
            if current_col == col {
                return u32::try_from(offset).ok();
            }
        }

        if current_col == col {
            u32::try_from(offset).ok()
        } else {
            None
        }
    }

    fn offset_to_line_col(&self, content: &str, offset: u32) -> Option<(u32, u32)> {
        let offset = usize::try_from(offset).ok()?;
        if offset > self.len {
            return None;
        }

        let line = self.starts.partition_point(|start| *start <= offset);
        let line = line.saturating_sub(1);
        let start = *self.starts.get(line)?;
        let end = self.line_end(line);
        let mut col = 0usize;
        let mut cursor = start;
        for ch in content[start..end].chars() {
            if cursor >= offset {
                break;
            }
            col += 1;
            cursor += ch.len_utf8();
        }
        Some((u32::try_from(line).ok()?, u32::try_from(col).ok()?))
    }

    fn line_end(&self, line: usize) -> usize {
        self.starts
            .get(line + 1)
            .map(|next_start| next_start.saturating_sub(1))
            .unwrap_or(self.len)
    }
}

fn uri_to_path(uri: &str) -> PathBuf {
    file_uri_to_path(uri).unwrap_or_else(|| PathBuf::from(uri))
}

fn parse_diagnostic_code(code: Option<&Value>) -> Option<u32> {
    match code {
        Some(Value::Number(value)) => value.as_u64().and_then(|value| u32::try_from(value).ok()),
        Some(Value::String(value)) => value
            .strip_prefix("TS")
            .unwrap_or(value.as_str())
            .parse()
            .ok(),
        _ => None,
    }
}

fn parse_severity(severity: Option<i32>) -> u8 {
    match severity {
        Some(value) if (1..=4).contains(&value) => value as u8,
        _ => 1,
    }
}

pub(super) fn should_skip_diagnostic(code: Option<u32>) -> bool {
    matches!(
        code,
        Some(2307) | Some(2666) | Some(6133) | Some(7006) | Some(7043) | Some(7044)
    )
}

#[cfg(test)]
mod tests {
    use super::{
        map_batch_diagnostics, parse_diagnostic_code, parse_severity, uri_to_path, LineIndex,
    };
    use crate::batch::VirtualProject;
    use serde_json::json;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use vize_carton::cstr;

    #[test]
    fn parses_numeric_and_string_diagnostic_codes() {
        assert_eq!(parse_diagnostic_code(Some(&json!(2322))), Some(2322));
        assert_eq!(parse_diagnostic_code(Some(&json!("TS2304"))), Some(2304));
        assert_eq!(parse_diagnostic_code(Some(&json!("2551"))), Some(2551));
        assert_eq!(parse_diagnostic_code(Some(&json!(false))), None);
    }

    #[test]
    fn normalizes_lsp_severity() {
        assert_eq!(parse_severity(Some(1)), 1);
        assert_eq!(parse_severity(Some(2)), 2);
        assert_eq!(parse_severity(Some(9)), 1);
        assert_eq!(parse_severity(None), 1);
    }

    #[test]
    fn strips_file_uri_scheme() {
        assert_eq!(
            uri_to_path("file:///workspace/src/App.vue.ts"),
            PathBuf::from("/workspace/src/App.vue.ts")
        );
    }

    #[test]
    fn decodes_file_uri_path_bytes() {
        assert_eq!(
            uri_to_path("file:///workspace/pages/%5Bname%5D%20%231.vue.ts"),
            PathBuf::from("/workspace/pages/[name] #1.vue.ts")
        );
    }

    #[test]
    fn line_index_matches_source_map_boundaries() {
        let content = "a\nbeta\n";
        let index = LineIndex::new(content);

        assert_eq!(index.line_col_to_offset(content, 0, 1), Some(1));
        assert_eq!(index.line_col_to_offset(content, 1, 4), Some(6));
        assert_eq!(index.line_col_to_offset(content, 2, 0), Some(7));
        assert_eq!(index.line_col_to_offset(content, 1, 5), None);
        assert_eq!(index.offset_to_line_col(content, 7), Some((2, 0)));

        let content = "é\n";
        let index = LineIndex::new(content);
        assert_eq!(index.offset_to_line_col(content, 1), Some((0, 1)));
    }

    #[test]
    fn maps_unmapped_diagnostics_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let project = VirtualProject::new(temp_dir.path()).unwrap();
        let diagnostics = map_batch_diagnostics(
            vec![(
                cstr!("file:///workspace/src/App.vue.ts"),
                vec![crate::corsa_client::LspDiagnostic {
                    range: crate::corsa_client::LspRange {
                        start: crate::corsa_client::LspPosition {
                            line: 3,
                            character: 5,
                        },
                        end: crate::corsa_client::LspPosition {
                            line: 3,
                            character: 12,
                        },
                    },
                    severity: Some(1),
                    code: Some(json!("TS2322")),
                    source: Some("ts".into()),
                    message: "Type 'string' is not assignable to type 'number'.".into(),
                }],
            )],
            &project,
        );

        insta::assert_debug_snapshot!("maps_unmapped_diagnostics_snapshot", diagnostics);
    }
}
