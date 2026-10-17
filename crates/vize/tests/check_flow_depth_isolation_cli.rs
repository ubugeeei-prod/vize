#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]

//! A long `v-if`/`v-else-if` chain must not exhaust TypeScript's control-flow
//! analysis. When a generated template body does, the checker reports TS2563
//! on it and stops flow typing for every file it checks afterwards, so an
//! unrelated `.ts` file loses contextual typing. Expectations are the
//! `vue-tsc` result recorded in the fixture README: no diagnostics.

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/typechecker/flow-depth-isolation/",
            $name
        ))
    };
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

fn write(root: &Path, file: &str, source: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

#[test]
fn a_long_else_if_chain_keeps_flow_typing_for_later_files() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let parent = workspace_root().join("npm/cli/target/vize-tests/flow-depth-isolation");
    std::fs::create_dir_all(&parent).unwrap();
    let case = tempfile::tempdir_in(parent).unwrap();
    write(case.path(), "tsconfig.json", fixture!("tsconfig.json.txt"));
    write(case.path(), "src/Chain.vue", fixture!("Chain.vue.txt"));
    write(case.path(), "src/Child.vue", fixture!("Child.vue.txt"));
    write(case.path(), "src/later.ts", fixture!("later.ts.txt"));

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(case.path())
        .env("CORSA_PATH", &corsa)
        .args(["check", "--no-config", "--format", "json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let json: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("{error}\n{stdout}\n{stderr}"));

    let reported: Vec<String> = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap().replace('\\', "/");
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |diagnostic| format!("{name} {}", diagnostic.as_str().unwrap()))
        })
        .collect();
    assert_eq!(reported, Vec::<String>::new(), "{stderr}");
    assert_eq!(json["errorCount"], 0, "{stdout}");
    assert!(output.status.success(), "{stdout}\n{stderr}");
}

const ISSUE_CONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true, "target": "ESNext", "module": "ESNext",
    "moduleResolution": "Bundler", "jsx": "preserve", "noEmit": true,
    "skipLibCheck": true
  },
  "vueCompilerOptions": { "strictTemplates": true },
  "include": ["src/**/*.vue", "src/**/*.ts"]
}
"#;

// Original source bytes from https://github.com/ubugeeei-prod/vize/issues/9360.
const ISSUE_ANY: &str = r#"<script setup lang="ts">
defineProps<{ item: unknown }>()
</script>

<template><div /></template>
"#;
const ISSUE_ONLY: &str = r#"<script setup lang="ts">
defineProps<{ item: { kind: 'b'; n: number } }>()
</script>

<template><div /></template>
"#;
const ISSUE_PAGE: &str = r#"<script setup lang="ts">
import Any from './Any.vue'
import Only from './Only.vue'
type Item = { kind: 'a'; s: string } | { kind: 'b'; n: number }
defineProps<{ item: Item; ready: boolean }>()
</script>

<template>
  <div v-if="ready">
    <Any v-if="item.kind === 'a'" :item="item" />
    <Only v-else :item="item" />
  </div>
</template>
"#;

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Use the installed Vue package through ordinary ancestor resolution. Keep the
/// complete authored inventory, report, process result and source bytes together.
fn assert_project(name: &str, files: &[(&str, &str)], page_diagnostics: &[&str]) {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let vue_path = workspace_root().join("npm/cli/node_modules/vue/package.json");
    let vue =
        std::fs::read(&vue_path).expect("the genuine installed CLI Vue dependency is required");
    assert_eq!(
        serde_json::from_slice::<Value>(&vue).unwrap()["name"],
        "vue"
    );
    let parent = workspace_root().join("npm/cli/target/vize-tests/flow-depth-isolation");
    std::fs::create_dir_all(&parent).unwrap();
    let case = tempfile::tempdir_in(parent).unwrap();
    let root = case.path().canonicalize().unwrap();
    write(&root, "tsconfig.json", ISSUE_CONFIG);
    for &(file, source) in files {
        write(&root, file, source);
    }
    let mut names: Vec<_> = files.iter().map(|(name, _)| *name).collect();
    names.sort_unstable();
    let expected_files: Vec<_> = names
        .iter()
        .map(|name| {
            let diagnostics = if *name == "src/Page.vue" {
                page_diagnostics
            } else {
                &[]
            };
            json!({"file": name, "diagnostics": diagnostics})
        })
        .collect();
    let config: Value = serde_json::from_str(ISSUE_CONFIG).unwrap();
    let expected = json!({
        "files": expected_files,
        "programs": [{"root": ".", "tsconfig": "tsconfig.json",
            "compilerOptions": config["compilerOptions"], "files": names}],
        "errorCount": page_diagnostics.len(), "warningCount": 0, "fileCount": files.len()
    });
    let args = ["check", "src", "--no-config", "--quiet", "--format", "json"];
    let removed_env = [
        "VIZE_VUE_PACKAGE",
        "VIZE_VUE_NAMESPACE_PACKAGE",
        "VIZE_VUE_RUNTIME_DOM_PACKAGE",
        "VIZE_RUNTIME_NODE_MODULES",
    ];
    let binary = env!("CARGO_BIN_EXE_vize");
    let mut command = Command::new(binary);
    command
        .current_dir(&root)
        .env("CORSA_PATH", &corsa)
        .args(args);
    for variable in removed_env {
        command.env_remove(variable);
    }
    let output = command.output().unwrap();
    let receipt_root = workspace_root().join("target/vize-tests/flow-depth-isolation");
    std::fs::create_dir_all(&receipt_root).unwrap();
    let sources: Vec<_> = std::iter::once(("tsconfig.json", ISSUE_CONFIG))
        .chain(files.iter().copied())
        .map(|(path, source)| {
            json!({"path": path, "source": source,
            "sha256": sha256(source.as_bytes())})
        })
        .collect();
    let receipt = json!({"issue":9360,"case":name,"cwd":root,
        "argv":std::iter::once(binary).chain(args).collect::<Vec<_>>(),
        "binary":{"path":binary,"sha256":sha256(&std::fs::read(binary).unwrap())},
        "corsa":{"path":corsa,"sha256":sha256(&std::fs::read(&corsa).unwrap())},
        "environment":{"CORSA_PATH":corsa},"status":format!("{:?}",output.status),
        "removedEnv":removed_env,"vue":{"path":vue_path,"manifest":serde_json::from_slice::<Value>(&vue).unwrap()},
        "sources":sources,"expected":expected,"exitCode":output.status.code(),
        "stdoutBytes":output.stdout,"stderrBytes":output.stderr,
        "stdout":std::str::from_utf8(&output.stdout).ok(),
        "stderr":std::str::from_utf8(&output.stderr).ok()});
    std::fs::write(
        receipt_root.join(format!("{name}.json")),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
    eprintln!("issue-9360-cli-receipt {receipt}");
    assert_eq!(
        output.status.code(),
        Some(i32::from(!page_diagnostics.is_empty())),
        "{receipt}"
    );
    assert_eq!(output.stderr, b"", "{receipt}");
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        expected,
        "{receipt}"
    );
    for &(file, source) in files {
        assert_eq!(std::fs::read(root.join(file)).unwrap(), source.as_bytes());
    }
    assert_eq!(
        std::fs::read(root.join("tsconfig.json")).unwrap(),
        ISSUE_CONFIG.as_bytes()
    );
}

fn assert_issue(name: &str, page: &str, diagnostics: &[&str]) {
    assert_project(
        name,
        &[
            ("src/Any.vue", ISSUE_ANY),
            ("src/Only.vue", ISSUE_ONLY),
            ("src/Page.vue", page),
        ],
        diagnostics,
    );
}

#[test]
fn nested_else_preserves_the_original_issue_and_its_positive_and_negative_controls() {
    assert_issue("original-nested", ISSUE_PAGE, &[]);
    assert_issue(
        "without-parent-guard",
        &ISSUE_PAGE.replace("<div v-if=\"ready\">", "<div>"),
        &[],
    );
    let unguarded = ISSUE_PAGE.replace(
        "  <div v-if=\"ready\">\n    <Any v-if=\"item.kind === 'a'\" :item=\"item\" />\n    <Only v-else :item=\"item\" />\n  </div>",
        "  <Only :item=\"item\" />",
    );
    assert_issue(
        "without-guards",
        &unguarded,
        &[
            "error:9:10 [TS2322] Type 'Item' is not assignable to type '{ kind: \"b\"; n: number; }'.\nProperty 'n' is missing in type '{ kind: \"a\"; s: string; }' but required in type '{ kind: \"b\"; n: number; }'.",
        ],
    );
    assert_issue(
        "literal-wrong-prop",
        &ISSUE_PAGE.replace("<Only v-else :item=\"item\"", "<Only v-else :item=\"42\""),
        &[
            "error:11:19 [TS2322] Type 'number' is not assignable to type '{ kind: \"b\"; n: number; }'.",
        ],
    );
}

#[test]
fn all_prior_else_if_conditions_narrow_every_prop_of_siblings_in_the_final_branch() {
    let only = r#"<script setup lang="ts">
defineProps<{ item: { kind: 'c'; flag: boolean }; flag: boolean }>()
</script>
<template><div /></template>
"#;
    let page = r#"<script setup lang="ts">
import Any from './Any.vue'
import Only from './Only.vue'
type Item = { kind: 'a'; s: string } | { kind: 'b'; n: number } | { kind: 'c'; flag: boolean }
defineProps<{ item: Item; ready: boolean }>()
</script>
<template>
  <div v-if="ready">
    <Any v-if="item.kind === 'a'" :item="item" />
    <Any v-else-if="item.kind === 'b'" :item="item" />
    <template v-else>
      <Only :item="item" :flag="item.flag" />
      <Only :item="item" :flag="item.flag" />
    </template>
  </div>
</template>
"#;
    assert_project(
        "all-prior-conditions-and-siblings",
        &[
            ("src/Any.vue", ISSUE_ANY),
            ("src/Only.vue", only),
            ("src/Page.vue", page),
        ],
        &[],
    );
}

#[test]
fn scoped_slot_generic_props_and_statement_handlers_keep_parent_and_sibling_narrowing() {
    let provider = r#"<script setup lang="ts">
type Item = { kind: 'a'; s: string } | { kind: 'b'; n: number }
defineProps<{ item: Item }>()
defineSlots<{ default(props: { item: Item }): unknown }>()
</script>
<template><slot :item="item" /></template>
"#;
    let generic = r#"<script setup lang="ts" generic="T extends { kind: 'b'; n: number }">
defineProps<{ item: T; amount: number }>()
defineEmits<{ choose: [item: T] }>()
</script>
<template><div /></template>
"#;
    let page = r#"<script setup lang="ts">
import Any from './Any.vue'
import Generic from './Generic.vue'
import Provider from './Provider.vue'
type Item = { kind: 'a'; s: string } | { kind: 'b'; n: number }
defineProps<{ item: Item; ready: boolean }>()
function take(value: number) { return value }
</script>
<template>
  <Provider :item="item" v-slot="{ item: slotted }">
    <div v-if="ready">
      <Any v-if="slotted.kind === 'a'" :item="slotted" />
      <Generic v-else :item="slotted" :amount="slotted.n"
        @choose="take($event.n); take(slotted.n)" />
    </div>
  </Provider>
</template>
"#;
    assert_project(
        "slot-generic-statement",
        &[
            ("src/Any.vue", ISSUE_ANY),
            ("src/Generic.vue", generic),
            ("src/Page.vue", page),
            ("src/Provider.vue", provider),
        ],
        &[],
    );
}

#[test]
fn the_original_long_chain_inside_a_parent_guard_keeps_later_file_contextual_typing() {
    let chain = fixture!("Chain.vue.txt")
        .replace(
            "defineProps<{ item: Item }>();",
            "defineProps<{ item: Item; ready: boolean }>();",
        )
        .replace("<template>\n", "<template>\n  <div v-if=\"ready\">\n")
        .replace("</template>", "  </div>\n</template>");
    assert_project(
        "long-chain-parent-guard",
        &[
            ("src/Chain.vue", &chain),
            ("src/Child.vue", fixture!("Child.vue.txt")),
            ("src/later.ts", fixture!("later.ts.txt")),
        ],
        &[],
    );
}
