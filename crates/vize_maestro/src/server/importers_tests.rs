#![allow(clippy::disallowed_methods)]

use super::{open_importers, resolve_import};
use crate::server::ServerState;
use tower_lsp::lsp_types::Url;
use vize_canon::PackageRouteResolver;

#[test]
fn index_tracks_and_removes_open_sfc_imports() {
    let dir = tempfile::tempdir().unwrap();
    let child = dir.path().join("Child.vue");
    let parent = dir.path().join("Parent.vue");
    std::fs::write(&child, "<template />").unwrap();
    std::fs::write(&parent, "<template />").unwrap();
    let child_uri = Url::from_file_path(&child).unwrap();
    let parent_uri = Url::from_file_path(&parent).unwrap();
    let state = ServerState::new();
    let source = "<script setup lang=\"ts\">import Child from './Child'</script>";

    state.update_virtual_docs(&parent_uri, source);
    assert_eq!(open_importers(&state, &child_uri), vec![parent_uri.clone()]);

    state.update_virtual_docs(&parent_uri, "<script setup>const local = 1</script>");
    assert!(open_importers(&state, &child_uri).is_empty());
}

#[test]
fn index_keeps_import_identity_across_dependency_creation_and_deletion() {
    let dir = tempfile::tempdir().unwrap();
    let child = dir.path().join("CreatedLater.vue");
    let parent = dir.path().join("Parent.vue");
    std::fs::write(&parent, "<template />").unwrap();
    let child_uri = Url::from_file_path(&child).unwrap();
    let parent_uri = Url::from_file_path(&parent).unwrap();
    let state = ServerState::new();
    let source = "<script setup lang=\"ts\">import Child from './CreatedLater.vue'</script>";

    state.update_virtual_docs(&parent_uri, source);
    assert_eq!(open_importers(&state, &child_uri), vec![parent_uri.clone()]);

    std::fs::write(&child, "<template />").unwrap();
    assert_eq!(open_importers(&state, &child_uri), vec![parent_uri.clone()]);

    std::fs::remove_file(&child).unwrap();
    assert_eq!(open_importers(&state, &child_uri), vec![parent_uri.clone()]);

    #[cfg(unix)]
    {
        let target = dir.path().join("Actual.vue");
        std::fs::write(&target, "<template />").unwrap();
        std::os::unix::fs::symlink(&target, &child).unwrap();
        assert_eq!(open_importers(&state, &child_uri), vec![parent_uri]);
    }
}

#[test]
fn index_tracks_script_module_specifier_forms() {
    let dir = tempfile::tempdir().unwrap();
    let child = dir.path().join("Child.vue");
    let consumer = dir.path().join("Consumer.tsx");
    std::fs::write(&child, "<template />").unwrap();
    std::fs::write(&consumer, "import Child from './Child.vue';\n").unwrap();
    let child_uri = Url::from_file_path(&child).unwrap();
    let consumer_uri = Url::from_file_path(&consumer).unwrap();
    let state = ServerState::new();

    for source in [
        "import Child from './Child.vue';\n",
        "export { default as Child } from './Child.vue';\n",
        "export * from './Child.vue';\n",
        "const Child = import('./Child.vue');\n",
        "const Child = require('./Child.vue');\n",
        "type Child = typeof import('./Child.vue');\n",
    ] {
        state.update_virtual_docs(&consumer_uri, source);
        assert_eq!(
            open_importers(&state, &child_uri),
            vec![consumer_uri.clone()],
            "{source}",
        );
    }
}

#[test]
fn index_resolves_explicit_script_dependencies_and_query_suffixes() {
    let dir = tempfile::tempdir().unwrap();
    let child = dir.path().join("types.ts");
    let parent = dir.path().join("Parent.vue");
    std::fs::write(&child, "export type Count = number").unwrap();
    std::fs::write(&parent, "<template />").unwrap();
    let child_uri = Url::from_file_path(&child).unwrap();
    let parent_uri = Url::from_file_path(&parent).unwrap();
    let state = ServerState::new();
    let source = "<script>import './types.ts?raw'</script>";

    state.update_virtual_docs(&parent_uri, source);
    assert_eq!(open_importers(&state, &child_uri), vec![parent_uri]);
}

#[test]
fn index_resolves_extensionless_imports_with_dotted_basenames() {
    let dir = tempfile::tempdir().unwrap();
    let dependency = dir.path().join("x.use.ts");
    let parent = dir.path().join("Parent.vue");
    std::fs::write(&dependency, "export const useX = () => 1").unwrap();
    std::fs::write(&parent, "<template />").unwrap();
    let dependency_uri = Url::from_file_path(&dependency).unwrap();
    let parent_uri = Url::from_file_path(&parent).unwrap();
    let state = ServerState::new();
    let source = "<script setup lang=\"ts\">import { useX } from './x.use'; useX()</script>";

    state.update_virtual_docs(&parent_uri, source);

    assert_eq!(open_importers(&state, &dependency_uri), vec![parent_uri]);
}

#[test]
fn index_resolves_package_export_declaration_variants() {
    let dir = tempfile::tempdir().unwrap();
    let package = dir.path().join("node_modules/vue-router");
    let parent = dir.path().join("Parent.vue");
    let module_declaration = package.join("routes.d.mts");
    let common_declaration = package.join("plugin.d.cts");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{
  "exports": {
    "./auto-routes": { "types": "./routes.d.mts" },
    "./volar/plugin": { "types": "./plugin.d.cts" }
  }
}"#,
    )
    .unwrap();
    std::fs::write(
        &module_declaration,
        "export declare const routes: unknown[]",
    )
    .unwrap();
    std::fs::write(&common_declaration, "export declare const plugin: unknown").unwrap();
    std::fs::write(&parent, "<template />").unwrap();
    let parent_uri = Url::from_file_path(&parent).unwrap();
    let module_uri = Url::from_file_path(&module_declaration).unwrap();
    let common_uri = Url::from_file_path(&common_declaration).unwrap();
    let state = ServerState::new();
    let source = r#"<script setup lang="ts">
import { routes } from 'vue-router/auto-routes'
import { plugin } from 'vue-router/volar/plugin'
void routes
void plugin
</script>"#;

    state.update_virtual_docs(&parent_uri, source);

    assert_eq!(
        open_importers(&state, &module_uri),
        vec![parent_uri.clone()]
    );
    assert_eq!(open_importers(&state, &common_uri), vec![parent_uri]);
}

#[test]
fn exact_directory_specifiers_resolve_index_files() {
    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("src");
    let source_index = source_dir.join("index.ts");
    let parent_index = dir.path().join("index.ts");
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(&source_index, "export const source = true").unwrap();
    std::fs::write(&parent_index, "export const parent = true").unwrap();
    std::fs::write(dir.path().join("src.vue"), "<template />").unwrap();
    let mut routes = PackageRouteResolver::default();

    assert_eq!(
        resolve_import(&source_dir, ".?raw", &mut routes).dependencies,
        [std::fs::canonicalize(&source_index).unwrap()]
    );
    assert_eq!(
        resolve_import(&source_dir, "..#parent", &mut routes).dependencies,
        [std::fs::canonicalize(&parent_index).unwrap()]
    );
}
