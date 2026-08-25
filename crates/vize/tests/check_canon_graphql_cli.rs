#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};
use vize_s0::cstr;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn temp_case_dir(name: &str) -> tempfile::TempDir {
    let base = workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests");
    std::fs::create_dir_all(&base).expect("test base directory should be writable");
    tempfile::Builder::new()
        .prefix(&format!("check-canon-graphql-{name}-"))
        .tempdir_in(base)
        .expect("test case directory should be created")
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

/// Runs `vize check` against `project_root` in JSON mode, asserts the run
/// succeeded with zero errors, and returns its stdout for further inspection.
fn run_check_json(project_root: &Path, corsa_path: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
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
    stdout.to_string()
}

#[test]
fn check_explicit_vue_keeps_generated_graphql_schema_out_of_canon() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = temp_case_dir("dedupe");
    let project_root = project.path();
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

    run_check_json(&project_root, &corsa_path);
    assert!(
        vize_canon::project_virtual_root(&project_root)
            .join("pages/_studyInfoId.vue.ts")
            .exists()
    );
    assert!(
        !vize_canon::project_virtual_root(&project_root)
            .join("types/codegen/schema.d.ts")
            .exists()
    );
}

/// Regression for #2227/#2307: a `types/index.ts` barrel that re-exports a
/// generated GraphQL `.d.ts` via a relative `export *` is materialized into
/// canon, but the `.d.ts` is intentionally kept on its real path (#2047).
/// Previously the barrel's relative `./codegen/schema` dangled inside the
/// mirror, dropping the generated module's type identity so members re-exported
/// through `~/types` were reported as missing/unrelated, including TS1360 false
/// positives vue-tsc does not raise.
#[test]
fn check_barrel_reexport_preserves_generated_graphql_identity() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = temp_case_dir("barrel");
    let project_root = project.path();
    std::fs::create_dir_all(project_root.join("fragments")).unwrap();
    std::fs::create_dir_all(project_root.join("pages")).unwrap();
    std::fs::create_dir_all(project_root.join("types/codegen")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "paths": {
      "~/*": ["*"],
      "@/*": ["*"]
    },
    "noEmit": true,
    "types": []
  },
  "include": ["fragments/**/*.vue", "pages/**/*.vue", "types/**/*.ts", "types/**/*.d.ts"]
}"#,
    )
    .unwrap();

    std::fs::write(
        project_root.join("types/codegen/schema.d.ts"),
        r#"// Generated GraphQL schema types.
export type AimTextComponent = {
  __typename: 'AimTextComponent'
  text: string
  children: AimContentsComponent[]
}

export type AimImageComponent = {
  __typename: 'AimImageComponent'
  url: string
  children: AimContentsComponent[]
}

export type AimContentsComponent = AimTextComponent | AimImageComponent

export type AimContentsMoshiQuery = {
  components: AimContentsComponent[]
}
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("types/index.ts"),
        r#"export * from './codegen/schema'

export type UnwrapArray<T> = T extends Array<infer U> ? U : never
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("pages/_studyInfoId.vue"),
        r#"<script setup lang="ts">
import type { AimContentsMoshiQuery } from '~/types/codegen/schema'

export type AimContentsMoshi = AimContentsMoshiQuery
</script>

<template><main /></template>
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("fragments/MoshiContentsSection.vue"),
        r#"<script setup lang="ts">
import { computed } from 'vue'
import { type AimContentsMoshi } from '~/pages/_studyInfoId.vue'
import { type UnwrapArray, type AimContentsComponent } from '~/types'

type AimComponent = UnwrapArray<AimContentsMoshi['components']>

const props = defineProps<{
  component: { childMoshiContentsComponents: AimContentsComponent[] }
}>()
const childComponents = computed(
  () => props.component.childMoshiContentsComponents satisfies AimComponent[],
)
void childComponents
</script>

<template><section /></template>
"#,
    )
    .unwrap();

    let stdout = run_check_json(&project_root, &corsa_path);
    assert!(
        !stdout.contains("TS1360"),
        "generated GraphQL symbols should keep one type identity:\n{stdout}"
    );
    assert!(
        !vize_canon::project_virtual_root(&project_root)
            .join("types/codegen/schema.d.ts")
            .exists()
    );
}
