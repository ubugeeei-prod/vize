use std::fs;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

use crate::ide::IdeContext;
use crate::ide::definition::DefinitionService;
use crate::server::ServerState;

#[test]
fn the_importing_statement_is_matched_by_offset_and_binding() {
    let content = "import { a } from \"./a\";\nimport { useCounter, type User } from \"../composables/useCounter\";\n";
    let offset = content.find("useCounter").unwrap() + 2;
    assert_eq!(
        super::importing_specifier(content, offset, "useCounter"),
        Some((
            "../composables/useCounter".to_owned(),
            "useCounter".to_owned()
        )),
    );
    // Same statement, different word: `type User` binds too.
    let offset = content.find("User").unwrap();
    assert_eq!(
        super::importing_specifier(content, offset, "User"),
        Some(("../composables/useCounter".to_owned(), "User".to_owned())),
    );
    // Outside any import statement: no match.
    assert_eq!(
        super::importing_specifier(content, content.len() - 1, "useCounter"),
        None
    );
}

#[test]
fn renamed_default_and_namespace_imports_bind() {
    // The alias is local; the target declares the source name.
    assert_eq!(
        super::bound_source_name("import { long as short } from", "short").as_deref(),
        Some("long"),
    );
    assert_eq!(
        super::bound_source_name("import { long as short } from", "long"),
        None
    );
    assert_eq!(
        super::bound_source_name("import { type User as U } from", "U").as_deref(),
        Some("User"),
    );
    assert_eq!(
        super::bound_source_name("import Default, { x } from", "Default").as_deref(),
        Some("Default"),
    );
    assert_eq!(
        super::bound_source_name("import * as ns from", "ns").as_deref(),
        Some("ns")
    );
}

#[test]
fn tag_imports_resolve_through_aliases_anywhere_in_the_file() {
    let content = "import { Widget as LocalWidget } from \"@/comps\";\n";
    assert_eq!(
        super::bound_import(content, "LocalWidget"),
        Some(("@/comps".to_owned(), "Widget".to_owned())),
    );
    assert_eq!(super::bound_import(content, "Widget"), None);
}

#[test]
fn component_tag_import_target_deleted_detection_preserves_open_buffers() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let target_path = workspace.path().join("LazyChild.vue");
    let importer_path = workspace.path().join("Parent.vue");
    let source = r#"<script setup lang="ts">
import LazyChild from "./LazyChild.vue";
</script>
<template><LazyChild /></template>
"#;
    fs::write(&importer_path, source).expect("importer");

    let importer_uri = Url::from_file_path(&importer_path).expect("importer URI");
    let target_uri = Url::from_file_path(&target_path).expect("target URI");
    let state = ServerState::new();
    state
        .documents
        .open(importer_uri.clone(), source.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&importer_uri, source);
    let offset = source.rfind("LazyChild").expect("component tag");
    let ctx = IdeContext::new(&state, &importer_uri, offset).expect("IDE context");

    assert!(super::component_tag_import_target_is_deleted(&ctx));
    assert!(super::component_tag_definition(&ctx).is_none());

    state
        .documents
        .open(target_uri, "<template />\n".to_owned(), 1, "vue".to_owned());
    assert!(!super::component_tag_import_target_is_deleted(&ctx));
    assert!(super::component_tag_definition(&ctx).is_some());
}

#[test]
fn reexports_cover_named_renames_and_stars() {
    let barrel =
        "export { default as UiButton } from \"./UiButton.vue\";\nexport * from \"./tokens\";\n";
    // `default` has no locatable declaration name, so the hop keeps the
    // requested name.
    assert_eq!(
        super::reexport_specifier(barrel, "UiButton"),
        Some(("./UiButton.vue".to_owned(), "UiButton".to_owned())),
    );
    assert_eq!(
        super::reexport_specifier(barrel, "anything"),
        Some(("./tokens".to_owned(), "anything".to_owned())),
    );
    // A renaming barrel hop continues under the source name.
    let renaming = "export { Widget as LocalWidget } from \"./Widget\";\n";
    assert_eq!(
        super::reexport_specifier(renaming, "LocalWidget"),
        Some(("./Widget".to_owned(), "Widget".to_owned())),
    );
}

#[test]
fn nuxt_reference_aliases_return_normalized_component_uris() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let nuxt = workspace.path().join(".nuxt");
    let pages = workspace.path().join("app/pages");
    let components = workspace.path().join("app/components");
    fs::create_dir_all(&nuxt).expect("Nuxt directory");
    fs::create_dir_all(&pages).expect("pages directory");
    fs::create_dir_all(&components).expect("components directory");
    fs::write(
        workspace.path().join("tsconfig.json"),
        r#"{"references":[{"path":"./.nuxt/tsconfig.app.json"}],"files":[]}"#,
    )
    .expect("solution config");
    fs::write(
        nuxt.join("tsconfig.app.json"),
        r#"{"compilerOptions":{"paths":{"~/*":["../app/*"]}}}"#,
    )
    .expect("Nuxt app config");

    let target_path = components.join("AccountSearchResult.vue");
    fs::write(&target_path, "<template />\n").expect("component");
    let importer_path = pages.join("accounts.vue");
    let source = r#"<script setup lang="ts">
import AccountSearchResult from '~/components/AccountSearchResult.vue'
</script>
<template><AccountSearchResult /></template>
"#;
    fs::write(&importer_path, source).expect("importer");

    let importer_uri = Url::from_file_path(&importer_path).expect("importer URI");
    let state = ServerState::new();
    state
        .documents
        .open(importer_uri.clone(), source.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&importer_uri, source);
    let offset = source.rfind("AccountSearchResult").expect("component tag");
    let ctx = IdeContext::new(&state, &importer_uri, offset).expect("IDE context");
    let definition = super::component_tag_definition(&ctx).expect("component definition");
    let GotoDefinitionResponse::Scalar(location) = definition else {
        panic!("component definition must be scalar");
    };

    assert_eq!(
        location
            .uri
            .to_file_path()
            .expect("definition path")
            .canonicalize()
            .expect("canonical definition path"),
        target_path.canonicalize().expect("canonical target path")
    );
}

#[test]
fn definition_service_follows_an_alias_barrel_to_the_component_source() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let source_dir = workspace.path().join("src");
    let components = source_dir.join("components");
    fs::create_dir_all(&components).expect("components directory");
    fs::write(
        workspace.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}}}"#,
    )
    .expect("TypeScript config");
    fs::write(
        components.join("index.ts"),
        "export { Widget } from \"./Widget.vue\";\n",
    )
    .expect("component barrel");
    let target_path = components.join("Widget.vue");
    fs::write(&target_path, "<template><button /></template>\n").expect("component");

    let importer_path = source_dir.join("App.vue");
    let source = r#"<script setup lang="ts">
import { Widget } from "@/components";
</script>
<template><Widget /></template>
"#;
    fs::write(&importer_path, source).expect("importer");

    let importer_uri = Url::from_file_path(&importer_path).expect("importer URI");
    let state = ServerState::new();
    state.set_workspace_root(workspace.path().to_path_buf());
    state
        .documents
        .open(importer_uri.clone(), source.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&importer_uri, source);
    let offset = source.rfind("Widget").expect("component tag");
    let ctx = IdeContext::new(&state, &importer_uri, offset).expect("IDE context");
    let definition = DefinitionService::definition(&ctx).expect("component definition");
    let GotoDefinitionResponse::Scalar(location) = definition else {
        panic!("component definition must be scalar");
    };

    assert_eq!(
        location
            .uri
            .to_file_path()
            .expect("definition path")
            .canonicalize()
            .expect("canonical definition path"),
        target_path.canonicalize().expect("canonical target path")
    );
}

#[test]
fn component_definition_resolves_lower_camel_import_as_kebab_tag() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let component_dir = workspace.path().join("descriptionItem");
    fs::create_dir_all(&component_dir).expect("component directory");
    let target_path = component_dir.join("index.vue");
    fs::write(&target_path, "<template><dl /></template>\n").expect("component");

    let importer_path = workspace.path().join("App.vue");
    let source = r#"<script setup lang="ts">
import descriptionItem from "./descriptionItem/index.vue";
</script>
<template><description-item title="Full Name" /></template>
"#;
    fs::write(&importer_path, source).expect("importer");

    let importer_uri = Url::from_file_path(&importer_path).expect("importer URI");
    let state = ServerState::new();
    state
        .documents
        .open(importer_uri.clone(), source.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&importer_uri, source);
    let offset = source.rfind("description-item").expect("component tag");
    let ctx = IdeContext::new(&state, &importer_uri, offset).expect("IDE context");
    let definition = super::component_tag_definition(&ctx).expect("component definition");
    let GotoDefinitionResponse::Scalar(location) = definition else {
        panic!("component definition must be scalar");
    };

    assert_eq!(
        location
            .uri
            .to_file_path()
            .expect("definition path")
            .canonicalize()
            .expect("canonical definition path"),
        target_path.canonicalize().expect("canonical target path")
    );
}

#[test]
fn component_definition_resolves_pug_kebab_import_tag() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let target_path = workspace.path().join("HighlightMessage.vue");
    fs::write(&target_path, "<template><p /></template>\n").expect("component");

    let importer_path = workspace.path().join("App.vue");
    let source = r#"<script setup lang="ts">
import HighlightMessage from "./HighlightMessage.vue";
</script>
<template lang="pug">
  highlight-message(type="success")
</template>
"#;
    fs::write(&importer_path, source).expect("importer");

    let importer_uri = Url::from_file_path(&importer_path).expect("importer URI");
    let state = ServerState::new();
    state
        .documents
        .open(importer_uri.clone(), source.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&importer_uri, source);
    let offset = source.rfind("highlight-message").expect("component tag");
    let ctx = IdeContext::new(&state, &importer_uri, offset).expect("IDE context");
    let definition = super::component_tag_definition(&ctx).expect("component definition");
    let GotoDefinitionResponse::Scalar(location) = definition else {
        panic!("component definition must be scalar");
    };

    assert_eq!(
        location
            .uri
            .to_file_path()
            .expect("definition path")
            .canonicalize()
            .expect("canonical definition path"),
        target_path.canonicalize().expect("canonical target path")
    );
}

#[test]
#[cfg(unix)]
fn definition_service_follows_a_workspace_package_vue_export() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let app = workspace.path().join("app");
    let package = workspace.path().join("packages/ui");
    let target_path = package.join("src/Widget.vue");
    fs::create_dir_all(target_path.parent().unwrap()).unwrap();
    fs::write(
        package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{"./widget":"./src/Widget.vue"}}"#,
    )
    .unwrap();
    fs::write(&target_path, "<template><button /></template>\n").unwrap();
    let package_link = app.join("node_modules/@scope/ui");
    crate::ide::tests::symlink_dir(&package, &package_link);

    let importer_path = app.join("src/App.vue");
    let source = r#"<script setup lang="ts">
import Widget from "@scope/ui/widget";
</script>
<template><Widget /></template>
"#;
    fs::create_dir_all(importer_path.parent().unwrap()).unwrap();
    fs::write(&importer_path, source).unwrap();
    let importer_uri = Url::from_file_path(&importer_path).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(app.clone());
    state
        .documents
        .open(importer_uri.clone(), source.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&importer_uri, source);
    let offset = source.rfind("Widget").unwrap();
    let ctx = IdeContext::new(&state, &importer_uri, offset).unwrap();
    let GotoDefinitionResponse::Scalar(location) =
        DefinitionService::definition(&ctx).expect("component definition")
    else {
        panic!("component definition must be scalar");
    };

    assert_eq!(
        location.uri.to_file_path().unwrap().canonicalize().unwrap(),
        target_path.canonicalize().unwrap()
    );
}
