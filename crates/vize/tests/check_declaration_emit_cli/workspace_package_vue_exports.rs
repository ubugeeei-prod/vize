use super::*;

#[test]
fn workspace_package_vue_exports_emit_declarations_with_package_identity() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let case_root = unique_case_dir("declaration-workspace-vue-export");
    let _ = std::fs::remove_dir_all(&case_root);
    let app_root = case_root.join("app");
    let package_root = case_root.join("packages/workspace-vue");
    std::fs::create_dir_all(app_root.join("src")).unwrap();
    std::fs::create_dir_all(package_root.join("src")).unwrap();
    std::fs::write(
        app_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "rootDir": "src",
    "declaration": true,
    "declarationMap": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        app_root.join("src/index.ts"),
        r#"export { default as WorkspaceWidget } from '@scope/workspace-vue'
export type { WorkspaceWidgetProps } from '@scope/workspace-vue'
export type WorkspaceWidgetModule = typeof import('@scope/workspace-vue')
"#,
    )
    .unwrap();
    std::fs::write(
        package_root.join("package.json"),
        r#"{
  "name": "@scope/workspace-vue",
  "exports": {
    ".": {
      "types": "./src/Root.vue",
      "default": "./src/Root.vue"
    }
  }
}"#,
    )
    .unwrap();
    std::fs::write(
        package_root.join("src/Root.vue"),
        r#"<script setup lang="ts">
export interface WorkspaceWidgetProps {
  count: number
}
defineProps<WorkspaceWidgetProps>()
</script>
<template>{{ count }}</template>
"#,
    )
    .unwrap();
    link_workspace_package(
        &package_root,
        &app_root.join("node_modules/@scope/workspace-vue"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&app_root)
        .env("CORSA_PATH", corsa_path.as_str())
        .args([
            "check",
            ".",
            "--format",
            "json",
            "--declaration",
            "--declaration-dir",
            "types",
        ])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    let declarations = json["declarations"]
        .as_array()
        .expect("declarations should be an array");
    assert!(
        declarations
            .iter()
            .filter_map(serde_json::Value::as_str)
            .all(|path| !path.contains("__vize_external__") && !path.contains(".vize")),
        "declaration paths leaked the virtual mirror:\n{stdout}\n{stderr}"
    );
    let index_declaration_path = declarations
        .iter()
        .filter_map(serde_json::Value::as_str)
        .find(|path| path.ends_with("/types/index.d.ts") || *path == "types/index.d.ts")
        .unwrap_or_else(|| panic!("missing index declaration:\n{stdout}\n{stderr}"));
    let index_declaration = std::fs::read_to_string(app_root.join(index_declaration_path)).unwrap();
    assert!(
        index_declaration.contains("@scope/workspace-vue"),
        "declaration must preserve package identity:\n{index_declaration}"
    );
    for leaked in ["__vize_external__", ".vize", ".vue.ts"] {
        assert!(
            !index_declaration.contains(leaked),
            "declaration leaked {leaked}:\n{index_declaration}"
        );
    }
    let map_path = app_root.join("types/index.d.ts.map");
    let map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&map_path).unwrap()).unwrap();
    assert_eq!(map["sourceRoot"], "");
    let source = map["sources"][0].as_str().unwrap();
    let mapped_source =
        vize_s0::path::canonicalize_non_verbatim(&map_path.parent().unwrap().join(source));
    assert_eq!(
        mapped_source,
        vize_s0::path::canonicalize_non_verbatim(&app_root.join("src/index.ts")),
        "declaration map must survive rootDir layout restoration: {map:#?}"
    );
    assert!(
        !app_root.join("types/__vize_external__").exists(),
        "inferred workspace-package declarations must be pruned"
    );

    let _ = std::fs::remove_dir_all(case_root);
}

fn link_workspace_package(source: &Path, target: &Path) {
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}
