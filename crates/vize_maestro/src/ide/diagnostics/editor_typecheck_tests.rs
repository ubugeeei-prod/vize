//! Native editor type-checking regressions.

use super::editor_typecheck_fixture::{
    assert_no_import_or_unknown_record_diagnostics, resolve_test_tsgo_binary, state_for_fixture,
    write_art_vue_import_fixture, write_corsa_config, write_speaker_fixture,
    write_vue_import_fixture,
};
use super::{DiagnosticService, sources};
use tower_lsp::lsp_types::Url;

#[test]
fn sync_collect_does_not_surface_legacy_type_false_positives() {
    let project = tempfile::TempDir::new().expect("temp project");
    let fixture = write_speaker_fixture(project.path());
    let uri = Url::from_file_path(&fixture.vue_path).expect("file uri");
    let state = state_for_fixture(project.path(), &uri, &fixture.source);

    let diagnostics = DiagnosticService::collect(&state, &uri);

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.source.as_deref() != Some(sources::TYPE_CHECKER)),
        "sync native diagnostics must not run the legacy type checker: {diagnostics:#?}",
    );
    assert_no_import_or_unknown_record_diagnostics(&diagnostics);
}

#[test]
fn async_collect_preserves_imported_computed_callback_types() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let fixture = write_speaker_fixture(project.path());
    write_corsa_config(project.path(), &corsa_path);

    let uri = Url::from_file_path(&fixture.vue_path).expect("file uri");
    let state = state_for_fixture(project.path(), &uri, &fixture.source);
    state.load_workspace_config(project.path());

    let diagnostics = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));

    assert_no_import_or_unknown_record_diagnostics(&diagnostics);
    assert!(
        diagnostics.is_empty(),
        "expected clean editor diagnostics, got: {diagnostics:#?}",
    );
}

#[test]
fn async_collect_resolves_relative_vue_imports_in_script_setup() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    write_vue_import_fixture(project.path());
    write_corsa_config(project.path(), &corsa_path);

    let parent_path = project.path().join("src/Parent.vue");
    let source = std::fs::read_to_string(&parent_path).expect("parent source");
    let uri = Url::from_file_path(&parent_path).expect("file uri");
    let state = state_for_fixture(project.path(), &uri, &source);
    state.load_workspace_config(project.path());

    let diagnostics = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("Cannot find module")),
        "relative .vue imports must resolve via editor virtual mirrors: {diagnostics:#?}",
    );
    assert!(
        diagnostics.is_empty(),
        "expected clean editor diagnostics, got: {diagnostics:#?}",
    );
}

#[test]
fn async_collect_resolves_define_art_target_vue_imports() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    write_art_vue_import_fixture(project.path());
    write_corsa_config(project.path(), &corsa_path);

    let art_path = project.path().join("src/Button.art.vue");
    let source = std::fs::read_to_string(&art_path).expect("art source");
    let uri = Url::from_file_path(&art_path).expect("file uri");
    let state = state_for_fixture(project.path(), &uri, &source);
    state.load_workspace_config(project.path());

    let diagnostics = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("Cannot find module")),
        "defineArt target .vue import must resolve via editor virtual mirrors: {diagnostics:#?}",
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("number")
                && diagnostic.message.contains("string")),
        "expected the resolved target component prop type to be checked: {diagnostics:#?}",
    );
}

#[test]
fn virtual_ts_generates_template_less_sfc_mirror() {
    let uri = Url::parse("file:///tmp/SpeakerFilterBar.vue").expect("parse uri");
    let source = r#"<script setup lang="ts">
defineProps<{ selected: string }>();
</script>
"#;

    let result = DiagnosticService::generate_virtual_ts(&uri, source, false, false)
        .expect("virtual TS generated for template-less SFC");

    assert!(
        result.code.contains("export default __vize_component__;"),
        "expected a component module mirror, got:\n{}",
        result.code,
    );
}
