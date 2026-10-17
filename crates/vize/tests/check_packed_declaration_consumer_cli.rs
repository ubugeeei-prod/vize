#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! Declarations emitted by `vize build --dts` must type-check when consumed
//! from a separate project that only has the packed files (#3984): no source
//! workspace links, no access to the library's `.vue` sources.
#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/vue_stub.rs"]
mod vue_stub;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_l0::cstr;

#[path = "check_packed_declaration_consumer_cli/marker_text.rs"]
mod marker_text;

const BUTTON_VUE: &str = r#"<script setup lang="ts">
export interface ButtonProps {
  label: string
  count?: number
}

const props = defineProps<ButtonProps>()

const emit = defineEmits<{
  change: [value: number]
}>()

function onClick() {
  emit('change', props.count ?? 0)
}
</script>

<template>
  <button type="button" @click="onClick">{{ label }}</button>
</template>
"#;

const CLEAN_USAGE: &str = r#"import Button, { type ButtonProps } from 'packed-button'

export const props: ButtonProps = { label: 'Save', count: 1 }
export const component: typeof Button = Button
"#;

const BROKEN_USAGE: &str = r#"import Button, { type ButtonProps } from 'packed-button'

export const props: ButtonProps = { label: 'Save', count: 'one' }
export const component: typeof Button = Button
"#;

const CLEAN_TEMPLATE: &str = r#"<script setup lang="ts">
import PackedButton from 'packed-button'

function onChange(value: number) {
  return value
}
</script>

<template>
  <PackedButton label="Save" :count="1" @change="onChange" />
</template>
"#;

const BROKEN_TEMPLATE: &str = r#"<script setup lang="ts">
import PackedButton from 'packed-button'

function onChange(value: number) {
  return value
}
</script>

<template>
  <PackedButton label="Save" :count="'one'" @change="onChange" />
</template>
"#;

const GENERIC_LIST_VUE: &str = r#"<script setup lang="ts" generic="T extends { id: number }">
defineProps<{ items: T[] }>()

defineEmits<{
  select: [item: T]
}>()
</script>

<template>
  <ul>
    <li v-for="item in items" :key="item.id">{{ item.id }}</li>
  </ul>
</template>
"#;

const GENERIC_USAGE: &str = r#"import List, { type Props } from 'packed-list'

export const list: typeof List = List
export const props: Props<{ id: number }> = { items: [{ id: 1 }] }
"#;

const BROKEN_GENERIC_USAGE: &str = r#"import List, { type Props } from 'packed-list'

export const list: typeof List = List
export const props: Props<{ id: number }> = { items: [{ id: 'one' }] }
"#;

const TSC_CONSUMER_TSCONFIG: &str = r#"{
  "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true, "skipLibCheck": false },
  "include": ["src/**/*.ts"]
}"#;

#[test]
fn packed_declarations_type_check_in_a_separate_consumer() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let consumer = packed_button_consumer(&corsa_path, "packed-dts-ts");
    consumer.write("src/usage.ts", CLEAN_USAGE);
    let clean = consumer.run(&corsa_path, &["check", "--format", "json"]);
    assert_eq!(
        clean.status,
        Some(0),
        "packed declarations should type-check:\n{}\n{}",
        clean.stdout,
        clean.stderr
    );
    assert_eq!(parse_json(&clean)["errorCount"], 0, "{}", clean.stdout);

    consumer.write("src/usage.ts", BROKEN_USAGE);
    let broken = consumer.run(&corsa_path, &["check", "--format", "json"]);
    assert_eq!(
        broken.status,
        Some(1),
        "a wrong prop type must fail against the packed types:\n{}\n{}",
        broken.stdout,
        broken.stderr
    );
    let broken_json = parse_json(&broken);
    assert_eq!(
        diagnostics_for(&broken_json, "src/usage.ts"),
        vec!["error:3:52 [TS2322] Type 'string' is not assignable to type 'number'."],
        "{}",
        broken.stdout
    );

    consumer.write("src/usage.ts", CLEAN_USAGE);
    let repaired = consumer.run(&corsa_path, &["check", "--format", "json"]);
    assert_eq!(
        repaired.status,
        Some(0),
        "repaired consumer should pass:\n{}\n{}",
        repaired.stdout,
        repaired.stderr
    );
}

#[test]
fn packed_component_props_and_emits_check_in_a_consumer_template() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let consumer = packed_button_consumer(&corsa_path, "packed-dts-template");
    consumer.write("src/App.vue", CLEAN_TEMPLATE);
    let clean = consumer.run(&corsa_path, &["check", "--format", "json"]);
    assert_eq!(
        clean.status,
        Some(0),
        "a template using the packed component should pass:\n{}\n{}",
        clean.stdout,
        clean.stderr
    );

    consumer.write("src/App.vue", BROKEN_TEMPLATE);
    let broken = consumer.run(&corsa_path, &["check", "--format", "json"]);
    assert_eq!(
        broken.status,
        Some(1),
        "a wrong prop type in the template must fail:\n{}\n{}",
        broken.stdout,
        broken.stderr
    );
    let diagnostics = diagnostics_for(&parse_json(&broken), "src/App.vue");
    assert_eq!(
        diagnostics,
        vec!["error:10:31 [TS2322] Type 'string' is not assignable to type 'number'."],
        "the diagnostic must land on the authored prop binding:\n{}",
        broken.stdout
    );

    consumer.write("src/App.vue", CLEAN_TEMPLATE);
    let repaired = consumer.run(&corsa_path, &["check", "--format", "json"]);
    assert_eq!(
        repaired.status,
        Some(0),
        "repaired template should pass:\n{}\n{}",
        repaired.stdout,
        repaired.stderr
    );
}

#[test]
fn packed_generic_declarations_compile_under_tsc_without_skip_lib_check() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let consumer = packed_consumer(
        &corsa_path,
        "packed-dts-generic",
        "List",
        GENERIC_LIST_VUE,
        "packed-list",
        true,
    );
    consumer.write("tsconfig.json", TSC_CONSUMER_TSCONFIG);
    consumer.write("src/usage.ts", GENERIC_USAGE);
    let clean = consumer.tsc(&corsa_path);
    assert_eq!(
        (clean.status, clean.stdout.as_str()),
        (Some(0), ""),
        "packed generic declarations must resolve every helper:\n{}",
        clean.stderr
    );

    consumer.write("src/usage.ts", BROKEN_GENERIC_USAGE);
    let broken = consumer.tsc(&corsa_path);
    assert_eq!(
        (broken.status, broken.stdout.as_str()),
        (
            Some(1),
            "src/usage.ts(4,57): error TS2322: Type 'string' is not assignable to type 'number'.\n"
        ),
        "{}",
        broken.stderr
    );
}

/// Builds `Button.vue` with `--dts` in one project and copies only the emitted
/// files into `node_modules/packed-button` of a second, unrelated project.
fn packed_button_consumer(corsa_path: &str, name: &str) -> Case {
    packed_consumer(
        corsa_path,
        name,
        "Button",
        BUTTON_VUE,
        "packed-button",
        false,
    )
}

/// Builds `src/{component}.vue` and packs only its emitted files as `package_name`;
/// `real_vue` swaps the Vue stub for the installed package on both sides.
fn packed_consumer(
    corsa_path: &str,
    name: &str,
    component: &str,
    source: &str,
    package_name: &str,
    real_vue: bool,
) -> Case {
    let library = Case::new(cstr!("{name}-library").as_str());
    if real_vue {
        library.link_real_vue();
    }
    library.write(
        "tsconfig.json",
        r#"{
  "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
  "include": ["src/**/*"]
}"#,
    );
    let entry = cstr!("src/{component}.vue");
    library.write(entry.as_str(), source);
    let build = library.run(
        corsa_path,
        &["build", entry.as_str(), "--output", "dist", "--dts"],
    );
    assert_eq!(
        build.status,
        Some(0),
        "library build should succeed:\n{}\n{}",
        build.stdout,
        build.stderr
    );

    let consumer = Case::new(cstr!("{name}-consumer").as_str());
    if real_vue {
        consumer.link_real_vue();
    }
    let package = consumer.root.join("node_modules").join(package_name);
    std::fs::create_dir_all(&package).unwrap();
    for entry in std::fs::read_dir(library.root.join("dist")).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), package.join(entry.file_name())).unwrap();
    }
    std::fs::write(
        package.join("package.json"),
        cstr!(
            r#"{{ "name": "{package_name}", "main": "./{component}.js", "types": "./{component}.vue.d.ts" }}"#
        )
        .as_str(),
    )
    .unwrap();
    drop(library);

    consumer.write(
        "tsconfig.json",
        r#"{
  "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
  "include": ["src/**/*"]
}"#,
    );
    consumer
}

struct RunResult {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

struct Case {
    root: PathBuf,
}

impl Case {
    fn new(name: &str) -> Self {
        let case_id = cstr!("{name}-{}", std::process::id());
        let root = workspace_root()
            .join("target")
            .join("vize-tests")
            .join("tests")
            .join(case_id.as_str());
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        vue_stub::install_vue_jsx_type_stub(&root);
        Self { root }
    }

    fn write(&self, path: &str, source: &str) {
        let path = self.root.join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, source).unwrap();
    }

    /// Replace the Vue stub with the workspace's installed Vue package.
    fn link_real_vue(&self) {
        let vue = workspace_root()
            .join("tests/node_modules/vue")
            .canonicalize()
            .expect("install workspace dependencies before packed Vue tests");
        let link = self.root.join("node_modules").join("vue");
        std::fs::remove_dir_all(&link).unwrap();
        std::os::unix::fs::symlink(vue, link).unwrap();
    }

    /// Run plain tsc (the Corsa binary) on the consumer project.
    fn tsc(&self, corsa_path: &str) -> RunResult {
        let output = Command::new(corsa_path)
            .current_dir(&self.root)
            .args(["--noEmit", "--pretty", "false", "-p", "tsconfig.json"])
            .output()
            .unwrap();
        RunResult {
            status: output.status.code(),
            stdout: std::str::from_utf8(&output.stdout).unwrap().into(),
            stderr: std::str::from_utf8(&output.stderr).unwrap().into(),
        }
    }

    fn run(&self, corsa_path: &str, args: &[&str]) -> RunResult {
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&self.root)
            .env("CORSA_PATH", corsa_path)
            .args(args)
            .output()
            .unwrap();
        RunResult {
            status: output.status.code(),
            stdout: std::str::from_utf8(&output.stdout).unwrap().into(),
            stderr: std::str::from_utf8(&output.stderr).unwrap().into(),
        }
    }
}

impl Drop for Case {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn parse_json(run: &RunResult) -> serde_json::Value {
    serde_json::from_str(&run.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse check output as JSON: {error}\nstdout:\n{}\nstderr:\n{}",
            run.stdout, run.stderr
        )
    })
}

fn diagnostics_for(json: &serde_json::Value, file_name: &str) -> Vec<String> {
    json["files"]
        .as_array()
        .expect("JSON output should include files")
        .iter()
        .find(|file| file["file"] == file_name)
        .map(|file| {
            file["diagnostics"]
                .as_array()
                .expect("file should include diagnostics")
                .iter()
                .map(|diagnostic| diagnostic.as_str().unwrap().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn resolve_test_corsa_path() -> Option<String> {
    corsa_requirement::required_or_skip(corsa_path::resolve(&workspace_root()))
}
