use std::path::{Path, PathBuf};

use serde_json::Value;

use super::super::{Diagnostic, OriginalPosition, VirtualFile, VirtualProject};
use crate::corsa_client::LspDiagnostic;
use crate::file_uri::file_uri_to_path;
use vize_carton::{FxHashMap, String, line_index::LineBreaks};

mod assembly;
mod dedup;
mod line_index;
mod module_specifier;
mod patterns;
mod virtual_path_message;

#[cfg(test)]
mod keyof_mapping_tests;

pub(in crate::batch::executor) use assembly::RawDiagnostic;
pub(super) use dedup::dedup_diagnostics;
use line_index::LineIndex;
use virtual_path_message::restore_authored_paths;
pub(super) use virtual_path_message::restore_authored_paths_in_messages;

/// The project-session backend: every diagnostic of the request goes through
/// the one assembly pass together.
pub(super) fn map_batch_diagnostics(
    results: Vec<(String, Vec<LspDiagnostic>)>,
    project: &VirtualProject,
) -> Vec<Diagnostic> {
    let diagnostic_count = results
        .iter()
        .fold(0usize, |acc, (_, diagnostics)| acc + diagnostics.len());
    let mut raws = Vec::with_capacity(diagnostic_count);
    for (uri, lsp_diagnostics) in results {
        let virtual_path = uri_to_path(uri.as_str());
        raws.extend(lsp_diagnostics.into_iter().map(|diagnostic| RawDiagnostic {
            virtual_path: virtual_path.clone(),
            line: diagnostic.range.start.line,
            column: diagnostic.range.start.character,
            end: Some((diagnostic.range.end.line, diagnostic.range.end.character)),
            code: parse_diagnostic_code(diagnostic.code.as_ref()),
            severity: parse_severity(diagnostic.severity),
            message: diagnostic.message,
        }));
    }
    let mut mapper = DiagnosticMapper::new(project);
    mapper.virtual_line_breaks = LineBreaks::Lsp;
    dedup_diagnostics(mapper.assemble(raws, &|_| true))
}

pub(super) struct DiagnosticMapper<'a> {
    project: &'a VirtualProject,
    preserve_unused_diagnostics: bool,
    original_sources: FxHashMap<PathBuf, CachedSource>,
    virtual_line_indexes: FxHashMap<PathBuf, LineIndex>,
    virtual_line_breaks: LineBreaks,
}

impl<'a> DiagnosticMapper<'a> {
    pub(super) fn new(project: &'a VirtualProject) -> Self {
        Self {
            project,
            preserve_unused_diagnostics: project.tsconfig_preserves_unused_diagnostics(),
            original_sources: FxHashMap::default(),
            virtual_line_indexes: FxHashMap::default(),
            virtual_line_breaks: LineBreaks::TypeScript,
        }
    }

    /// A checker message with every trace of the virtual project removed.
    ///
    /// Two independent halves of the mirroring have to be undone, and a message
    /// can carry both: the import rewriter's `./Panel.vue` -> `./Panel.vue.ts`
    /// specifier spelling, and the materialized root every path is printed from.
    fn authored_message(&mut self, original: &OriginalPosition, message: String) -> String {
        let message = self.devirtualized_module_message(original, message);
        restore_authored_paths(
            &message,
            self.project.virtual_root(),
            self.project.project_root(),
        )
    }

    /// Map a Corsa position to its authored source, honoring the JavaScript
    /// SFC `checkJs` gate (#3322).
    pub(super) fn map_to_original(
        &mut self,
        virtual_path: &Path,
        line: u32,
        column: u32,
    ) -> Option<OriginalPosition> {
        if self.project.skips_typescript_diagnostics(virtual_path) {
            return None;
        }
        let Some(file) = self.project.find_by_diagnostic_virtual(virtual_path) else {
            return self.project.diagnostic_position(virtual_path, line, column);
        };
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

    fn original_source(&mut self, path: &Path) -> Option<&CachedSource> {
        if !self.original_sources.contains_key(path) {
            let registered = self.project.find_by_original(path).and_then(|file| {
                self.project
                    .original_content_for_virtual(&file.virtual_path)
            });
            // Source maps belong to the registered snapshot. Reading the disk
            // here loses errors after an unsaved edit, or moves them onto stale
            // lines; a new in-memory document need not exist on disk at all.
            let content: String = match registered {
                Some(source) => source.into(),
                None => std::fs::read_to_string(path).ok()?.into(),
            };
            let line_index = LineIndex::for_backend(&content, self.virtual_line_breaks);
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

#[cfg(test)]
mod tests;
