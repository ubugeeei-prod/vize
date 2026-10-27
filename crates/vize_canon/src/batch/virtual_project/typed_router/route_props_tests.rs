//! Project admission reads the installed declarations of both real `5.4.0` packs.

use super::*;
use std::fs;
use tempfile::TempDir;

const CONFIG: &str = r#"{"compilerOptions":{"moduleResolution":"Bundler","rootDir":"."},"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#;
const PAGE: &str = "<script setup lang=\"ts\">\nimport { definePage } from 'vue-router/experimental'\nconst props = defineProps<{ extra?: string }>()\ndefinePage({ props: true })\nvoid props\n</script>\n<template>{{ id }}</template>";

/// Install the unchanged package, entry and shared declaration members.
fn install(root: &Path, kind: &str) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typechecker/experimental-router-project-capability");
    let package = root.join("node_modules/vue-router");
    let _ = fs::remove_dir_all(&package);
    let authority: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("artifacts.json")).unwrap()).unwrap();
    let provider = authority["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|provider| provider["kind"] == kind)
        .unwrap();
    for entry in provider["files"].as_array().unwrap() {
        let path = entry["path"].as_str().unwrap();
        let target = package.join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(
            fixture
                .join(kind)
                .join(vize_carton::cstr!("{path}.txt").as_str()),
            target,
        )
        .unwrap();
    }
}

fn project(kind: &str, config: &str) -> (TempDir, PathBuf) {
    let root = TempDir::new().unwrap();
    let canonical = vize_carton::path::canonicalize_non_verbatim(root.path());
    fs::write(canonical.join("tsconfig.json"), config).unwrap();
    install(&canonical, kind);
    (root, canonical)
}

fn snapshot(root: &Path) -> crate::CompilerConfigurationSnapshot {
    crate::CompilerConfigurationSnapshot::capture(root, Some(&root.join("tsconfig.json"))).unwrap()
}

#[test]
fn only_the_source_built_declarations_admit_route_props() {
    for (kind, admitted) in [("published", false), ("preview", true)] {
        let (_guard, root) = project(kind, CONFIG);
        let captured = snapshot(&root);
        assert!(captured.typed_router_enabled());
        assert_eq!(captured.typed_router_route_props(), admitted, "{kind}");
        let declarations = root.join("node_modules/vue-router/dist/experimental/index.d.ts");
        let watched = captured.dependencies().contains(&declarations);
        assert!(watched, "{kind}");
        let mut live = VirtualProject::new(&root).unwrap();
        live.set_tsconfig_path(Some(root.join("tsconfig.json")));
        assert_eq!(live.typed_router.route_props, admitted, "{kind}");
    }
}

#[test]
fn opt_out_resolution_and_dependency_edits_revoke_the_admission() {
    let (_guard, root) = project("preview", CONFIG);
    let admitted = snapshot(&root);
    assert!(admitted.typed_router_route_props());
    for config in [
        r#"{"compilerOptions":{"moduleResolution":"Bundler"}}"#,
        r#"{"compilerOptions":{"moduleResolution":"Node10"},"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
        r#"{"compilerOptions":{"moduleResolution":"Bundler","resolvePackageJsonExports":false},"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
    ] {
        fs::write(root.join("tsconfig.json"), config).unwrap();
        assert!(!snapshot(&root).typed_router_route_props(), "{config}");
    }
    fs::write(root.join("tsconfig.json"), CONFIG).unwrap();
    assert!(admitted.same_authority(&snapshot(&root)));
    // Replacing the installed package with the published bytes is observed.
    install(&root, "published");
    assert!(!admitted.observations().changed_paths().is_empty());
    let replaced = snapshot(&root);
    assert!(!replaced.typed_router_route_props());
    assert!(!admitted.same_authority(&replaced));
    assert!(
        admitted.typed_router_route_props(),
        "an admitted snapshot is immutable"
    );
    fs::remove_dir_all(root.join("node_modules")).unwrap();
    assert!(!snapshot(&root).typed_router_route_props());
}

#[test]
fn admitted_pages_project_the_public_template_and_setup_contracts() {
    let mut generated = Vec::new();
    for kind in ["published", "preview"] {
        let (_guard, root) = project(kind, CONFIG);
        let mut live = VirtualProject::new(&root).unwrap();
        live.set_tsconfig_path(Some(root.join("tsconfig.json")));
        let path = root.join("src/pages/[id].vue");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, PAGE).unwrap();
        live.register_vue_file(&path, PAGE).unwrap();
        assert_eq!(live.has_typed_router_imports(), kind == "preview");
        let file = live
            .virtual_files_sorted()
            .into_iter()
            .find(|file| file.original_path == path)
            .unwrap();
        generated.push(file.content.clone());
    }
    let mut generated = generated.iter();
    let (published, preview) = (generated.next().unwrap(), generated.next().unwrap());
    let helper = "import('vue-router/experimental')._RoutePropsForFilePath<\"src/pages/[id].vue\">";
    assert_eq!(
        line(published, "definePage<"),
        "definePage<\"src/pages/[id].vue\">({ props: true })"
    );
    assert_eq!(
        line(published, "const props ="),
        "const props = defineProps<{ extra?: string }>()"
    );
    assert_eq!(published.matches(helper).count(), 0);
    assert_eq!(
        line(preview, "definePage<"),
        "definePage<\"src/pages/[id].vue\", typeof import(\"./[id].vue.ts\").default>({ props: true })"
    );
    assert_eq!(
        line(preview, "const props ="),
        vize_carton::cstr!("const props = defineProps<{helper} & ({{ extra?: string }})>()")
    );
    assert_eq!(
        line(preview, "export type Props"),
        vize_carton::cstr!("export type Props = {helper} & ({{ extra?: string }});")
    );
    // A route param used bare in the template resolves through the projected contract.
    assert_eq!(
        line(preview, "const id ="),
        "const id = __vize_keyed_template_context[(\"id\" satisfies keyof typeof __vize_keyed_template_context)];"
    );
}

fn line<'a>(code: &'a str, start: &str) -> &'a str {
    code.lines()
        .map(str::trim)
        .find(|line| line.starts_with(start))
        .unwrap_or_default()
}
