use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn check_nuxt2_use_context_sees_plugin_injections() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_project("nuxt2-plugin-injections");

    write_file(&project_root, "nuxt.config.ts", "export default {}\n");
    write_file(
        &project_root,
        "tsconfig.json",
        r##"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "noEmit": true
  },
  "include": ["pages/**/*.vue", "plugins/**/*.ts", "types/**/*.d.ts"]
}"##,
    );
    write_file(
        &project_root,
        "types/nuxt.d.ts",
        r##"declare module "@nuxt/types" {
  export interface Context {}
  export interface NuxtAppOptions {}
}

declare module "@nuxtjs/composition-api" {
  export interface UseContextReturn
    extends Omit<import("@nuxt/types").Context, "route" | "query" | "from" | "params"> {}
  export function useContext(): UseContextReturn;
}

declare module "#app" {
  export interface NuxtApp {}
}
"##,
    );
    write_file(
        &project_root,
        "plugins/logger.ts",
        r#"export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  if (true) {
    inject("logger", {
      info(message: string) {
        return message.length;
      },
    });
  }
};
"#,
    );
    write_file(
        &project_root,
        "pages/index.vue",
        r#"<script setup lang="ts">
import { useContext } from "@nuxtjs/composition-api";

const context = useContext();
context.$logger.info("ready");
</script>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "pages",
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
        "Nuxt2 useContext plugin injections should type-check\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[cfg(feature = "legacy")]
#[test]
fn check_nuxt2_false_positive_fixture_matrix() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_project("nuxt2-false-positive-matrix");
    write_false_positive_fixture(&project_root);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "components",
            "pages",
            "--config",
            "vize.config.json",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    assert!(
        output.status.success(),
        "Nuxt2 false-positive matrix should type-check\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["errorCount"], 0);
    assert_eq!(json["fileCount"], 3);
    let files = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["file"].as_str().unwrap().to_owned(),
                file["diagnostics"].as_array().unwrap().clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        files,
        vec![
            ("components/KeyboardPanel.vue".to_owned(), Vec::new()),
            ("components/OverlayPanel.vue".to_owned(), Vec::new()),
            ("pages/index.vue".to_owned(), Vec::new()),
        ]
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[cfg(feature = "legacy")]
const FALSE_POSITIVE_FIXTURES: &[(&str, &str)] = &[
    (
        "vize.config.json",
        include_str!("fixtures/nuxt2_false_positive/vize.config.json"),
    ),
    (
        "nuxt.config.ts",
        include_str!("fixtures/nuxt2_false_positive/nuxt.config.ts"),
    ),
    (
        "tsconfig.json",
        include_str!("fixtures/nuxt2_false_positive/tsconfig.json"),
    ),
    (
        "types/nuxt.d.ts",
        include_str!("fixtures/nuxt2_false_positive/types/nuxt.d.ts"),
    ),
    (
        "plugins/logger.ts",
        include_str!("fixtures/nuxt2_false_positive/plugins/logger.ts"),
    ),
    (
        "plugins/auth.ts",
        include_str!("fixtures/nuxt2_false_positive/plugins/auth.ts"),
    ),
    (
        "plugins/gtm.ts",
        include_str!("fixtures/nuxt2_false_positive/plugins/gtm.ts"),
    ),
    (
        "plugins/repository.ts",
        include_str!("fixtures/nuxt2_false_positive/plugins/repository.ts"),
    ),
    (
        "components/KeyboardPanel.vue",
        include_str!("fixtures/nuxt2_false_positive/components/KeyboardPanel.vue"),
    ),
    (
        "components/OverlayPanel.vue",
        include_str!("fixtures/nuxt2_false_positive/components/OverlayPanel.vue"),
    ),
    (
        "pages/index.vue",
        include_str!("fixtures/nuxt2_false_positive/pages/index.vue"),
    ),
];

#[cfg(feature = "legacy")]
fn write_false_positive_fixture(project_root: &Path) {
    for (path, contents) in FALSE_POSITIVE_FIXTURES {
        write_file(project_root, path, contents);
    }
}

fn create_project(name: &str) -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    link_workspace_node_modules(&project_root);
    project_root
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn link_workspace_node_modules(project_root: &Path) {
    let source = workspace_root().join("node_modules");
    if source.exists() {
        symlink_path(&source, &project_root.join("node_modules")).unwrap();
    }
}

fn resolve_test_corsa_path() -> Option<String> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path.display().to_string());
        }
    }
    let workspace_root = workspace_root();
    [workspace_root.join("node_modules/.bin/tsgo")]
        .into_iter()
        .find(|candidate| candidate.exists())
        .map(|candidate| candidate.display().to_string())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}
