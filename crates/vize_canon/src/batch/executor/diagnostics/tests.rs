#![expect(
    clippy::disallowed_macros,
    reason = "insta snapshots expand to format!"
)]
use super::{LineIndex, map_batch_diagnostics, parse_diagnostic_code, parse_severity, uri_to_path};
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
