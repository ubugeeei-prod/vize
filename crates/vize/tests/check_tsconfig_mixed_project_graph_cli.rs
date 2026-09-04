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

use vize_s0::cstr;

#[test]
fn default_check_reports_mixed_tsconfig_program_graph_and_repair() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("mixed-tsconfig-project-graph");
    project.write(
        "vize.config.json",
        r#"{ "typeChecker": { "jsxTypecheck": true } }"#,
    );
    project.write(
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "types": ["vue/jsx"],
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    );
    project.write(
        "src/contracts.ts",
        r#"export interface WidgetProps {
  count: number
  title: string
}
"#,
    );
    project.write(
        "src/Widget.vue",
        r#"<script setup lang="ts">
import type { WidgetProps } from './contracts'

defineProps<WidgetProps>()
</script>

<template>
  <section>{{ title }} {{ count }}</section>
</template>
"#,
    );
    project.write("src/App.vue", APP_SOURCE);
    project.write("src/entry.tsx", clean_entry_source());

    let clean = project.check(&corsa_path);
    assert_eq!(
        clean.status,
        Some(0),
        "clean mixed project should pass:\n{}\n{}",
        clean.stdout,
        clean.stderr
    );
    let clean_json = parse_json(&clean);
    assert_program_files(
        &clean_json,
        &[
            "node_modules/vue/jsx.d.ts",
            "src/App.vue",
            "src/Widget.vue",
            "src/contracts.ts",
            "src/entry.tsx",
        ],
    );
    assert_eq!(clean_json["fileCount"], 4, "{}", clean.stdout);
    assert_eq!(clean_json["errorCount"], 0, "{}", clean.stdout);

    project.write("src/entry.tsx", broken_entry_source());
    let broken = project.check(&corsa_path);
    assert_eq!(
        broken.status,
        Some(1),
        "broken TSX root should fail:\n{}\n{}",
        broken.stdout,
        broken.stderr
    );
    let broken_json = parse_json(&broken);
    assert_program_files(
        &broken_json,
        &[
            "node_modules/vue/jsx.d.ts",
            "src/App.vue",
            "src/Widget.vue",
            "src/contracts.ts",
            "src/entry.tsx",
        ],
    );
    assert_eq!(broken_json["errorCount"], 1, "{}", broken.stdout);
    let entry_diagnostics = diagnostics_for(&broken_json, "src/entry.tsx");
    assert_eq!(
        entry_diagnostics,
        vec!["error:5:30 [TS2322] Type 'string' is not assignable to type 'number'."],
        "diagnostic should stay on the authored TSX root:\n{}",
        broken.stdout
    );

    project.write("src/entry.tsx", clean_entry_source());
    let repaired = project.check(&corsa_path);
    assert_eq!(
        repaired.status,
        Some(0),
        "repaired TSX root should pass again:\n{}\n{}",
        repaired.stdout,
        repaired.stderr
    );
    let repaired_json = parse_json(&repaired);
    assert_program_files(
        &repaired_json,
        &[
            "node_modules/vue/jsx.d.ts",
            "src/App.vue",
            "src/Widget.vue",
            "src/contracts.ts",
            "src/entry.tsx",
        ],
    );
    assert_eq!(repaired_json["errorCount"], 0, "{}", repaired.stdout);
}

const APP_SOURCE: &str = r#"<script setup lang="ts">
import Entry from './entry'
</script>

<template>
  <Entry />
</template>
"#;

fn clean_entry_source() -> &'static str {
    r#"import Widget from './Widget.vue';
import type { WidgetProps } from './contracts';

const props: WidgetProps = { count: 1, title: 'Ready' };
export default () => <Widget {...props} />;
"#
}

fn broken_entry_source() -> &'static str {
    r#"import Widget from './Widget.vue';
import type { WidgetProps } from './contracts';

const props: WidgetProps = { count: 1, title: 'Ready' };
export default () => <Widget count="wrong" title={props.title} />;
"#
}

struct CheckRun {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

struct ProjectCase {
    root: PathBuf,
}

impl ProjectCase {
    fn new(name: &str) -> Self {
        let case_id = cstr!("check-{name}-{}", std::process::id());
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

    fn check(&self, corsa_path: &str) -> CheckRun {
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&self.root)
            .env("CORSA_PATH", corsa_path)
            .args(["check", "--format", "json"])
            .output()
            .unwrap();
        CheckRun {
            status: output.status.code(),
            stdout: std::str::from_utf8(&output.stdout).unwrap().into(),
            stderr: std::str::from_utf8(&output.stderr).unwrap().into(),
        }
    }
}

impl Drop for ProjectCase {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn parse_json(run: &CheckRun) -> serde_json::Value {
    serde_json::from_str(&run.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse check output as JSON: {error}\nstdout:\n{}\nstderr:\n{}",
            run.stdout, run.stderr
        )
    })
}

fn assert_program_files(json: &serde_json::Value, expected: &[&str]) {
    let programs = json["programs"]
        .as_array()
        .expect("JSON output should include programs");
    assert_eq!(programs.len(), 1, "programs: {programs:#?}");
    assert_eq!(programs[0]["tsconfig"], "tsconfig.json");
    let files = programs[0]["files"]
        .as_array()
        .expect("program should include files")
        .iter()
        .map(|file| file.as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(files, expected, "program files: {files:#?}");
}

fn diagnostics_for(json: &serde_json::Value, file_name: &str) -> Vec<String> {
    json["files"]
        .as_array()
        .expect("JSON output should include files")
        .iter()
        .find(|file| file["file"] == file_name)
        .unwrap_or_else(|| panic!("missing {file_name} in files: {json:#?}"))["diagnostics"]
        .as_array()
        .expect("file should include diagnostics")
        .iter()
        .map(|diagnostic| diagnostic.as_str().unwrap().to_owned())
        .collect()
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
