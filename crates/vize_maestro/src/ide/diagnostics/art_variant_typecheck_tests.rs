//! End-to-end editor type checking for every `.art.vue` variant (#4015).
//!
//! A single generated document per art file only ever type-checked the default
//! variant, so an authored type error in any other variant was silent.

use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url};

use super::editor_typecheck_fixture::{
    resolve_test_tsgo_binary, state_for_fixture, write_corsa_config, write_vue_test_package,
};
use super::{DiagnosticService, sources};

const ART_SOURCE: &str = r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });

function format(value: string, precision: number): string {
  return value.slice(0, precision);
}
</script>

<art>
  <variant name="Primary" default>
    <Button :label="format('primary', 2)" />
  </variant>
  <variant name="Secondary">
    <Button :label="format('secondary', 'two')" />
  </variant>
</art>
"#;

const REPAIRED_ART_SOURCE: &str = r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });

function format(value: string, precision: number): string {
  return value.slice(0, precision);
}
</script>

<art>
  <variant name="Primary" default>
    <Button :label="format('primary', 2)" />
  </variant>
  <variant name="Secondary">
    <Button :label="format('secondary', 3)" />
  </variant>
</art>
"#;

fn write_art_project(root: &Path, source: &str) -> PathBuf {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("src dir");
    write_vue_test_package(root);
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .expect("tsconfig");
    std::fs::write(
        src.join("Button.vue"),
        r#"<script setup lang="ts">
defineProps<{ label?: string }>();
</script>

<template>
  <button>{{ label }}</button>
</template>
"#,
    )
    .expect("button vue");
    let art_path = src.join("Button.art.vue");
    std::fs::write(&art_path, source).expect("art vue");
    art_path
}

fn collect(root: &Path, art_path: &Path, source: &str) -> Vec<Diagnostic> {
    let uri = Url::from_file_path(art_path).expect("file uri");
    let state = state_for_fixture(root, &uri, source);
    state.load_workspace_config(root);
    crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri))
}

fn type_diagnostics(diagnostics: &[Diagnostic]) -> Vec<(Option<NumberOrString>, String, Range)> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.source.as_deref() == Some(sources::TYPE_CHECKER))
        .map(|diagnostic| {
            (
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                diagnostic.range,
            )
        })
        .collect()
}

fn range(start_line: u32, start_character: u32, end_line: u32, end_character: u32) -> Range {
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

#[test]
fn async_collect_reports_the_type_error_in_a_non_default_art_variant() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let art_path = write_art_project(project.path(), ART_SOURCE);
    write_corsa_config(project.path(), &corsa_path);

    let diagnostics = collect(project.path(), &art_path, ART_SOURCE);

    assert_eq!(
        type_diagnostics(&diagnostics),
        vec![(
            Some(NumberOrString::Number(2345)),
            "Argument of type 'string' is not assignable to parameter of type 'number'."
                .to_string(),
            range(13, 40, 13, 45),
        )],
        "full diagnostic list: {diagnostics:#?}",
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity == Some(DiagnosticSeverity::ERROR)),
        "unexpected non-error diagnostics: {diagnostics:#?}",
    );
}

#[test]
fn async_collect_keeps_a_repaired_art_variant_clean() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let art_path = write_art_project(project.path(), REPAIRED_ART_SOURCE);
    write_corsa_config(project.path(), &corsa_path);

    let diagnostics = collect(project.path(), &art_path, REPAIRED_ART_SOURCE);

    assert_eq!(
        type_diagnostics(&diagnostics),
        Vec::new(),
        "full diagnostic list: {diagnostics:#?}",
    );
}

/// Every variant document carries the same authored script, so one authored
/// script error must still be published once rather than once per variant.
#[test]
fn a_shared_script_error_is_published_once_across_variants() {
    const SHARED_SCRIPT_ERROR: &str = r#"<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });

const broken: number = "not a number";

function format(value: string, precision: number): string {
  return value.slice(0, precision);
}
</script>

<art>
  <variant name="Primary" default>
    <Button :label="format('primary', broken)" />
  </variant>
  <variant name="Secondary">
    <Button :label="format('secondary', 3)" />
  </variant>
  <variant name="Tertiary">
    <Button :label="format('tertiary', 4)" />
  </variant>
</art>
"#;

    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let art_path = write_art_project(project.path(), SHARED_SCRIPT_ERROR);
    write_corsa_config(project.path(), &corsa_path);

    let diagnostics = collect(project.path(), &art_path, SHARED_SCRIPT_ERROR);

    assert_eq!(
        type_diagnostics(&diagnostics),
        vec![(
            Some(NumberOrString::Number(2322)),
            "Type 'string' is not assignable to type 'number'.".to_string(),
            range(3, 6, 3, 12),
        )],
        "full diagnostic list: {diagnostics:#?}",
    );
}

/// Negative control for the per-variant documents themselves: an art file with
/// no imports still produces one document per variant, and the extra documents
/// must not redeclare each other's setup bindings inside the same project.
#[test]
fn extra_variant_documents_do_not_collide_in_one_project() {
    const PLAIN_ART_SOURCE: &str = r#"<script setup lang="ts">
const label: string = "plain";
</script>

<art title="Plain">
  <variant name="Primary" default>
    <span>{{ label }}</span>
  </variant>
  <variant name="Secondary">
    <span>{{ label.toUpperCase() }}</span>
  </variant>
  <variant name="Tertiary">
    <span>{{ label.trim() }}</span>
  </variant>
</art>
"#;

    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let art_path = write_art_project(project.path(), PLAIN_ART_SOURCE);
    write_corsa_config(project.path(), &corsa_path);

    let diagnostics = collect(project.path(), &art_path, PLAIN_ART_SOURCE);

    assert_eq!(
        type_diagnostics(&diagnostics),
        Vec::new(),
        "full diagnostic list: {diagnostics:#?}",
    );
}

/// The cached variant documents must follow the buffer: an edit that repairs
/// the non-default variant has to clear its diagnostic in the same session.
#[test]
fn editing_a_non_default_variant_refreshes_its_typed_document() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let art_path = write_art_project(project.path(), ART_SOURCE);
    write_corsa_config(project.path(), &corsa_path);

    let uri = Url::from_file_path(&art_path).expect("file uri");
    let state = state_for_fixture(project.path(), &uri, ART_SOURCE);
    state.load_workspace_config(project.path());

    let before = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));
    assert_eq!(
        type_diagnostics(&before),
        vec![(
            Some(NumberOrString::Number(2345)),
            "Argument of type 'string' is not assignable to parameter of type 'number'."
                .to_string(),
            range(13, 40, 13, 45),
        )],
        "full diagnostic list: {before:#?}",
    );

    state.documents.open(
        uri.clone(),
        REPAIRED_ART_SOURCE.to_string(),
        2,
        "vue".to_string(),
    );
    let after = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));

    assert_eq!(
        type_diagnostics(&after),
        Vec::new(),
        "full diagnostic list: {after:#?}",
    );
}
