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
        .join(
            cstr!(
                "check-nuxt-composition-api-plugin-{name}-{}",
                std::process::id()
            )
            .as_str(),
        )
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
fn check_resolves_latest_nuxt2_composition_api_exports_with_plugin_augmentation() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("exports");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("components")).unwrap();
    write(&project_root, "nuxt.config.ts", "export default {};\n");
    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["components/**/*.vue", "plugins/**/*.ts", "types/**/*.d.ts"]
}"#,
    );
    write(
        &project_root,
        "types/nuxt.d.ts",
        r##"declare module "@nuxt/types" {
  export interface Context {}
  export interface NuxtAppOptions {}
}

declare module "#app" {
  export interface NuxtApp {}
}
"##,
    );
    write(
        &project_root,
        "plugins/logger.ts",
        r#"export default (_context: unknown, inject: (key: string, value: unknown) => void) => {
  inject("logger", {
    info(message: string) {
      return message.length;
    },
  });
};
"#,
    );
    write_latest_composition_api_package(&project_root);
    write(
        &project_root,
        "components/AppDialog.vue",
        r#"<template>
  <p>{{ doubled }}</p>
</template>

<script lang="ts">
import {
  computed,
  defineComponent,
  PropType,
  ref,
  useContext,
  useFetch,
  useRoute,
  useRouter,
  useStore,
  watch,
} from "@nuxtjs/composition-api";

export default defineComponent({
  props: { label: String as PropType<string> },
  setup() {
    const count = ref(1);
    const doubled = computed(() => count.value * 2);
    const context = useContext();
    const fetchState = useFetch(() => undefined);
    const route = useRoute();
    const router = useRouter();
    const store = useStore();

    watch(count, () => context.$logger.info(route.value.path));
    void fetchState;
    void router;
    void store;

    return { doubled };
  },
});
</script>
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
            "components/AppDialog.vue",
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
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert!(
        !stdout.contains("TS2305") && !stdout.contains("@nuxtjs/composition-api"),
        "composition-api exports should remain available after plugin augmentation:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn write_latest_composition_api_package(project_root: &Path) {
    write(
        project_root,
        "node_modules/@nuxtjs/composition-api/package.json",
        r#"{
  "name": "@nuxtjs/composition-api",
  "version": "0.34.0",
  "main": "./dist/runtime/index.js",
  "module": "./dist/runtime/index.mjs",
  "types": "./dist/runtime/index.d.ts",
  "exports": {
    ".": "./dist/runtime/index.mjs",
    "./module": "./dist/module/index.mjs",
    "./package.json": "./package.json",
    "./dist/babel-plugin": "./dist/babel-plugin/index.js",
    "./dist/runtime/globals": "./dist/runtime/globals.js",
    "./dist/runtime/templates/*": "./dist/runtime/templates/*"
  }
}"#,
    );
    write_composition_api_runtime(project_root, "index.d.ts");
    write_composition_api_runtime(project_root, "index.d.mts");
    write(
        project_root,
        "node_modules/@nuxtjs/composition-api/dist/runtime/index.mjs",
        "export {};\n",
    );
}

fn write_composition_api_runtime(project_root: &Path, name: &str) {
    write(
        project_root,
        &format!("node_modules/@nuxtjs/composition-api/dist/runtime/{name}"),
        r#"export declare function defineComponent<T>(options: T): T;
export declare function ref<T>(value: T): { value: T };
export declare function computed<T>(getter: () => T): { value: T };
export declare function watch<T>(source: { value: T }, callback: () => void): void;
export type PropType<T> = { new (...args: never[]): T } | { (): T };
export interface UseContextReturn {
  route: { value: { path: string } };
}
export declare function useContext(): UseContextReturn;
export declare function useFetch<T>(callback: () => T): { fetch: () => Promise<void> };
export declare function useStore(): { state: unknown };
export declare function useRoute(): { value: { path: string } };
export declare function useRouter(): { push(path: string): Promise<void> };
"#,
    );
}
