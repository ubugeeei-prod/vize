//! #11591: a content-mapper declaration lets vue-tsc templates pass undeclared
//! attributes, exactly as the `vize check --declaration` output does (#11454).
#![cfg(test)]
use crate::support::corsa_requirement;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use vize_canon::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options,
};

const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/declaration-fallthrough-attrs-11454/tsconfig.json.txt"
));

const FOO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/declaration-fallthrough-attrs-11454/Foo.vue.txt"
));

const PROBE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/declaration-fallthrough-attrs-11454/Probe.vue.txt"
));

/// The shape vue-tsc emits for a generic SFC (as in `generic_library_children`).
const LIBRARY: &str = r#"export declare const PinRoot: <Type extends 'text' | 'number' = 'text'>(
  props: import('vue').PublicProps & { type?: Type } & (typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P } ? P : {}),
  ctx?: any,
) => import('vue').VNode & { __ctx?: { props: { type?: Type } } };
"#;

const GENERIC_PROBE: &str = r#"<script setup lang="ts">
import Foo from "./vendor/Foo.vue";
import { PinRoot } from "./vendor/library";
</script>

<template>
  <Foo />
  <PinRoot type="text" custom-attr="x" />
</template>
"#;

/// The #11591 producer: one emit, no props.
const EMIT_ONLY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/content-mapper-declaration-fallthrough-11591/Foo.vue.txt"
));

/// The reported consumer, with its deliberate TS2322.
const REPORTED_PROBE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/content-mapper-declaration-fallthrough-11591/Probe.vue.txt"
));

const REPORTED_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/content-mapper-declaration-fallthrough-11591/tsconfig.json.txt"
));

const PRODUCER_CONFIG: &str = r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "declaration": true,
    "emitDeclarationOnly": true,
    "outDir": "../out",
    "rootDir": "./src",
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext",
    "types": []
  },
  "include": ["src/Foo.vue.ts"]
}
"#;

const TSX_CONFIG: &str = r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "types": []
  },
  "include": ["src/Probe.tsx"]
}
"#;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

/// Install the published `vize` declaration surface (its `files` list) beside Vue.
fn install_packages(node_modules: &Path) {
    let package = root().join("npm/cli");
    let target = node_modules.join("vize");
    fs::create_dir_all(target.join("generated")).unwrap();
    fs::copy(package.join("package.json"), target.join("package.json")).unwrap();
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(package.join("package.json")).unwrap()).unwrap();
    for file in manifest["files"].as_array().unwrap() {
        let file = file.as_str().unwrap();
        if [".d.ts", ".d.mts", ".d.cts"]
            .iter()
            .any(|extension| file.ends_with(extension))
        {
            fs::copy(package.join(file), target.join(file)).unwrap();
        }
    }
    let vue = fs::canonicalize(root().join("tests/node_modules/vue")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue, node_modules.join("vue")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(vue, node_modules.join("vue")).unwrap();
}

/// `(line, column, code)` of each diagnostic a checker reports for `file`.
fn sites(stdout: &str, file: &str) -> Vec<(u32, u32, u32)> {
    stdout
        .lines()
        .filter_map(|line| {
            let (site, rest) = line
                .strip_prefix(file)?
                .strip_prefix('(')?
                .split_once("): error TS")?;
            let (line, column) = site.split_once(',')?;
            let (code, _) = rest.split_once(':')?;
            Some((line.parse().ok()?, column.parse().ok()?, code.parse().ok()?))
        })
        .collect()
}

/// Emit `Foo.vue` through the installed-package content mapper and return its declaration.
fn emit(corsa: &Path, dir: &Path, foo: &str) -> String {
    let producer = dir.join("producer");
    fs::create_dir_all(producer.join("src")).unwrap();
    let transform = generate_vue_content_mapper_transform_with_options(
        &producer.join("src/Foo.vue"),
        foo,
        ContentMapperTransformOptions::default()
            .with_external_component_attrs_module(true)
            .with_external_component_type_helpers(true)
            .with_shared_virtual_module(true),
    )
    .unwrap();
    fs::write(producer.join("src/Foo.vue.ts"), transform.text.as_str()).unwrap();
    fs::write(producer.join("tsconfig.json"), PRODUCER_CONFIG).unwrap();
    let emitted = Command::new(corsa)
        .current_dir(&producer)
        .args(["-p", "tsconfig.json", "--pretty", "false"])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&emitted.stdout).unwrap();
    assert!(emitted.status.success(), "{stdout}\n{}", transform.text);
    fs::read_to_string(dir.join("out/Foo.vue.d.ts")).unwrap()
}

/// `(line, column, code)` of each diagnostic for a consumer of the mapper declaration.
fn consume(corsa: &Path, foo: &str, probe: &str, config: &str) -> (Vec<(u32, u32, u32)>, String) {
    let dir = tempfile::Builder::new()
        .prefix("mapper-declaration-fallthrough-11591-")
        .tempdir()
        .unwrap();
    install_packages(&dir.path().join("node_modules"));
    let declaration = emit(corsa, dir.path(), foo);
    let consumer = dir.path().join("consumer");
    fs::create_dir_all(consumer.join("src/vendor")).unwrap();
    fs::write(consumer.join("src/vendor/Foo.vue.d.ts"), &declaration).unwrap();
    fs::write(consumer.join("src/vendor/library.d.ts"), LIBRARY).unwrap();
    let tsx = config == TSX_CONFIG;
    let (file, checker) = if tsx {
        ("src/Probe.tsx", "tests/node_modules/typescript/bin/tsc")
    } else {
        ("src/Probe.vue", "tests/node_modules/vue-tsc/bin/vue-tsc.js")
    };
    fs::write(consumer.join(file), probe).unwrap();
    fs::write(consumer.join("tsconfig.json"), config).unwrap();
    let checked = Command::new("node")
        .current_dir(&consumer)
        .arg(root().join(checker))
        .args(["--noEmit", "-p", "tsconfig.json", "--pretty", "false"])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&checked.stdout).unwrap();
    let found = sites(stdout, file);
    assert_eq!(
        checked.status.code(),
        Some(if found.is_empty() { 0 } else { 2 }),
        "{stdout}\n{declaration}"
    );
    (found, stdout.to_owned())
}

type Case<'a> = (&'a str, String, String, &'a str, Vec<(u32, u32, u32)>);

#[test]
fn mapper_declarations_accept_fallthrough_attributes_in_vue_tsc_templates() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    for provider in [
        "tests/node_modules/vue-tsc/bin/vue-tsc.js",
        "tests/node_modules/typescript/bin/tsc",
    ] {
        assert!(root().join(provider).is_file(), "{provider} required");
    }
    let strict = CONFIG.replace(
        "\"include\"",
        "\"vueCompilerOptions\": { \"checkUnknownProps\": true },\n  \"include\"",
    );
    let reported = |attrs: &str| {
        PROBE.replace(
            "<Foo :bar=\"1\" disabled />",
            &vize_l0::cstr!("<Foo {attrs} />"),
        )
    };
    let typed = EMIT_ONLY.replace(
        "defineEmits",
        "defineProps<{ configuration: string }>();\ndefineEmits",
    );
    let cases: [Case; 12] = [
        (
            "original",
            EMIT_ONLY.into(),
            REPORTED_PROBE.into(),
            REPORTED_CONFIG,
            vec![(4, 7, 2322)],
        ),
        (
            "reported",
            EMIT_ONLY.into(),
            reported("id=\"r\""),
            CONFIG,
            vec![],
        ),
        (
            "reported-attributes",
            EMIT_ONLY.into(),
            reported(
                "id=\"r\" role=\"r\" title=\"r\" hidden disabled custom-attr=\"r\" class=\"r\"",
            ),
            CONFIG,
            vec![],
        ),
        ("prop-and-emit", FOO.into(), PROBE.into(), CONFIG, vec![]),
        (
            "typed-prop-and-attribute",
            typed.clone(),
            reported("configuration=\"x\" id=\"r\""),
            CONFIG,
            vec![],
        ),
        (
            "wrong-typed-prop",
            typed,
            reported(":configuration=\"1\" id=\"r\""),
            CONFIG,
            vec![(6, 9, 2322)],
        ),
        (
            "missing-prop",
            FOO.into(),
            PROBE.replace(":bar=\"1\" ", ""),
            CONFIG,
            vec![(6, 4, 2345)],
        ),
        (
            "wrong-listener",
            FOO.into(),
            PROBE.replace(" disabled", " onChange=\"wrong\" disabled"),
            CONFIG,
            vec![(6, 17, 2322)],
        ),
        (
            "check-unknown-props",
            EMIT_ONLY.into(),
            reported("id=\"r\""),
            &strict,
            vec![(6, 4, 2345)],
        ),
        (
            "generic-library",
            EMIT_ONLY.into(),
            GENERIC_PROBE.into(),
            CONFIG,
            vec![],
        ),
        // A loaded mapper declaration must not switch vue-tsc's own check off.
        (
            "check-unknown-props-generic-library",
            EMIT_ONLY.into(),
            GENERIC_PROBE.into(),
            &strict,
            vec![(8, 24, 2353)],
        ),
        (
            "plain-tsx",
            EMIT_ONLY.into(),
            "import Foo from \"./vendor/Foo.vue\";\n\nexport const probe = <Foo id=\"r\" />;\n"
                .into(),
            TSX_CONFIG,
            vec![(3, 23, 2322)],
        ),
    ];
    for (id, foo, probe, config, expected) in cases {
        let (found, stdout) = consume(&corsa, &foo, &probe, config);
        assert_eq!(found, expected, "{id}\n{stdout}");
    }
}
