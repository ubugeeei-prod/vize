//! Native editor type-checking regressions.

use super::editor_typecheck_fixture::{
    assert_no_import_or_unknown_record_diagnostics, resolve_test_tsgo_binary, state_for_fixture,
    write_art_vue_import_fixture, write_corsa_config, write_speaker_fixture,
    write_vue_import_fixture,
};
use super::{DiagnosticService, sources};
use tower_lsp::lsp_types::Url;
use vize_s0::cstr;

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
#[cfg(unix)]
fn async_collect_resolves_workspace_package_vue_exports() {
    assert_workspace_package_vue_export_resolves("./src/Widget.vue", None);
}

#[test]
#[cfg(unix)]
fn async_collect_resolves_workspace_package_vue_exports_through_a_ts_barrel() {
    assert_workspace_package_vue_export_resolves(
        "./src/index.ts",
        Some("export { default } from './Widget.vue';\n"),
    );
}

#[test]
#[cfg(unix)]
fn async_collect_recovers_when_a_workspace_package_is_created_after_open() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let root = project.path();
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();
    super::editor_typecheck_fixture::write_vue_test_package(root);
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["src/**/*"]}"#,
    )
    .unwrap();
    let parent_path = src.join("Parent.vue");
    let source = r#"<script setup lang="ts">
import Widget from "@scope/ui/widget";
type IsAny<T> = 0 extends 1 & T ? true : false;
const componentMustBeTyped: IsAny<typeof Widget> = true;
void componentMustBeTyped;
</script>
"#;
    std::fs::write(&parent_path, source).unwrap();
    write_corsa_config(root, &corsa_path);
    let uri = Url::from_file_path(&parent_path).unwrap();
    let state = state_for_fixture(root, &uri, source);
    state.load_workspace_config(root);

    let unresolved = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));
    assert!(
        unresolved.iter().any(|diagnostic| {
            diagnostic.message.contains("Cannot find module")
                || diagnostic.code == Some(tower_lsp::lsp_types::NumberOrString::Number(2307))
        }),
        "the regression must begin unresolved: {unresolved:#?}",
    );

    let package = root.join("packages/ui");
    std::fs::create_dir_all(package.join("src")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{"./widget":"./src/Widget.vue"}}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("src/Widget.vue"),
        "<script setup lang=\"ts\">defineProps<{ label: string }>()</script>\n",
    )
    .unwrap();
    link_workspace_package(&package, &root.join("node_modules/@scope/ui"));

    let resolved = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));
    assert!(
        resolved
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("Cannot find module")),
        "created package must resolve in the same editor session: {resolved:#?}",
    );
    assert!(resolved.iter().any(|diagnostic| {
        diagnostic.code == Some(tower_lsp::lsp_types::NumberOrString::Number(2322))
    }));
}

#[cfg(unix)]
fn assert_workspace_package_vue_export_resolves(entry: &str, barrel: Option<&str>) {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let root = project.path();
    let src = root.join("src");
    let package = root.join("packages/ui");
    std::fs::create_dir_all(&src).expect("app src");
    std::fs::create_dir_all(package.join("src")).expect("package src");
    super::editor_typecheck_fixture::write_vue_test_package(root);
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .expect("tsconfig");
    std::fs::write(
        package.join("package.json"),
        cstr!(r#"{{"name":"@scope/ui","exports":{{"./widget":"{entry}"}}}}"#),
    )
    .expect("package manifest");
    if let Some(barrel) = barrel {
        std::fs::write(package.join("src/index.ts"), barrel).expect("package barrel");
    }
    std::fs::write(
        package.join("src/Widget.vue"),
        "<script setup lang=\"ts\">defineProps<{ label: string }>()</script>\n",
    )
    .expect("component");
    link_workspace_package(&package, &root.join("node_modules/@scope/ui"));
    let parent_path = src.join("Parent.vue");
    let source = r#"<script setup lang="ts">
import Widget from "@scope/ui/widget";
type IsAny<T> = 0 extends 1 & T ? true : false;
const componentMustBeTyped: IsAny<typeof Widget> = true;
void componentMustBeTyped;
</script>
"#;
    std::fs::write(&parent_path, source).expect("parent source");
    write_corsa_config(root, &corsa_path);

    let uri = Url::from_file_path(&parent_path).expect("file uri");
    let state = state_for_fixture(root, &uri, source);
    state.load_workspace_config(root);
    let diagnostics = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));
    assert!(
        diagnostics.iter().all(|diagnostic| {
            !diagnostic.message.contains("Cannot find module")
                && !matches!(
                    diagnostic.code,
                    Some(tower_lsp::lsp_types::NumberOrString::Number(2307 | 2882))
                )
        }),
        "workspace package Vue export must resolve in the editor: {diagnostics:#?}",
    );
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == Some(tower_lsp::lsp_types::NumberOrString::Number(2322))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        errors.len(),
        1,
        "a resolved component must not degrade to any: {diagnostics:#?}"
    );
    assert_eq!(errors[0].range.start.line, 3);
}

#[cfg(unix)]
fn link_workspace_package(source: &std::path::Path, target: &std::path::Path) {
    crate::ide::tests::symlink_dir(source, target);
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
