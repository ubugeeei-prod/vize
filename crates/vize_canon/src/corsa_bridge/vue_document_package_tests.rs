//! Importer-scoped workspace package routes in editor sessions (#4000).
#![cfg(unix)]

use std::path::{Path, PathBuf};

use super::vue_document::{CorsaVueVirtualDocumentOptions, build_vue_virtual_project};
use vize_carton::cstr;

const TSCONFIG: &str = r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler"}}"#;
const UI_BUTTON: &str = "<script setup lang=\"ts\">\ndefineProps<{ variant: \"ghost\" | \"primary\" }>();\n</script>\n<template><button /></template>\n";

#[test]
fn a_workspace_package_vue_export_resolves_to_the_external_mirror() {
    let fixture = package_fixture("ui");
    write_package_manifest(&fixture.package, "Widget.vue");
    write_component(&fixture.package, "Widget.vue", UI_BUTTON);
    link_package(&fixture.package, &fixture.link);
    let source = host_import("widget");

    let project = build(&fixture.host, &source);

    assert!(!project.host.code.contains("@scope/ui/widget"));
    let target = rewritten_package_target(&project.host.code);
    assert!(target.with_extension("vue.ts").is_file());
}

#[test]
fn package_ts_routes_through_the_mirror_while_declarations_stay_bare() {
    let fixture = package_fixture("ui");
    std::fs::write(
        fixture.package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{"./widget":"./src/index.ts","./types":"./src/index.d.ts"}}"#,
    )
    .unwrap();
    write_component(&fixture.package, "Widget.vue", UI_BUTTON);
    std::fs::write(
        fixture.package.join("src/index.ts"),
        "export { default } from './Widget.vue';\n",
    )
    .unwrap();
    std::fs::write(
        fixture.package.join("src/index.d.ts"),
        "export interface Props { label: string }\n",
    )
    .unwrap();
    link_package(&fixture.package, &fixture.link);
    let source = "<script setup lang=\"ts\">\nimport Widget from '@scope/ui/widget'\nimport type { Props } from '@scope/ui/types'\nconst props = {} as Props\nvoid Widget; void props\n</script>\n";

    let project = build(&fixture.host, source);

    assert!(source.contains("@scope/ui/widget"));
    assert!(!project.host.code.contains("@scope/ui/widget"));
    assert!(project.host.code.contains("/__vize_external__/"));
    assert!(project.host.code.contains("@scope/ui/types"));
    let barrel = project
        .documents
        .iter()
        .find(|(uri, _)| uri.ends_with("/packages/ui/src/index.ts"))
        .expect("physical package barrel is synchronized");
    assert!(barrel.1.contains("/__vize_external__/"));
    let barrel_target = rewritten_package_target(&barrel.1);
    assert!(barrel_target.with_extension("vue.ts").is_file());
    assert!(
        project
            .documents
            .iter()
            .any(|(uri, _)| uri.ends_with("Widget.vue.ts"))
    );
}

#[test]
fn cache_detects_same_mtime_source_and_manifest_changes() {
    let fixture = package_fixture("ui");
    let original = fixture.package.join("src/Widget.vue");
    let retargeted = fixture.package.join("src/Gadget.vue");
    write_package_manifest(&fixture.package, "Widget.vue");
    write_component(&fixture.package, "Widget.vue", UI_BUTTON);
    link_package(&fixture.package, &fixture.link);
    let source = host_import("widget");

    let initial = build(&fixture.host, &source);
    let initial_target = rewritten_package_target(&initial.host.code);
    let mirror = initial_target.with_extension("vue.ts");
    let initial_mirror = std::fs::read_to_string(&mirror).unwrap();

    let source_mtime = modified(&original);
    let changed = UI_BUTTON.replace("ghost", "solid");
    assert_eq!(changed.len(), UI_BUTTON.len());
    std::fs::write(&original, changed).unwrap();
    restore_mtime(&original, source_mtime);
    let refreshed = build(&fixture.host, &source);
    let refreshed_mirror = std::fs::read_to_string(
        rewritten_package_target(&refreshed.host.code).with_extension("vue.ts"),
    )
    .unwrap();
    assert_ne!(initial_mirror, refreshed_mirror);
    assert!(refreshed_mirror.contains("solid"));

    std::fs::remove_file(&original).unwrap();
    let deleted = build(&fixture.host, &source);
    assert!(deleted.host.code.contains("@scope/ui/widget"));
    assert!(!mirror.exists());

    write_component(&fixture.package, "Widget.vue", UI_BUTTON);
    assert!(
        !build(&fixture.host, &source)
            .host
            .code
            .contains("@scope/ui/widget")
    );

    let manifest = fixture.package.join("package.json");
    let manifest_mtime = modified(&manifest);
    std::fs::rename(&original, &retargeted).unwrap();
    write_package_manifest(&fixture.package, "Gadget.vue");
    restore_mtime(&manifest, manifest_mtime);
    let rerouted = build(&fixture.host, &source);
    let rerouted_target = rewritten_package_target(&rerouted.host.code);
    assert!(rerouted_target.to_string_lossy().contains("Gadget.vue"));
    assert!(!mirror.exists());
}

#[test]
fn unresolved_package_cache_recovers_after_link_manifest_and_source_creation() {
    let fixture = package_fixture("ui");
    let source = host_import("widget");

    let unresolved = build(&fixture.host, &source);
    assert!(unresolved.host.code.contains("@scope/ui/widget"));

    write_package_manifest(&fixture.package, "Widget.vue");
    write_component(&fixture.package, "Widget.vue", UI_BUTTON);
    link_package(&fixture.package, &fixture.link);
    let resolved = build(&fixture.host, &source);
    let target = rewritten_package_target(&resolved.host.code);
    assert!(target.with_extension("vue.ts").is_file());
}

#[test]
fn invalid_manifest_cache_recovers_after_same_mtime_fix() {
    let fixture = package_fixture("ui");
    let valid = package_manifest("Widget.vue");
    let invalid = valid.replacen('{', "[", 1);
    assert_eq!(invalid.len(), valid.len());
    std::fs::write(fixture.package.join("package.json"), invalid).unwrap();
    write_component(&fixture.package, "Widget.vue", UI_BUTTON);
    link_package(&fixture.package, &fixture.link);
    let source = host_import("widget");
    let manifest = fixture.package.join("package.json");
    let manifest_mtime = modified(&manifest);

    let unresolved = build(&fixture.host, &source);
    assert!(unresolved.host.code.contains("@scope/ui/widget"));

    std::fs::write(&manifest, valid).unwrap();
    restore_mtime(&manifest, manifest_mtime);
    let resolved = build(&fixture.host, &source);
    assert!(
        rewritten_package_target(&resolved.host.code)
            .with_extension("vue.ts")
            .is_file()
    );
}

#[test]
fn full_host_content_invalidates_export_from_routes() {
    let fixture = package_fixture("ui");
    std::fs::write(
        fixture.package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{"./widget":"./src/Widget.vue","./gadget":"./src/Gadget.vue"}}"#,
    )
    .unwrap();
    write_component(&fixture.package, "Widget.vue", UI_BUTTON);
    write_component(&fixture.package, "Gadget.vue", UI_BUTTON);
    link_package(&fixture.package, &fixture.link);
    let widget =
        "<script lang=\"ts\">\nexport { default as Selected } from '@scope/ui/widget'\n</script>\n";
    let gadget = widget.replace("widget", "gadget");
    assert_eq!(widget.len(), gadget.len());

    let first = rewritten_package_target(&build(&fixture.host, widget).host.code);
    let second = rewritten_package_target(&build(&fixture.host, &gadget).host.code);
    assert!(first.to_string_lossy().contains("Widget.vue"));
    assert!(second.to_string_lossy().contains("Gadget.vue"));
}

#[test]
fn cache_detects_workspace_package_symlink_retarget() {
    let fixture = package_fixture("ui-a");
    let second = fixture.root.path().join("packages/ui-b");
    write_package_manifest(&fixture.package, "Widget.vue");
    write_component(&fixture.package, "Widget.vue", UI_BUTTON);
    write_package_manifest(&second, "Widget.vue");
    write_component(&second, "Widget.vue", UI_BUTTON);
    link_package(&fixture.package, &fixture.link);
    let source = host_import("widget");

    let first = rewritten_package_target(&build(&fixture.host, &source).host.code);
    std::fs::remove_file(&fixture.link).unwrap();
    link_package(&second, &fixture.link);
    let retargeted = rewritten_package_target(&build(&fixture.host, &source).host.code);

    assert!(first.to_string_lossy().contains("ui-a"));
    assert!(retargeted.to_string_lossy().contains("ui-b"));
}

struct PackageFixture {
    root: tempfile::TempDir,
    host: PathBuf,
    package: PathBuf,
    link: PathBuf,
}

fn package_fixture(package_name: &str) -> PackageFixture {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let host = app.join("src/Host.vue");
    let package = root.path().join("packages").join(package_name);
    std::fs::create_dir_all(host.parent().unwrap()).unwrap();
    std::fs::create_dir_all(package.join("src")).unwrap();
    std::fs::write(app.join("tsconfig.json"), TSCONFIG).unwrap();
    PackageFixture {
        root,
        host,
        package,
        link: app.join("node_modules/@scope/ui"),
    }
}

fn build(host: &Path, source: &str) -> super::vue_document::CorsaVueVirtualProject {
    build_vue_virtual_project(host, source, CorsaVueVirtualDocumentOptions::default()).unwrap()
}

fn host_import(subpath: &str) -> vize_carton::String {
    cstr!(
        "<script setup lang=\"ts\">\nimport Widget from '@scope/ui/{subpath}'\nvoid Widget\n</script>\n"
    )
}

fn write_package_manifest(package: &Path, target: &str) {
    std::fs::create_dir_all(package).unwrap();
    std::fs::write(package.join("package.json"), package_manifest(target)).unwrap();
}

fn package_manifest(target: &str) -> vize_carton::String {
    cstr!("{{\"name\":\"@scope/ui\",\"exports\":{{\"./widget\":\"./src/{target}\"}}}}")
}

fn write_component(package: &Path, name: &str, content: &str) {
    std::fs::create_dir_all(package.join("src")).unwrap();
    std::fs::write(package.join("src").join(name), content).unwrap();
}

fn rewritten_package_target(code: &str) -> PathBuf {
    code.split(['\'', '"'])
        .find(|part| part.contains("/__vize_external__/"))
        .map(PathBuf::from)
        .expect("rewritten package target")
}

fn modified(path: &Path) -> std::time::SystemTime {
    std::fs::metadata(path).unwrap().modified().unwrap()
}

fn restore_mtime(path: &Path, modified: std::time::SystemTime) {
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(modified)
        .unwrap();
}

fn link_package(source: &Path, target: &Path) {
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(source, target).unwrap();
}
