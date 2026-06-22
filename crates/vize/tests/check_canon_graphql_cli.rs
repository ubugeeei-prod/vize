use std::{
    path::{Path, PathBuf},
    process::Command,
};
use vize_carton::cstr;

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
        .join(cstr!("check-canon-graphql-{name}-{}", std::process::id()).as_str())
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

#[test]
fn check_explicit_vue_keeps_generated_graphql_schema_out_of_canon() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("dedupe");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::create_dir_all(project_root.join("fragments")).unwrap();
    std::fs::create_dir_all(project_root.join("pages")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "paths": {
      "~/*": ["*"]
    },
    "noEmit": true
  },
  "include": ["fragments/**/*.vue", "pages/**/*.vue", "types/**/*.d.ts"]
}"#,
    )
    .unwrap();

    let schema_path = project_root.join("types/codegen/schema.d.ts");
    let schema_path_text = schema_path.to_string_lossy().replace('\\', "/");
    let schema_specifier = schema_path_text
        .strip_suffix(".d.ts")
        .expect("schema path should end with .d.ts");
    std::fs::create_dir_all(schema_path.parent().unwrap()).unwrap();
    std::fs::write(
        &schema_path,
        r#"// Generated GraphQL schema types.
export enum AimQuestionDisplayKind {
  Text = 'TEXT',
}

export type AimQuestion = {
  kind: AimQuestionDisplayKind
}
"#,
    )
    .unwrap();

    std::fs::write(
        project_root.join("pages/_studyInfoId.vue"),
        r#"<script setup lang="ts">
import type { AimQuestion } from '~/types/codegen/schema'

export type AimContentsMoshi = {
  components: AimQuestion[]
}
</script>

<template><main /></template>
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("fragments/MoshiContentsSection.vue"),
        cstr!(
            r#"<script setup lang="ts">
import type {{ AimContentsMoshi }} from '~/pages/_studyInfoId.vue'
import {{ type AimQuestion }} from '{schema_specifier}'

type AimComponent = AimContentsMoshi['components'][number]

const props = defineProps<{{
  component: {{ childMoshiContentsComponents: AimQuestion[] }}
}}>()
const childComponents = props.component.childMoshiContentsComponents satisfies AimComponent[]
void childComponents
</script>

<template><section /></template>
"#
        ),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "--no-check-props",
            "--no-check-emits",
            "--no-check-template-bindings",
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
    assert!(
        project_root
            .join("node_modules/.vize/canon/pages/_studyInfoId.vue.ts")
            .exists()
    );
    assert!(
        !project_root
            .join("node_modules/.vize/canon/types/codegen/schema.d.ts")
            .exists()
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
