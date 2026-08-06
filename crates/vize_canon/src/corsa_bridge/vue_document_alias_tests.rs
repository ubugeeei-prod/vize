//! Alias-resolved editor sessions (#3900).
//!
//! The mirror an alias rewrite points into is materialized from the *buffers*
//! the session is holding, not from disk: an import typed into an unsaved file
//! is reachable to nothing on disk, so a disk-only mirror never generates its
//! target and the rewritten specifier keeps an unresolvable alias path until
//! the user saves.

use super::vue_document::{
    CorsaVueVirtualDocumentOptions, build_vue_virtual_project,
    build_vue_virtual_project_with_overlays,
};

const TSCONFIG: &str = "{\n  \"compilerOptions\": {\n    \"baseUrl\": \".\",\n    \"paths\": { \"@/*\": [\"./src/*\"] }\n  }\n}\n";

const UI_BUTTON: &str = "<script setup lang=\"ts\">\ndefineProps<{ variant: \"ghost\" | \"primary\" }>();\n</script>\n<template><button /></template>\n";

/// A temp project with the alias config and the aliased component on disk.
fn alias_project() -> tempfile::TempDir {
    let project = tempfile::TempDir::new().expect("temp project");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src dir");
    std::fs::write(project.path().join("tsconfig.json"), TSCONFIG).expect("tsconfig");
    std::fs::write(src.join("UiButton.vue"), UI_BUTTON).expect("component");
    project
}

fn mirror_companion(project: &tempfile::TempDir) -> std::path::PathBuf {
    project
        .path()
        .join("node_modules")
        .join(".vize")
        .join("canon")
        .join("src")
        .join("UiButton.vue.ts")
}

#[test]
fn an_alias_import_typed_into_the_host_buffer_resolves_to_the_mirror() {
    let project = alias_project();
    let host_path = project.path().join("src").join("Host.vue");
    // On disk the host imports nothing…
    std::fs::write(
        &host_path,
        "<script setup lang=\"ts\"></script>\n<template><span /></template>\n",
    )
    .expect("host");
    // …the unsaved buffer imports the aliased component.
    let buffer = "<script setup lang=\"ts\">\nimport UiButton from \"@/UiButton.vue\";\nconst _button = UiButton;\n</script>\n<template><UiButton variant=\"ghost\" /></template>\n";

    let virtual_project = build_vue_virtual_project(
        &host_path,
        buffer,
        CorsaVueVirtualDocumentOptions::default(),
    )
    .expect("virtual project");

    assert!(
        !virtual_project.host.code.contains("@/UiButton"),
        "an unsaved alias import must not keep its unresolvable alias path:\n{}",
        virtual_project.host.code,
    );
    assert!(
        virtual_project.host.code.contains("/.vize/canon/"),
        "an unsaved alias import must resolve into the materialized mirror:\n{}",
        virtual_project.host.code,
    );
    assert!(
        mirror_companion(&project).is_file(),
        "the resolution target must exist on disk, where the checker looks",
    );
}

#[test]
fn an_alias_import_typed_into_a_dependency_buffer_resolves_to_the_mirror() {
    let project = alias_project();
    let src = project.path().join("src");
    let host_path = src.join("Host.vue");
    let child_path = src.join("Child.vue");
    let host = "<script setup lang=\"ts\">\nimport Child from \"./Child.vue\";\nconst _child = Child;\n</script>\n<template><Child /></template>\n";
    std::fs::write(&host_path, host).expect("host");
    // On disk the dependency imports nothing…
    std::fs::write(
        &child_path,
        "<script setup lang=\"ts\"></script>\n<template><span /></template>\n",
    )
    .expect("child");
    // …its unsaved buffer imports the aliased component.
    let overlays = vec![(
        child_path.clone(),
        "<script setup lang=\"ts\">\nimport UiButton from \"@/UiButton.vue\";\nconst _button = UiButton;\n</script>\n<template><UiButton variant=\"ghost\" /></template>\n",
    )];

    let virtual_project = build_vue_virtual_project_with_overlays(
        &host_path,
        host,
        CorsaVueVirtualDocumentOptions::default(),
        &overlays,
    )
    .expect("virtual project");

    let child = virtual_project
        .documents
        .iter()
        .find(|(uri, _)| uri.ends_with("Child.vue.ts"))
        .map(|(_, content)| content.as_str())
        .expect("child virtual document");
    assert!(
        !child.contains("@/UiButton"),
        "an unsaved dependency's alias import must not keep its alias path:\n{child}",
    );
    assert!(
        child.contains("/.vize/canon/"),
        "an unsaved dependency's alias import must resolve into the mirror:\n{child}",
    );
    assert!(
        mirror_companion(&project).is_file(),
        "the resolution target must exist on disk, where the checker looks",
    );
}
