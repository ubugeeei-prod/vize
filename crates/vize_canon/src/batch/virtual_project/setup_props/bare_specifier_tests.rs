//! Runtime props imported through bare specifiers resolve the way the
//! program's TypeScript project does (#13110).

use std::fs;
use std::path::Path;

use vize_carton::{CompactString, cstr};

use super::RuntimePropResolveCache;
use super::imports::RuntimePropVisitSet;
use super::runtime_props::resolve_imported_runtime_props;

const PROPS: &str =
    "export const sharedProps = { label: { type: String, required: true } } as const\n";

fn write(root: &Path, path: &str, contents: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn link(root: &Path, name: &str, target: &str) {
    let path = root.join("node_modules").join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(target, path).unwrap();
}

fn prop_names(
    importer: &Path,
    specifier: &str,
    cache: &RuntimePropResolveCache,
) -> Vec<CompactString> {
    resolve_imported_runtime_props(
        importer,
        specifier,
        "sharedProps",
        "sharedProps",
        &mut RuntimePropVisitSet::default(),
        cache,
    )
    .into_iter()
    .map(|prop| prop.name)
    .collect()
}

#[test]
fn bare_specifiers_follow_workspace_links_manifests_and_paths() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "paths": { "~shared/*": ["./shared/*"], "@/*": ["./app/*"] }
  },
  "include": ["src/**/*"]
}"#,
    );
    write(root, "src/App.vue", "");
    write(root, "shared/props.ts", PROPS);
    write(root, "app/props.ts", PROPS);
    for (folder, manifest, entry) in [
        (
            "fake-props",
            r#"{"name":"fake-props","exports":"./index.ts"}"#,
            "index.ts",
        ),
        (
            "packages/in-packages",
            r#"{"name":"in-packages","exports":"./index.ts"}"#,
            "index.ts",
        ),
        (
            "packages/some-folder",
            r#"{"name":"renamed-props","exports":"./index.ts"}"#,
            "index.ts",
        ),
        (
            "libs/ui",
            r#"{"name":"@acme/design-system","exports":{".":{"types":"./src/entry.ts","import":"./src/entry.ts","default":"./src/entry.ts"},"./props":"./src/props.ts"}}"#,
            "src/entry.ts",
        ),
        (
            "libs/import-only",
            r#"{"name":"import-only","exports":{".":{"import":"./src/entry.ts"}}}"#,
            "src/entry.ts",
        ),
        (
            "libs/string-exports",
            r#"{"name":"string-exports","exports":"./src/props.ts"}"#,
            "src/props.ts",
        ),
        (
            "libs/types-entry",
            r#"{"name":"types-entry","types":"./lib/props.ts"}"#,
            "lib/props.ts",
        ),
        (
            "libs/main-entry",
            r#"{"name":"main-entry","main":"./lib/props.ts"}"#,
            "lib/props.ts",
        ),
        (
            "packages/unlinked",
            r#"{"name":"unlinked","exports":"./index.ts"}"#,
            "index.ts",
        ),
    ] {
        write(root, &cstr!("{folder}/package.json"), manifest);
        write(root, &cstr!("{folder}/{entry}"), PROPS);
    }
    write(root, "libs/ui/src/props.ts", PROPS);
    for (name, target) in [
        ("fake-props", "../fake-props"),
        ("in-packages", "../packages/in-packages"),
        ("renamed-props", "../packages/some-folder"),
        ("@acme/design-system", "../../libs/ui"),
        ("import-only", "../libs/import-only"),
        ("string-exports", "../libs/string-exports"),
        ("types-entry", "../libs/types-entry"),
        ("main-entry", "../libs/main-entry"),
    ] {
        link(root, name, target);
    }

    let importer = root.join("src/App.vue");
    for cache in [
        RuntimePropResolveCache::for_tsconfig(Some(&root.join("tsconfig.json"))),
        RuntimePropResolveCache::default(),
    ] {
        for (specifier, expected) in [
            ("fake-props", &["label"][..]),
            ("in-packages", &["label"]),
            ("renamed-props", &["label"]),
            ("@acme/design-system", &["label"]),
            ("@acme/design-system/props", &["label"]),
            ("import-only", &["label"]),
            ("string-exports", &["label"]),
            ("types-entry", &["label"]),
            ("main-entry", &["label"]),
            ("~shared/props", &["label"]),
            ("@/props", &["label"]),
            // Neither linked nor mapped: TypeScript cannot resolve it either.
            ("unlinked", &[]),
        ] {
            assert_eq!(
                prop_names(&importer, specifier, &cache),
                expected,
                "{specifier}"
            );
        }
    }
}

#[test]
fn at_alias_without_paths_keeps_the_src_convention() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "tsconfig.json",
        r#"{ "compilerOptions": { "moduleResolution": "Bundler" }, "include": ["src/**/*"] }"#,
    );
    write(root, "src/components/App.vue", "");
    write(root, "src/props.ts", PROPS);

    let importer = root.join("src/components/App.vue");
    for cache in [
        RuntimePropResolveCache::for_tsconfig(Some(&root.join("tsconfig.json"))),
        RuntimePropResolveCache::default(),
    ] {
        assert_eq!(prop_names(&importer, "@/props", &cache), ["label"]);
    }
}
