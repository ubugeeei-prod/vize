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
        .join(cstr!("check-nuxt-ambient-{name}-{}", std::process::id()).as_str())
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

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn check_tsconfig_default_run_loads_nuxt_ambient_declarations() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("import-meta");
    let _ = std::fs::remove_dir_all(&project_root);

    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "extends": "./.nuxt/tsconfig.json"
}"#,
    );
    write(
        &project_root,
        ".nuxt/tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["../app/**/*.ts", "./nuxt.d.ts"]
}"#,
    );
    write(
        &project_root,
        ".nuxt/nuxt.d.ts",
        "/// <reference path=\"types/import-meta.d.ts\" />\nexport {};\n",
    );
    write(
        &project_root,
        ".nuxt/types/import-meta.d.ts",
        "export {};\ndeclare global { interface ImportMeta { vitest: boolean; } }\n",
    );
    write(
        &project_root,
        "app/plugins/auth.ts",
        "export const runningUnderVitest: boolean = import.meta.vitest;\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
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

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check should load Nuxt ambient declarations in default tsconfig runs\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert!(
        !stdout.contains("TS2339"),
        "ImportMeta augmentation should be in scope:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_explicit_nuxt_build_dir_loads_generated_context_types() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("custom-build-dir-context");
    let _ = std::fs::remove_dir_all(&project_root);

    write(
        &project_root,
        "nuxt.config.ts",
        r#"export default defineNuxtConfig({ buildDir: ".out/.nuxt" })
"#,
    );
    write(
        &project_root,
        "tsconfig.json",
        r##"{
  "extends": "./.out/.nuxt/tsconfig.json",
  "compilerOptions": {
    "paths": {
      "#imports": [".nuxt/imports"]
    }
  }
}
"##,
    );
    write(
        &project_root,
        ".out/.nuxt/tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["../../src/**/*.ts", "./nuxt.d.ts"]
}
"#,
    );
    write(
        &project_root,
        ".out/.nuxt/nuxt.d.ts",
        r#"/// <reference path="types/context.d.ts" />
/// <reference path="types/imports.d.ts" />
export {};
"#,
    );
    write(
        &project_root,
        ".out/.nuxt/types/context.d.ts",
        r#"declare module "@nuxtjs/composition-api" {
  export interface UseContextReturn {
    $gtm: {
      track(event: string): void;
    };
  }

  export function useContext(): UseContextReturn;
}
"#,
    );
    write(
        &project_root,
        ".out/.nuxt/types/imports.d.ts",
        r#"declare global {
  const useRuntimeConfig: () => {
    public: {
      stage: string;
    };
  }
}
export {}
"#,
    );
    write(
        &project_root,
        "src/useReAuth.ts",
        r#"import { useContext } from "@nuxtjs/composition-api";

const context = useContext();
context.$gtm.track("login");

const stage: string = useRuntimeConfig().public.stage;
void stage;
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "src",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "custom Nuxt buildDir generated context should type-check\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert!(
        !stderr.contains("no generated `.nuxt` types found"),
        "custom buildDir should not fall back to default .nuxt warning:\n{stderr}"
    );
    assert!(
        !stdout.contains("TS2339") && !stdout.contains("TS2304"),
        "generated context and auto-import types should be in scope:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
