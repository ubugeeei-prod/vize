#[path = "support/check_output.rs"]
mod check_output;
#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use check_output::normalize_check_output;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_carton::cstr;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<String> {
    corsa_requirement::required_or_skip(corsa_path::resolve(workspace_root()))
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str())
}

fn create_cli_project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    for (path, source) in files {
        let target = project_root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(target, source).unwrap();
    }
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
}
"#,
    )
    .unwrap();
    project_root
}

fn run_check(project_root: &Path, corsa_path: &str, format: Option<&str>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .env("NO_COLOR", "1")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("FORCE_COLOR")
        .args(["check", "."]);
    if let Some(format) = format {
        command.args(["--format", format]);
    }
    command.output().unwrap()
}

#[test]
fn check_json_stays_machine_oriented_while_text_includes_source_context() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "text-source-context",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
        )],
    );

    let json_output = run_check(&project_root, &corsa_path, Some("json"));
    let json_stdout = std::str::from_utf8(&json_output.stdout).unwrap();
    let json_stderr = std::str::from_utf8(&json_output.stderr).unwrap();
    assert_eq!(
        json_output.status.code(),
        Some(1),
        "stdout:\n{json_stdout}\nstderr:\n{json_stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(json_stdout).unwrap();
    let json_diagnostics: Vec<_> = json["files"]
        .as_array()
        .expect("JSON report should contain files")
        .iter()
        .flat_map(|file| {
            file["diagnostics"]
                .as_array()
                .expect("JSON report files should contain diagnostics")
        })
        .filter_map(serde_json::Value::as_str)
        .collect();
    // Exact oracle: the JSON report must carry exactly the known mismatch and
    // stay machine-oriented — the pinned text proves there is no `source:`
    // context (or anything else) appended to the diagnostic.
    assert_eq!(
        json_diagnostics,
        ["error:2:7 [TS2322] Type 'number' is not assignable to type 'string'."],
        "stderr:\n{json_stderr}"
    );

    let text_output = run_check(&project_root, &corsa_path, None);
    let stdout = std::str::from_utf8(&text_output.stdout).unwrap();
    let stderr = std::str::from_utf8(&text_output.stderr).unwrap();
    assert_eq!(
        text_output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    // Exact oracle over normalized text: the diagnostic line keeps the
    // parser-friendly `error:LINE:COL [CODE]` shape and appends the authored
    // `source:` context that the JSON report must not carry.
    assert_eq!(
        normalize_check_output(stdout, &project_root),
        "\n<project>/src/App.vue\n  error:2:7 [TS2322] Type 'number' is not assignable to type 'string'. (source: const count: string = 0;)\n\n\u{2717} Type checked 1 files in <duration> (collect: <duration>, gen: <duration>, corsa: <duration>)\n  1 error(s)\n",
        "stderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_text_diagnostics_name_template_bindings() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "text-template-source-context",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{
  modelValue: number;
  kind: "num";
  n: number;
  label?: string;
  value?: number;
  first?: number;
  second?: number;
}>();
defineEmits<{ save: [id: number] }>();
defineSlots<{ default(props: { count: number }): unknown }>();
</script>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";

const text = ref("bad");
</script>

<template>
  <Child v-model="text" />
  <Child kind="num" :s="'bad'" />
  <Child label="v-model:fake" kind="num" :n="1" :value="'bad'" />
  <Child label="😀" kind="num" :n="1" :first="1" :second="'bad'" />
  <Child :model-value="1" kind="num" :n="1" @save="(id: string) => {}" />
  <Child :model-value="1" kind="num" :n="1" v-slot="{ count }">{{ count.toUpperCase() }}</Child>
</template>
"#,
            ),
        ],
    );

    let output = run_check(&project_root, &corsa_path, None);
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    // Exact oracle over normalized text, pinning every binding rendering in
    // one artifact: the modeled prop (`modelValue`), the shorthand prop
    // (`'s'`), the directive-like quoted value (`'value'`), the binding after
    // a UTF-16 astral-plane column offset (`'second'`), the event (`@save`),
    // and the slot (`#default`). Declaring that slot must not add a missing-slot
    // diagnostic to any invocation that omits it.
    assert_eq!(
        normalize_check_output(stdout, &project_root),
        concat!(
            "\n<project>/src/App.vue\n",
            "  error:9:10 [TS2322] Type 'string' is not assignable to type 'number'. (source: <Child v-model=\"text\" />; binding: modelValue)\n",
            "  error:10:4 [TS2345] Argument of type '{ kind: \"num\"; s: string; }' is not assignable to parameter of type '{ readonly modelValue: number; readonly kind: \"num\"; readonly n: number; readonly label?: string | undefined; readonly value?: number | undefined; readonly first?: number | undefined; readonly second?: number | undefined; readonly onSave?: ((id: number) => any) | undefined; } & __VizePublicComponentAttrs & { ...; } ...'.\n",
            "Type '{ kind: \"num\"; s: string; }' is missing the following properties from type '{ readonly modelValue: number; readonly kind: \"num\"; readonly n: number; readonly label?: string | undefined; readonly value?: number | undefined; readonly first?: number | undefined; readonly second?: number | undefined; readonly onSave?: ((id: number) => any) | undefined; }': modelValue, n (source: <Child kind=\"num\" :s=\"'bad'\" />; binding: 's')\n",
            "  error:11:50 [TS2322] Type 'string' is not assignable to type 'number'. (source: <Child label=\"v-model:fake\" kind=\"num\" :n=\"1\" :value=\"'bad'\" />; binding: 'value')\n",
            "  error:12:51 [TS2322] Type 'string' is not assignable to type 'number'. (source: <Child label=\"😀\" kind=\"num\" :n=\"1\" :first=\"1\" :second=\"'bad'\" />; binding: 'second')\n",
            "  error:13:46 [TS2322] Type '(id: string) => void' is not assignable to type '(id: number) => any'.\n",
            "Types of parameters 'id' and 'id' are incompatible.\n",
            "Type 'number' is not assignable to type 'string'. (source: <Child :model-value=\"1\" kind=\"num\" :n=\"1\" @save=\"(id: string) => {}\" />; binding: @save)\n",
            "  error:14:73 [TS2339] Property 'toUpperCase' does not exist on type 'number'. (source: <Child :model-value=\"1\" kind=\"num\" :n=\"1\" v-slot=\"{ count }\">{{ count.toUpperCase() }}</Child>; binding: #default)\n",
            "\n\u{2717} Type checked 2 files in <duration> (collect: <duration>, gen: <duration>, corsa: <duration>)\n",
            "  6 error(s)\n",
        ),
        "stderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
