use std::{fs, path::Path, process::Command};

const PINIA_RULE: &str = "ecosystem/pinia-prefer-store-to-refs";

fn write_file(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

fn run_lint(root: &Path, files: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .arg("lint")
        .args(files)
        .args([
            "--no-config",
            "--preset",
            "ecosystem",
            "--format",
            "json",
            "--help-level",
            "none",
        ])
        .output()
        .unwrap()
}

fn output_details(output: &std::process::Output) -> vize_s0::String {
    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8 stdout>");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8 stderr>");
    vize_s0::cstr!("stdout:\n{stdout}\nstderr:\n{stderr}")
}

fn pinia_diagnostic_count(output: &std::process::Output) -> usize {
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    report
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|file| file.get("messages")?.as_array())
        .flatten()
        .filter(|message| {
            message.get("ruleId").and_then(|value| value.as_str()) == Some(PINIA_RULE)
        })
        .count()
}

#[test]
fn pinia_store_rule_follows_project_dependency_resolution() {
    let project = tempfile::tempdir().unwrap();
    write_file(project.path(), "package.json", r#"{"private":true}"#);
    write_file(
        project.path(),
        "src/store.ts",
        r#"import { inject, type Ref } from "vue"

export interface CounterStore {
  count: Ref<number>
  actions: { inc(): void }
}

export function useCounterStore(): CounterStore {
  return inject("counter") as CounterStore
}
"#,
    );
    write_file(
        project.path(),
        "src/SameFile.vue",
        r#"<script setup lang="ts">
import { ref } from "vue"
function useCounterStore() {
  return { count: ref(0), actions: { inc() {} } }
}
const { count, actions } = useCounterStore()
</script>
"#,
    );
    write_file(
        project.path(),
        "src/Imported.vue",
        r#"<script setup lang="ts">
import { useCounterStore } from "./store"
const { count, actions } = useCounterStore()
</script>
"#,
    );

    let without_pinia = run_lint(project.path(), &["src/SameFile.vue", "src/Imported.vue"]);
    assert!(
        without_pinia.status.success(),
        "{}",
        output_details(&without_pinia),
    );
    assert_eq!(
        pinia_diagnostic_count(&without_pinia),
        0,
        "{}",
        output_details(&without_pinia),
    );

    write_file(
        project.path(),
        "node_modules/pinia/package.json",
        r#"{"name":"pinia","version":"3.0.0"}"#,
    );
    write_file(
        project.path(),
        "src/store.ts",
        r#"import { defineStore } from "pinia"

export const useCounterStore = defineStore("counter", {
  state: () => ({ count: 0 }),
  actions: { inc() { this.count += 1 } },
})
"#,
    );

    let with_pinia = run_lint(project.path(), &["src/Imported.vue"]);
    assert!(
        with_pinia.status.success(),
        "{}",
        output_details(&with_pinia)
    );
    assert_eq!(
        pinia_diagnostic_count(&with_pinia),
        1,
        "{}",
        output_details(&with_pinia),
    );
}
