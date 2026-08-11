//! User `paths` authority over an installed same-named Vue package (#4002).

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{path::Path, process::Command};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<String> {
    corsa_path::resolve(workspace_root())
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file = root.join(path);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file, content).unwrap();
}

#[test]
fn user_paths_win_over_same_named_package_without_changing_other_aliases() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!("package-paths-authority-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project);
    write_file(
        &project,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "allowArbitraryExtensions": true,
    "baseUrl": ".",
    "paths": {
      "@scope/ui": ["src/local/LocalWidget.vue"],
      "@helpers/*": ["src/helpers/*"]
    },
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    );
    write_file(
        &project,
        "src/local/LocalWidget.vue",
        r#"<script setup lang="ts">
defineProps<{ localOnly: string }>()
</script>
"#,
    );
    write_file(
        &project,
        "src/helpers/identity.ts",
        "export const aliasIdentity = 'local-helper' as const\n",
    );
    write_file(
        &project,
        "node_modules/@scope/ui/package.json",
        r#"{
  "name": "@scope/ui",
  "exports": { ".": "./src/InstalledWidget.vue" }
}"#,
    );
    write_file(
        &project,
        "node_modules/@scope/ui/src/InstalledWidget.vue",
        r#"<script setup lang="ts">
defineProps<{ installedOnly: number }>()
</script>
"#,
    );
    write_file(
        &project,
        "src/entry.ts",
        r#"import Widget from "@scope/ui"
import { aliasIdentity } from "@helpers/identity"

type Props = InstanceType<typeof Widget>["$props"]
type HasLocal = "localOnly" extends keyof Props ? true : false
type HasInstalled = "installedOnly" extends keyof Props ? true : false

export const props: Props = { localOnly: "user-paths" }
export const hasLocal: HasLocal = true
export const hasInstalled: HasInstalled = false
export const helperIdentity: "local-helper" = aliasIdentity
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project)
        .env("CORSA_PATH", corsa)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "user paths lost native authority:\n{stdout}\n{stderr}"
    );
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["errorCount"], 0, "{stdout}\n{stderr}");

    let _ = std::fs::remove_dir_all(&project);
}
