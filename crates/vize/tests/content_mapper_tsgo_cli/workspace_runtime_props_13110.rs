//! #13110: `defineProps(sharedProps)` with a runtime props object imported
//! through a bare specifier resolves where TypeScript resolves the import:
//! `node_modules` workspace links (any folder, any package name), package
//! entry points and tsconfig `paths`. Before, only a folder named after the
//! package under `packages/` was found, and every other row lost its props
//! in the template (TS2339 on `$TemplateInstance`).

use std::{fs, path::Path, path::PathBuf};

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

macro_rules! reported {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/workspace-runtime-props-13110/",
            $path
        ))
    };
}

/// The reporter's workspace, file for file.
const REPORTED: [(&str, &str); 14] = [
    ("package.json", reported!("package.json.txt")),
    ("pnpm-workspace.yaml", reported!("pnpm-workspace.yaml.txt")),
    ("tsconfig.json", reported!("tsconfig.json.txt")),
    (
        "fake-props/package.json",
        reported!("fake-props/package.json.txt"),
    ),
    ("fake-props/index.ts", reported!("index.ts.txt")),
    (
        "packages/in-packages/package.json",
        reported!("packages/in-packages/package.json.txt"),
    ),
    ("packages/in-packages/index.ts", reported!("index.ts.txt")),
    (
        "packages/some-folder/package.json",
        reported!("packages/some-folder/package.json.txt"),
    ),
    ("packages/some-folder/index.ts", reported!("index.ts.txt")),
    ("src/props.ts", reported!("index.ts.txt")),
    (
        "src/RelativeProps.vue",
        reported!("src/RelativeProps.vue.txt"),
    ),
    (
        "src/InPackagesProps.vue",
        reported!("src/InPackagesProps.vue.txt"),
    ),
    (
        "src/RenamedFolderProps.vue",
        reported!("src/RenamedFolderProps.vue.txt"),
    ),
    (
        "src/PackageProps.vue",
        reported!("src/PackageProps.vue.txt"),
    ),
];

/// (folder, manifest, entry file) per additional workspace package.
const PACKAGES: [(&str, &str, &str); 5] = [
    // A scoped package whose conditional exports name a source entry.
    (
        "libs/ui",
        r#"{ "name": "@acme/design-system", "type": "module", "exports": { ".": { "import": "./src/entry.ts", "default": "./src/entry.ts" }, "./props": "./src/props.ts" } }"#,
        "src/entry.ts",
    ),
    (
        "libs/string-exports",
        r#"{ "name": "string-exports", "type": "module", "exports": "./src/props.ts" }"#,
        "src/props.ts",
    ),
    (
        "libs/types-entry",
        r#"{ "name": "types-entry", "type": "module", "types": "./lib/props.ts" }"#,
        "lib/props.ts",
    ),
    (
        "libs/main-entry",
        r#"{ "name": "main-entry", "type": "module", "main": "./lib/props.ts" }"#,
        "lib/props.ts",
    ),
    (
        "shared",
        r#"{ "name": "shared", "private": true }"#,
        "props.ts",
    ),
];

/// (link under node_modules, workspace folder), as pnpm links workspace packages.
const LINKS: [(&str, &str); 7] = [
    ("fake-props", "../fake-props"),
    ("in-packages", "../packages/in-packages"),
    ("renamed-props", "../packages/some-folder"),
    ("@acme/design-system", "../../libs/ui"),
    ("string-exports", "../libs/string-exports"),
    ("types-entry", "../libs/types-entry"),
    ("main-entry", "../libs/main-entry"),
];

/// (component, import specifier) beyond the reported rows.
const COMPONENTS: [(&str, &str); 6] = [
    ("ScopedProps", "@acme/design-system"),
    ("SubpathProps", "@acme/design-system/props"),
    ("StringExportsProps", "string-exports"),
    ("TypesEntryProps", "types-entry"),
    ("MainEntryProps", "main-entry"),
    ("PathsProps", "~shared/props"),
];

fn component(specifier: &str, template: &str) -> CompactString {
    cstr!(
        "<script setup lang=\"ts\">\nimport {{ sharedProps }} from \"{specifier}\";\n\ndefineProps(sharedProps);\n</script>\n\n<template>\n  <p>{template}</p>\n</template>\n"
    )
}

fn write(root: &Path, path: &str, contents: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn link(root: &Path, name: &str, target: &str) {
    let path = root.join("node_modules").join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, path).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(path.parent().unwrap().join(target), path).unwrap();
}

fn assert_clean(tsgo: &Path, root: &Path, config: &str) {
    let output = check_project(tsgo, root, config);
    assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
    assert_eq!(output.stdout, b"", "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
}

#[test]
fn standard_tsgo_reads_runtime_props_from_workspace_packages() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("workspace-runtime-props-13110-")
        .tempdir_in(cases_root)
        .unwrap();
    let root = project.path();
    install_mapper_manifest(root);
    install_vue_package(root);
    for (path, contents) in REPORTED {
        write(root, path, contents);
    }
    for (name, target) in LINKS {
        link(root, name, target);
    }

    // The reported project: vue-tsc accepts all four rows.
    assert_clean(&tsgo, root, "tsconfig.json");

    // Scoped names, conditional and subpath exports, `types`, `main`, `paths`.
    let props = reported!("index.ts.txt");
    for (folder, manifest, entry) in PACKAGES {
        write(root, cstr!("{folder}/package.json").as_str(), manifest);
        write(root, cstr!("{folder}/{entry}").as_str(), props);
    }
    write(root, "libs/ui/src/props.ts", props);
    for (name, specifier) in COMPONENTS {
        write(
            root,
            cstr!("src/{name}.vue").as_str(),
            &component(specifier, "{{ label }}"),
        );
    }
    write(
        root,
        "tsconfig.paths.json",
        r#"{
  "extends": "./tsconfig.json",
  "compilerOptions": { "paths": { "~shared/*": ["./shared/*"] } },
  "contentMappers": [{ "package": "vize", "extensions": [".vue"] }]
}"#,
    );
    assert_clean(&tsgo, root, "tsconfig.paths.json");

    // The props carry their declared type, not an open one (vue-tsc reports
    // the same single row).
    write(
        root,
        "libs/ui/src/entry.ts",
        "export const sharedProps = { label: { type: String, required: true } } as const;\n",
    );
    write(
        root,
        "src/ScopedProps.vue",
        &component("@acme/design-system", "{{ label.notAMethod() }}"),
    );
    let typed = check_project(&tsgo, root, "tsconfig.paths.json");
    assert_eq!(typed.status.code(), Some(2), "{}", output_text(&typed));
    assert_eq!(typed.stderr, b"", "{}", output_text(&typed));
    assert_eq!(
        std::str::from_utf8(&typed.stdout).unwrap(),
        "src/ScopedProps.vue(8,15): error TS2339: Property 'notAMethod' does not exist on type 'string'.\n",
        "{}",
        output_text(&typed)
    );
}
