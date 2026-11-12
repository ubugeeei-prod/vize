#![cfg(test)]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use crate::support::corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_l0::cstr;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("check-canon-recent-{name}-{}", std::process::id()).as_str())
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn link_workspace_vue(project_root: &Path) -> std::io::Result<()> {
    let Some(vue_package) = workspace_vue_package() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package missing",
        ));
    };
    let workspace_node_modules = vue_package.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package has no node_modules parent",
        )
    })?;
    let target = project_root.join("node_modules");
    std::fs::create_dir_all(&target)?;
    symlink_path(&vue_package, &target.join("vue"))?;
    let vue_namespace = workspace_node_modules.join("@vue");
    if vue_namespace.exists() {
        symlink_path(&vue_namespace, &target.join("@vue"))?;
    }
    Ok(())
}

fn workspace_vue_package() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.join("node_modules/vue"),
        root.join("tests/node_modules/vue"),
        root.join("playground/node_modules/vue"),
        root.join("examples/vite-musea/node_modules/vue"),
        root.join("examples/jsx-tsx/node_modules/vue"),
        root.join("npm/framework/nuxt/node_modules/vue"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(target)?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}

fn write_tsconfig(project_root: &Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
}

fn create_case(name: &str, vue_file: &str, source: &str) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    std::fs::write(project_root.join("src").join(vue_file), source).unwrap();
    project_root
}

fn run_check_json(project_root: &Path, corsa_path: &Path, target: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            target,
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "check failed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    stdout.to_string()
}

#[test]
fn check_with_defaults_preserves_loose_required_props() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case(
        "with-defaults-loose-required",
        "Foo.vue",
        r#"<script lang="ts">
export interface FooProps {
  modelValue?: string
  as?: string
}
</script>

<script setup lang="ts">
type ReadModelValueOptions<TValue> = {
  props: {
    modelValue: TValue | undefined
  }
}

function readModelValue<TValue>(options: ReadModelValueOptions<TValue>) {
  return options.props.modelValue
}

const props = withDefaults(defineProps<FooProps>(), {
  as: 'input',
})

readModelValue({ props })
</script>

<template>{{ props.modelValue }}</template>
"#,
    );

    run_check_json(&project_root, &corsa_path, "src/Foo.vue");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_with_defaults_undefined_default_preserves_optional_prop_value() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case(
        "with-defaults-undefined-default",
        "Foo.vue",
        r#"<script setup lang="ts">
interface FooProps {
  modelValue?: string
}

type UseModelOptions<P, K extends keyof P> = {
  props: P
  key: K
  clone?: boolean | ((value: P[K]) => P[K])
}

function useModel<P, K extends keyof P>(options: UseModelOptions<P, K>) {
  void options
}

const props = withDefaults(defineProps<FooProps>(), {
  modelValue: undefined,
})

useModel({
  props,
  key: 'modelValue',
  clone: value => value === undefined ? undefined : value.trim(),
})
</script>

<template>{{ props.modelValue }}</template>
"#,
    );

    run_check_json(&project_root, &corsa_path, "src/Foo.vue");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_v_for_infers_items_from_union_of_arrays() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case(
        "vfor-array-union",
        "Foo.vue",
        r#"<script setup lang="ts">
import { computed, ref } from 'vue'

type ItemA = { key: string; valueA: number }
type ItemB = { key: string; valueB: string }

const showA = ref(true)
const items = computed<ItemA[] | ItemB[]>(() => (
  showA.value
    ? [{ key: 'a', valueA: 1 }]
    : [{ key: 'b', valueB: 'b' }]
))
</script>

<template>
  <div v-for="item in items" :key="item.key">
    {{ item.key }}
  </div>
</template>
"#,
    );

    run_check_json(&project_root, &corsa_path, "src/Foo.vue");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_define_props_return_assignable_to_record_helpers() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case(
        "define-props-record",
        "Foo.vue",
        r#"<script setup lang="ts">
type MaybeRefOrGetter<T> = T | (() => T)

interface FooProps {
  forceMount?: boolean
}

function forwardProps<T extends Record<string, unknown>>(value: MaybeRefOrGetter<T>) {
  return value
}

const props = defineProps<FooProps>()
forwardProps(props)
</script>

<template><div /></template>
"#,
    );

    run_check_json(&project_root, &corsa_path, "src/Foo.vue");

    let _ = std::fs::remove_dir_all(&project_root);
}

const SOLUTION_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/solution-tsconfig-imported-props/"
);

/// reka-ui's `packages/core/tsconfig.json` is solution-style (`files: []`
/// plus `references`); its `paths` live in the referenced app config. Imported
/// prop types behind `@/` must resolve through the selected `--tsconfig`, not
/// the nearest `tsconfig.json`, or `as` (bound as `_as` in the template scope)
/// is reported as a missing instance property.
#[test]
fn check_imported_props_resolve_through_the_selected_tsconfig_paths() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("solution-tsconfig-imported-props");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src/Primitive")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    for (input, target) in [
        ("tsconfig.json.txt", "tsconfig.json"),
        ("tsconfig.app.json.txt", "tsconfig.app.json"),
        ("Primitive.ts.txt", "src/Primitive/Primitive.ts"),
        ("index.ts.txt", "src/Primitive/index.ts"),
        ("Trigger.vue.txt", "src/Trigger.vue"),
    ] {
        let source = std::fs::read(Path::new(SOLUTION_FIXTURE).join(input)).unwrap();
        std::fs::write(project_root.join(target), source).unwrap();
    }

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.app.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    let diagnostics: Vec<&str> = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().iter())
        .filter_map(serde_json::Value::as_str)
        .collect();
    // vue-tsc 3.3.14 reports exactly this one row for the same project.
    assert_eq!(
        diagnostics,
        vec![
            "error:20:43 [TS2339] Property 'toFixed' does not exist on type 'boolean'. (source: <span>{{ as.toUpperCase() }} {{ asChild.toFixed() }}</span>)"
        ],
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_extended_interface_props_are_available_in_template_scope() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_case(
        "extended-interface-template-props",
        "Foo.vue",
        r#"<script setup lang="ts">
interface BaseProps {
  required?: boolean
}

interface FooProps extends BaseProps {
  label: string
}

defineProps<FooProps>()
</script>

<template>
  <label>
    <input :required="required" />
    {{ label }} {{ required }}
  </label>
</template>
"#,
    );

    run_check_json(&project_root, &corsa_path, "src/Foo.vue");

    let _ = std::fs::remove_dir_all(&project_root);
}

const UNION_HOST_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/union-host-slot-payload/"
);

/// Every diagnostic row of a `vize check --format json` run, in report order,
/// with the raw report for assertion messages.
fn check_diagnostic_rows(project_root: &Path, corsa_path: &Path) -> (Vec<String>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--tsconfig", "tsconfig.json", "--format", "json"])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap().to_string();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let rows = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().iter())
        .filter_map(serde_json::Value::as_str)
        .map(str::to_string)
        .collect();
    (rows, stdout)
}

/// A slot scope on a component picked from a union (nuxt/ui's `InputDate`
/// mounts `computed(() => range ? DateRangeField : DateField)`) receives the
/// props of whichever member renders. Inferring one parameter from the union of
/// both slot functions intersected the payloads, so the `v-else` branch of an
/// `Array.isArray` check narrowed to `never`.
#[test]
fn check_union_host_slot_payload_is_the_union_of_member_payloads() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("union-host-slot-payload");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    for name in ["Host.vue", "Single.vue", "Range.vue"] {
        let input = cstr!("{name}.txt");
        let source = std::fs::read(Path::new(UNION_HOST_FIXTURE).join(input.as_str())).unwrap();
        std::fs::write(project_root.join("src").join(name), source).unwrap();
    }

    let (rows, stdout) = check_diagnostic_rows(&project_root, &corsa_path);
    // vue-tsc 3.3.14 reports exactly this one row for the same project.
    assert_eq!(
        rows,
        vec![
            "error:14:66 [TS2339] Property 'toFixed' does not exist on type '{ part: string; }[]'. (source: <template v-else>{{ segments.start.length }} {{ segments.end.toFixed() }}</template>)"
        ],
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

const NESTED_CONSUMER_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/nested-consumer-rooted-stub/"
);

/// nuxt/ui keeps its `docs/` app below the library root, with its own
/// `package.json`, and the stub build's `dist/module.d.mts` re-exports the
/// source by a rooted path. A type exported from a library `.vue` must reach
/// the docs app: the docs import is not a self-reference, and the rooted
/// re-export must land on the mirrored source, not the unreadable original.
#[test]
fn check_nested_consumer_reaches_library_vue_types_through_a_rooted_stub() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let library = unique_case_dir("nested-consumer-rooted-stub");
    let _ = std::fs::remove_dir_all(&library);
    for dir in [
        "dist",
        "src/runtime/types",
        "src/runtime/components",
        "docs/src",
    ] {
        std::fs::create_dir_all(library.join(dir)).unwrap();
    }
    link_workspace_vue(&library).unwrap();
    std::fs::create_dir_all(library.join("docs/node_modules/@x")).unwrap();
    symlink_path(&library, &library.join("docs/node_modules/@x/ui")).unwrap();
    let canonical_library = std::fs::canonicalize(&library).unwrap();
    for (input, target) in [
        ("package.json.txt", "package.json"),
        ("module.d.mts.txt", "dist/module.d.mts"),
        ("module.ts.txt", "src/module.ts"),
        ("types-index.ts.txt", "src/runtime/types/index.ts"),
        ("Chip.vue.txt", "src/runtime/components/Chip.vue"),
        ("docs-package.json.txt", "docs/package.json"),
        ("docs-tsconfig.json.txt", "docs/tsconfig.json"),
        ("Use.vue.txt", "docs/src/Use.vue"),
    ] {
        let source = std::fs::read_to_string(Path::new(NESTED_CONSUMER_FIXTURE).join(input))
            .unwrap()
            .replace(
                "__PACKAGE_ROOT__",
                canonical_library.to_str().expect("utf-8 test path"),
            );
        std::fs::write(library.join(target), source).unwrap();
    }

    let (rows, stdout) = check_diagnostic_rows(&library.join("docs"), &corsa_path);
    // vue-tsc 3.3.14 reports exactly this one row for the same project.
    assert_eq!(
        rows,
        vec!["error:4:45 [TS2322] Type 'string' is not assignable to type 'boolean | undefined'."],
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&library);
}

const AUTO_IMPORTED_TEMPLATE_REF_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/auto-imported-template-ref/"
);

/// Nuxt auto-imports `useTemplateRef` as a global. Vue's language tools
/// recognize the composable by name (`composables.useTemplateRef` defaults to
/// `["useTemplateRef", "templateRef"]`), so the ref is the exposed instance of
/// the component that registers it, not `unknown`.
#[test]
fn check_auto_imported_use_template_ref_reads_the_template_registry() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("auto-imported-template-ref");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    for name in [
        "Input.vue",
        "Use.vue",
        "auto-imports.d.ts",
        "components.d.ts",
    ] {
        let input = cstr!("{name}.txt");
        let source =
            std::fs::read(Path::new(AUTO_IMPORTED_TEMPLATE_REF_FIXTURE).join(input.as_str()))
                .unwrap();
        std::fs::write(project_root.join("src").join(name), source).unwrap();
    }

    let (rows, stdout) = check_diagnostic_rows(&project_root, &corsa_path);
    // vue-tsc 3.3.14 reports exactly this one row for the same project.
    assert_eq!(
        rows,
        vec![
            "error:5:16 [TS2339] Property 'missing' does not exist on type '{ inputRef: HTMLInputElement | null; }'."
        ],
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

/// A one-file project from a pinned fixture, with `vue_compiler_options`
/// merged into the shared test tsconfig.
fn create_fixture_case(
    name: &str,
    fixture: &str,
    file: &str,
    vue_compiler_options: Option<serde_json::Value>,
) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    if let Some(options) = vue_compiler_options {
        let path = project_root.join("tsconfig.json");
        let mut config: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        config["vueCompilerOptions"] = options;
        std::fs::write(&path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
    }
    let input = cstr!("{file}.txt");
    let source = std::fs::read(
        workspace_root()
            .join("tests/_fixtures/differential/typechecker")
            .join(fixture)
            .join(input.as_str()),
    )
    .unwrap();
    std::fs::write(project_root.join("src").join(file), source).unwrap();
    project_root
}

/// The checker names Vize's private copy of Vue's `defineProps` result type;
/// the message must name it the way Vue declares it, as `vue-tsc` does.
#[test]
fn check_define_props_messages_use_vue_public_type_names() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_fixture_case(
        "define-props-public-name",
        "define-props-public-name",
        "Props.vue",
        None,
    );

    let (rows, stdout) = check_diagnostic_rows(&project_root, &corsa_path);
    // vue-tsc 3.3.14 reports exactly this one row for the same project.
    assert_eq!(
        rows,
        vec![
            "error:8:7 [TS2322] Type 'DefineProps<LooseRequired<Props>, \"disabled\">' is not assignable to type 'number'."
        ],
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
