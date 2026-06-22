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
        .join(cstr!("check-nuxt-composition-api-{name}-{}", std::process::id()).as_str())
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
fn check_resolves_nuxt2_composition_api_named_exports() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("exports");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
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
  "include": ["src/**/*"]
}"#,
    );
    write(
        &project_root,
        "node_modules/@nuxtjs/composition-api/package.json",
        r#"{
  "name": "@nuxtjs/composition-api",
  "version": "0.26.0",
  "main": "./dist/runtime/index.js",
  "module": "./dist/runtime/index.mjs",
  "types": "./dist/runtime/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/runtime/index.mjs",
      "require": "./dist/runtime/index.js"
    },
    "./package.json": "./package.json"
  }
}"#,
    );
    write(
        &project_root,
        "node_modules/@nuxtjs/composition-api/dist/runtime/index.d.ts",
        r#"export declare function defineComponent<T>(options: T): T;
export declare function ref<T>(value: T): { value: T };
export declare function computed<T>(getter: () => T): { value: T };
export type PropType<T> = { new (...args: never[]): T } | { (): T };
export declare function useContext(): { app: unknown };
export declare function useFetch<T>(callback: () => T): { fetch: () => Promise<T> };
export declare function useStore(): { state: unknown };
export declare function useRoute(): { value: { path: string } };
"#,
    );
    write(
        &project_root,
        "src/App.ts",
        r#"import {
  computed,
  defineComponent,
  PropType,
  ref,
  useContext,
  useFetch,
  useRoute,
  useStore,
} from "@nuxtjs/composition-api";

const component = defineComponent({
  props: { label: String as PropType<string> },
});
const count = ref(1);
const doubled = computed(() => count.value * 2);
const context = useContext();
const fetchState = useFetch(() => doubled.value);
const store = useStore();
const route = useRoute();

void component;
void context;
void fetchState;
void store;
void route;
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
            "src/App.ts",
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
        !stdout.contains("TS2307") && !stdout.contains("@nuxtjs/composition-api"),
        "composition-api exports should resolve through Nuxt fallback paths:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
