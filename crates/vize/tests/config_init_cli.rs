//! Public setup commands preserve the complete project artifact inventory.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

const VITE: &str = include_str!("fixtures/config-init/vite.config.mjs");
const BUTTON: &str = include_str!("fixtures/config-init/button.ts");
const REGISTRY: &str = include_str!("fixtures/config-init/registry.json");
const STORY: &str = r#"<script setup lang="ts">
defineArt("../src/Button.vue", {
  title: "Button",
  category: "Components",
  tags: ["button", "ui"],
});
</script>

<art>
  <variant name="Primary" default>
    <Button variant="primary">Click me</Button>
  </variant>

  <variant name="Secondary">
    <Button variant="secondary">Click me</Button>
  </variant>

  <variant name="Disabled">
    <Button variant="primary" disabled>Disabled</Button>
  </variant>
</art>

<style scoped>
.art-preview {
  padding: 0.5rem 1rem;
  display: flex;
  gap: 0.75rem;
  align-items: center;
}
</style>
"#;
const CREATED_VITE: &str = r#"export default {
  "vize": {
    "lib": {
      "composableDir": "src/composables/vize",
      "uiDir": "src/ui"
    }
  }
};
"#;

fn run(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()
        .expect("failed to execute the public vize CLI")
}

fn text(value: &str) -> Value {
    json!({"text":value})
}
fn json_file(value: Value) -> Value {
    json!({"json":value})
}
fn directory() -> Value {
    json!({"directory":true})
}

// Preserve every directory and file. JSON whitespace is normalized by parsing
// the whole document; source files retain every byte. Unexpected artifacts fail
// the map equality, including any dedicated configuration or cache directory.
fn inventory(root: &Path) -> BTreeMap<String, Value> {
    fn visit(root: &Path, dir: &Path, result: &mut BTreeMap<String, Value>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let kind = entry.file_type().unwrap();
            let artifact = if kind.is_dir() {
                directory()
            } else if kind.is_symlink() {
                json!({"symlink":fs::read_link(&path).unwrap().to_string_lossy()})
            } else {
                let bytes = fs::read_to_string(&path).unwrap();
                if path
                    .extension()
                    .is_some_and(|extension| extension == "json")
                {
                    json_file(serde_json::from_str(&bytes).unwrap())
                } else {
                    text(&bytes)
                }
            };
            result.insert(name, artifact);
            if kind.is_dir() {
                visit(root, &path, result);
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

fn expected(entries: impl IntoIterator<Item = (&'static str, Value)>) -> BTreeMap<String, Value> {
    entries
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

fn musea_stderr(name: &str) -> String {
    format!(
        "vize musea new: Creating Musea project '{name}'...\n  Created stories/Button.art.vue\n\nMusea project '{name}' created successfully!\n\nNext steps:\n  1. Add more art files in the 'stories' directory\n  2. Enable @vizejs/vite-plugin-musea in your Vite or Nuxt project\n     Musea discovers .art.vue files by default; configure vize.musea in vite.config.* when needed\n"
    )
}

#[test]
fn musea_new_uses_defaults_and_preserves_existing_config_files() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::write(root.join("vite.config.mjs"), VITE).unwrap();
    let output = run(root, &["musea", "new", "gallery"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        musea_stderr("gallery")
    );
    assert_eq!(
        inventory(root),
        expected([
            ("stories", directory()),
            ("stories/Button.art.vue", text(STORY)),
            ("vite.config.mjs", text(VITE)),
        ])
    );

    let existing = json!({"musea":{"include":["stories/**/*.art.vue"]}});
    fs::write(
        root.join("vize.config.json"),
        serde_json::to_vec(&existing).unwrap(),
    )
    .unwrap();
    let output = run(root, &["musea", "new"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        musea_stderr(root.file_name().unwrap().to_str().unwrap())
    );
    assert_eq!(
        inventory(root),
        expected([
            ("stories", directory()),
            ("stories/Button.art.vue", text(STORY)),
            ("vite.config.mjs", text(VITE)),
            ("vize.config.json", json_file(existing)),
        ])
    );
}

fn init_report(action: &str, ui_dir: &str) -> Value {
    json!({
        "dryRun":false,"sourceDir":"src","framework":"vue","typescript":false,
        "allowImportingTsExtensions":false,"configPath":"vite.config.mjs","action":action,
        "lib":{"uiDir":ui_dir,"composableDir":"src/composables/vize"},
        "hints":["add `vue` (^3.5.0) to package.json; pulled items import it"]
    })
}

#[test]
fn library_init_writes_and_reads_vite_lib_settings_without_a_dedicated_config() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    let output = run(
        root,
        &["lib", "--json", "--offline", "init", "--ui-dir", "src/ui"],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stderr, b"");
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        init_report("create", "src/ui")
    );
    assert_eq!(
        inventory(root),
        expected([("vite.config.mjs", text(CREATED_VITE))])
    );

    let registry = root.join("registry");
    fs::create_dir_all(registry.join("files")).unwrap();
    fs::write(registry.join("files/button.ts"), BUTTON).unwrap();
    fs::write(registry.join("registry.json"), REGISTRY).unwrap();
    let output = run(
        root,
        &[
            "lib",
            "--json",
            "--offline",
            "--registry",
            registry.to_str().unwrap(),
            "pull",
            "button",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stderr, b"");
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!({
            "command":"pull","dryRun":false,"applied":true,"conflicts":null,"missingNpmDependencies":[],
            "items":[{"kind":"ui","name":"button","direct":true,"fromVersion":null,"toVersion":"1.0.0","dir":"src/ui","files":[{"path":"button.ts","action":"create"}],"npmDependencies":[]}]
        })
    );
    let lock = json!({"lockfileVersion":1,"items":[{
        "name":"button","kind":"ui","package":"@vizejs/ui","version":"1.0.0",
        "contentHash":"a7609c62fef628a9db16b94a0d377841038b83ffb52160b0ac04c0d1426ddf5d",
        "dir":"src/ui","direct":true,"registryDependencies":[],
        "files":{"button.ts":"4dccadb6ee454a00bd5d2817fcdf543b38b51955cb7890bf1c11056afd332765"}
    }]});
    assert_eq!(
        inventory(root),
        expected([
            ("registry", directory()),
            ("registry/files", directory()),
            ("registry/files/button.ts", text(BUTTON)),
            (
                "registry/registry.json",
                json_file(serde_json::from_str(REGISTRY).unwrap())
            ),
            ("src", directory()),
            ("src/ui", directory()),
            ("src/ui/button.ts", text(BUTTON)),
            ("vite.config.mjs", text(CREATED_VITE)),
            ("vize-lib.lock.json", json_file(lock)),
        ])
    );
}

#[test]
fn library_init_preserves_an_existing_vite_configuration() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::write(root.join("vite.config.mjs"), VITE).unwrap();
    let output = run(root, &["lib", "--json", "--offline", "init"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stderr, b"");
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        init_report("unchanged", "src/components/vize")
    );
    assert_eq!(inventory(root), expected([("vite.config.mjs", text(VITE))]));
}
