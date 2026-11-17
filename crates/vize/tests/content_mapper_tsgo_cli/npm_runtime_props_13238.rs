//! #13238 / #13121: `defineProps(sharedProps)` with a runtime props object
//! from a package that ships only declarations
//! (`export declare const sharedProps: { ... }`) has no initializer to read
//! prop names from. The contract and the template are typed from the
//! checker's `typeof sharedProps` instead, under the content mapper and
//! `vize check` alike. Before, every prop reported TS2339 on
//! `$TemplateInstance` and the parent saw a component with no props.

use std::{fs, path::Path, path::PathBuf, process::Command};

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

macro_rules! reported {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/npm-runtime-props-13238/",
            $path
        ))
    };
}

/// The reporter's project, file for file. The relative control is the same
/// object the issue describes.
const REPORTED: [(&str, &str); 8] = [
    ("package.json", reported!("package.json.txt")),
    ("tsconfig.json", reported!("tsconfig.json.txt")),
    (
        "node_modules/props-pkg/package.json",
        reported!("props-pkg/package.json.txt"),
    ),
    (
        "node_modules/props-pkg/index.d.ts",
        reported!("props-pkg/index.d.ts.txt"),
    ),
    (
        "node_modules/props-pkg/index.js",
        reported!("props-pkg/index.js.txt"),
    ),
    ("src/pkg-props.vue", reported!("src/pkg-props.vue.txt")),
    ("src/local/props.ts", reported!("src/local/props.ts.txt")),
    ("src/local-props.vue", reported!("src/local-props.vue.txt")),
];

/// Declarations-only packages: a barrel re-exporting through `export *`, and
/// a default-exported object, each with defaulted and Boolean props.
const PACKAGES: [(&str, &str); 6] = [
    (
        "node_modules/star-pkg/package.json",
        r#"{ "name": "star-pkg", "type": "module", "exports": { ".": { "types": "./dist/index.d.ts" } } }"#,
    ),
    (
        "node_modules/star-pkg/dist/index.d.ts",
        "export * from \"./props\";\n",
    ),
    (
        "node_modules/star-pkg/dist/props.d.ts",
        r#"import type { PropType } from "vue";
export declare const starProps: {
    title: {
        type: PropType<string>;
        required: true;
    };
    count: {
        type: NumberConstructor;
        default: number;
    };
    flag: BooleanConstructor;
    note: StringConstructor;
};
"#,
    ),
    (
        "node_modules/default-pkg/package.json",
        r#"{ "name": "default-pkg", "type": "module", "types": "./index.d.ts" }"#,
    ),
    (
        "node_modules/default-pkg/index.d.ts",
        r#"declare const _default: {
    caption: {
        type: StringConstructor;
        required: true;
    };
};
export default _default;
"#,
    ),
    (
        "src/star-props.vue",
        r#"<script setup lang="ts">
import { starProps } from "star-pkg";

defineProps(starProps);
</script>

<template>
  <p :title="title">{{ count.toFixed(1) }} {{ flag ? 1 : 0 }} {{ note?.length }}</p>
  <p>{{ $props.title.length }}</p>
</template>
"#,
    ),
];

const DEFAULT_COMPONENT: (&str, &str) = (
    "src/default-props.vue",
    r#"<script setup lang="ts">
import defaultProps from "default-pkg";

defineProps(defaultProps);
</script>

<template>
  <p>{{ caption.toUpperCase() }}</p>
</template>
"#,
);

/// The props stay exact: an unknown member, an optional prop's `undefined`,
/// an undeclared name, and a parent that omits a required prop all report.
const NEGATIVE: [(&str, &str); 2] = [
    (
        "src/negative.vue",
        r#"<script setup lang="ts">
import { starProps } from "star-pkg";

defineProps(starProps);
</script>

<template>
  <p>{{ title.notAMethod() }}</p>
  <p>{{ note.length }}</p>
  <p>{{ titel }}</p>
</template>
"#,
    ),
    (
        "src/parent.vue",
        r#"<script setup lang="ts">
import PkgProps from "./pkg-props.vue";
import StarProps from "./star-props.vue";
</script>

<template>
  <PkgProps label="a" :on-remove="() => {}" />
  <StarProps title="t" :count="2" flag />
  <PkgProps />
</template>
"#,
    ),
];

fn write(root: &Path, path: &str, contents: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn mapper_lines(tsgo: &Path, root: &Path) -> (Option<i32>, CompactString) {
    let output = check_project(tsgo, root, "tsconfig.json");
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    (
        output.status.code(),
        std::str::from_utf8(&output.stdout).unwrap().into(),
    )
}

fn cli_lines(tsgo: &Path, root: &Path) -> (Option<i32>, Vec<CompactString>) {
    let cli = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", tsgo)
        .args([
            "check",
            "--quiet",
            "--format",
            "json",
            "--tsconfig",
            "tsconfig.json",
            "src",
        ])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&cli.stdout)
        .unwrap_or_else(|error| panic!("{error}\n{}", output_text(&cli)));
    let mut observed = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap().replace('\\', "/");
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |diagnostic| {
                    let text = diagnostic.as_str().unwrap();
                    let head = text.split(" (source:").next().unwrap_or(text);
                    let head = head.split(" [").next().unwrap_or(head);
                    let code = text
                        .split_once(" [")
                        .and_then(|(_, rest)| rest.split_once(']'))
                        .map_or("", |(code, _)| code);
                    cstr!("{name} {head} {code}")
                })
        })
        .collect::<Vec<_>>();
    observed.sort();
    (cli.status.code(), observed)
}

#[test]
fn standard_tsgo_types_template_props_from_declared_runtime_props() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("npm-runtime-props-13238-")
        .tempdir_in(cases_root)
        .unwrap();
    let root = project.path();
    install_mapper_manifest(root);
    install_vue_package(root);
    for (path, contents) in REPORTED {
        write(root, path, contents);
    }

    // The reported project: both rows are clean.
    assert_eq!(
        mapper_lines(&tsgo, root),
        (Some(0), CompactString::default())
    );
    assert_eq!(cli_lines(&tsgo, root), (Some(0), Vec::new()));

    // `export *` barrels and default exports from declarations-only packages,
    // with defaulted and Boolean props resolved in the template.
    for (path, contents) in PACKAGES.into_iter().chain([DEFAULT_COMPONENT]) {
        write(root, path, contents);
    }
    assert_eq!(
        mapper_lines(&tsgo, root),
        (Some(0), CompactString::default())
    );
    assert_eq!(cli_lines(&tsgo, root), (Some(0), Vec::new()));

    for (path, contents) in NEGATIVE {
        write(root, path, contents);
    }
    let (mapper_status, mapper) = mapper_lines(&tsgo, root);
    assert_eq!(mapper_status, Some(2), "{mapper}");
    assert_eq!(mapper, MAPPER_NEGATIVE);
    let (cli_status, cli) = cli_lines(&tsgo, root);
    assert_eq!(cli_status, Some(1), "{cli:?}");
    assert_eq!(cli, CLI_NEGATIVE);
}

const MAPPER_NEGATIVE: &str = "";

const CLI_NEGATIVE: [&str; 0] = [];
