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
mod tests {
    use super::{
        LineIndex, map_batch_diagnostics, parse_diagnostic_code, parse_severity, uri_to_path,
    };
    use crate::batch::{SfcBlockType, VirtualProject};
    use serde_json::json;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use vize_carton::cstr;

    /// Interpolations now have one authored read. The collection boundary still
    /// collapses duplicate backend findings at the same authored location.
    #[test]
    fn duplicated_template_diagnostic_is_reported_once() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().canonicalize().unwrap();
        let src_dir = project_root.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        let app_path = src_dir.join("App.vue");
        std::fs::write(
            &app_path,
            "<script setup lang=\"ts\">\n</script>\n<template><div>{{ missingThing }}</div></template>\n",
        )
        .unwrap();

        let mut project = VirtualProject::new(&project_root).unwrap();
        project.register_path(&app_path).unwrap();
        let virtual_file = project.find_by_original(&app_path).unwrap();
        let virtual_source = virtual_file.content.as_str();

        // Each authored interpolation is emitted once. A backend can still
        // report the same diagnostic twice; mapping must deduplicate it.
        let message = "Cannot find name 'missingThing'.";
        let mut diagnostics: Vec<_> = virtual_source
            .match_indices("void (missingThing)")
            .map(|(at, _)| at + "void (".len())
            .map(|offset| {
                let (line, character) = LineIndex::new(virtual_source)
                    .offset_to_line_col(virtual_source, offset as u32)
                    .expect("virtual offset should map to LSP position");
                crate::corsa_client::LspDiagnostic {
                    range: crate::corsa_client::LspRange {
                        start: crate::corsa_client::LspPosition { line, character },
                        end: crate::corsa_client::LspPosition {
                            line,
                            character: character + "missingThing".len() as u32,
                        },
                    },
                    severity: Some(1),
                    code: Some(json!("TS2304")),
                    source: Some("ts".into()),
                    message: message.into(),
                }
            })
            .collect();

        assert_eq!(
            diagnostics.len(),
            1,
            "an interpolation should not create duplicate unresolved reads"
        );
        diagnostics.push(diagnostics[0].clone());

        let mapped = map_batch_diagnostics(
            vec![(file_uri_for(&virtual_file.virtual_path), diagnostics)],
            &project,
        );

        assert_eq!(
            mapped.len(),
            1,
            "the duplicated template diagnostic must be deduplicated: {mapped:#?}"
        );
        assert_eq!(mapped[0].file, app_path);
        assert_eq!(mapped[0].code, Some(2339));
    }

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
    fn preserves_ts2307_even_when_a_source_sibling_exists_on_disk() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().canonicalize().unwrap();
        let src_dir = project_root.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let app_path = src_dir.join("App.vue");
        std::fs::write(
            &app_path,
            r#"<script setup lang="ts">
import ExistingPanel from './ExistingPanel.vue'
</script>

<template>
  <ExistingPanel />
</template>
"#,
        )
        .unwrap();
        std::fs::write(
            src_dir.join("ExistingPanel.vue"),
            r#"<template><section /></template>
"#,
        )
        .unwrap();

        let mut project = VirtualProject::new(&project_root).unwrap();
        project.register_path(&app_path).unwrap();
        let virtual_file = project.find_by_original(&app_path).unwrap();
        let diagnostic = ts2307_diagnostic_at(
            virtual_file.content.as_str(),
            "ExistingPanel.vue.ts",
            "Cannot find module './ExistingPanel.vue.ts' or its corresponding type declarations.",
        );

        let diagnostics = map_batch_diagnostics(
            vec![(file_uri_for(&virtual_file.virtual_path), vec![diagnostic])],
            &project,
        );

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].code, Some(2307));
        assert!(diagnostics[0].message.contains("'./ExistingPanel.vue'"));
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

    #[test]
    fn maps_user_ts6133_but_suppresses_generated_vue_ts6133() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("src").join("App.vue");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(
            temp_dir.path().join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "noUnusedLocals": true
  },
  "include": ["src/**/*"]
}"#,
        )
        .unwrap();
        std::fs::write(
            &source,
            r#"<script setup lang="ts">
const used = 1
const unusedLocal = 2
</script>

<template>{{ used }}</template>
"#,
        )
        .unwrap();

        let source = source.canonicalize().unwrap();
        let mut project = VirtualProject::new(temp_dir.path()).unwrap();
        project.register_path(&source).unwrap();
        let virtual_file = project.find_by_original(&source).unwrap();
        let virtual_uri = file_uri_for(&virtual_file.virtual_path);

        let diagnostics = map_batch_diagnostics(
            vec![(
                virtual_uri,
                vec![
                    ts6133_diagnostic_at(
                        virtual_file.content.as_str(),
                        "unusedLocal",
                        "'unusedLocal' is declared but its value is never read.",
                    ),
                    ts6133_diagnostic_at(
                        virtual_file.content.as_str(),
                        "const defineProps",
                        "'defineProps' is declared but its value is never read.",
                    ),
                ],
            )],
            &project,
        );

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].file, source);
        assert_eq!(diagnostics[0].line, 2);
        assert_eq!(diagnostics[0].column, 6);
        assert_eq!(diagnostics[0].code, Some(6133));
        assert!(diagnostics[0].message.contains("unusedLocal"));
        assert_eq!(diagnostics[0].block_type, Some(SfcBlockType::ScriptSetup));
    }

    fn ts2307_diagnostic_at(
        virtual_source: &str,
        needle: &str,
        message: &str,
    ) -> crate::corsa_client::LspDiagnostic {
        let (line, character) = virtual_position_for(virtual_source, needle);
        crate::corsa_client::LspDiagnostic {
            range: crate::corsa_client::LspRange {
                start: crate::corsa_client::LspPosition { line, character },
                end: crate::corsa_client::LspPosition {
                    line,
                    character: character + 1,
                },
            },
            severity: Some(1),
            code: Some(json!("TS2307")),
            source: Some("ts".into()),
            message: message.into(),
        }
    }

    fn ts6133_diagnostic_at(
        virtual_source: &str,
        needle: &str,
        message: &str,
    ) -> crate::corsa_client::LspDiagnostic {
        let (line, character) = virtual_position_for(virtual_source, needle);
        crate::corsa_client::LspDiagnostic {
            range: crate::corsa_client::LspRange {
                start: crate::corsa_client::LspPosition { line, character },
                end: crate::corsa_client::LspPosition { line, character },
            },
            severity: Some(1),
            code: Some(json!("TS6133")),
            source: Some("ts".into()),
            message: message.into(),
        }
    }

    fn virtual_position_for(virtual_source: &str, needle: &str) -> (u32, u32) {
        let offset = virtual_source
            .find(needle)
            .unwrap_or_else(|| panic!("expected virtual source to contain {needle:?}"));
        LineIndex::new(virtual_source)
            .offset_to_line_col(virtual_source, offset as u32)
            .expect("virtual offset should map to LSP position")
    }

    fn file_uri_for(path: &std::path::Path) -> vize_carton::String {
        cstr!("file://{}", path.display())
    }
}
