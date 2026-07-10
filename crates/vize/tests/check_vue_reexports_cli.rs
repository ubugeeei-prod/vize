use std::{path::Path, process::Command};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(format!("{name}-{}-{case_id}", std::process::id()))
}

#[cfg(unix)]
fn link_workspace_node_modules(project_root: &Path) {
    let source = workspace_root().join("node_modules");
    let target = project_root.join("node_modules");
    if target.exists() {
        return;
    }
    std::os::unix::fs::symlink(source, target).unwrap();
}

fn resolve_test_corsa_path() -> Option<String> {
    let workspace_root = workspace_root();
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.display().to_string());
    }

    let workspace_bin = workspace_root.join("node_modules/.bin/tsgo");
    workspace_bin
        .exists()
        .then(|| workspace_bin.display().to_string())
}

#[cfg(unix)]
#[test]
fn check_preserves_named_exports_from_vue_reexported_through_symlinked_absolute_path() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };

    let real_root = unique_case_dir("vue-reexport-real-root");
    let link_root =
        real_root.with_file_name(format!("vue-reexport-link-root-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&real_root);
    let _ = std::fs::remove_file(&link_root);
    let _ = std::fs::remove_dir_all(&link_root);
    std::fs::create_dir_all(&real_root).unwrap();
    std::os::unix::fs::symlink(&real_root, &link_root).unwrap();
    link_workspace_node_modules(&real_root);

    std::fs::create_dir_all(real_root.join(".nuxt")).unwrap();
    std::fs::create_dir_all(real_root.join("app")).unwrap();
    std::fs::create_dir_all(real_root.join("design/components")).unwrap();

    std::fs::write(
        real_root.join("tsconfig.vize.json"),
        r##"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "baseUrl": ".",
    "paths": {
      "#design": [".nuxt/design.ts"]
    }
  },
  "include": ["app/**/*", "design/**/*", ".nuxt/design.ts", "vite-env.d.ts"]
}"##,
    )
    .unwrap();
    std::fs::write(
        real_root.join("vite-env.d.ts"),
        r#"declare module "*.vue" {
  const component: unknown;
  export default component;
}
"#,
    )
    .unwrap();
    std::fs::write(
        real_root.join("design/components/Form.vue"),
        r#"<script lang="ts">
export interface FormProps {
  modelValue: string;
}

export type FormSubmitEvent = {
  value: string;
};

export const FormContextInjectionKey = Symbol("form");

export function useFormField(event: FormSubmitEvent): string {
  return event.value;
}

export default {};
</script>

<template>
  <form />
</template>
"#,
    )
    .unwrap();

    let form_path = link_root.join("design/components/Form.vue");
    std::fs::write(
        real_root.join(".nuxt/design.ts"),
        format!(
            r#"export {{ default as Form }} from "{}";
export * from "{}";
"#,
            form_path.display(),
            form_path.display()
        ),
    )
    .unwrap();
    std::fs::write(
        real_root.join("app/use-form.ts"),
        r##"import { Form, FormContextInjectionKey, useFormField } from "#design";
import type { FormProps, FormSubmitEvent } from "#design";

const props: FormProps = { modelValue: "ok" };
const event: FormSubmitEvent = { value: props.modelValue };
const value: string = useFormField(event);

void Form;
void FormContextInjectionKey;
void value;
"##,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&link_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.vize.json",
            "app",
            "design",
            "vite-env.d.ts",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert!(!stdout.contains("TS2305"), "{stdout}\n{stderr}");
    assert!(!stderr.contains("TS2305"), "{stdout}\n{stderr}");

    let _ = std::fs::remove_file(&link_root);
    let _ = std::fs::remove_dir_all(&real_root);
}
