use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn check_nuxt_sfc_virtual_ts_prefers_explicit_tsconfig_paths_over_fallback_modules() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_project("nuxt-explicit-alias-shims");

    write_file(&project_root, "nuxt.config.ts", "export default {}\n");
    write_file(
        &project_root,
        "tsconfig.vize.json",
        r##"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "noEmit": true,
    "paths": {
      "#imports": ["types/vize/imports.ts"],
      "#components": ["types/vize/components.ts"],
      "@typed-router": ["types/vize/typed-router.ts"]
    }
  },
  "include": ["app/**/*.vue", "design/**/*.vue", "types/vize/**/*.ts"]
}"##,
    );
    write_file(
        &project_root,
        "app/pages/index.vue",
        r##"<script setup lang="ts">
import { ref } from "#imports";
import { NuxtPage } from "#components";
import { useRouter } from "@typed-router";

const count = ref(0);
const router = useRouter();
void [count, router, NuxtPage];
</script>
"##,
    );
    write_file(
        &project_root,
        "design/components/AliasConsumer.vue",
        r##"<script setup lang="ts">
import { NuxtLink } from "#components";
import {
  useAttrs,
  useId,
  readonly,
  provide,
  type InjectionKey,
  type Ref,
} from "#imports";
import { useRoute, type TypedRouteLocationRawFromName } from "@typed-router";

const key = Symbol("value") as InjectionKey<Ref<string>>;
const value: Ref<string> = { value: useId() };
provide(key, readonly(value));
const attrs = useAttrs();
const route = useRoute();
const target: TypedRouteLocationRawFromName<"home"> = { name: "home" };

void [attrs, route, target, NuxtLink];
</script>

<template>
  <NuxtLink to="/">Home</NuxtLink>
</template>
"##,
    );
    write_file(
        &project_root,
        "types/vize/imports.ts",
        r#"export interface Ref<T = unknown> {
  value: T;
}

export interface InjectionKey<T> extends Symbol {}

export function ref<T>(value: T): Ref<T> {
  return { value };
}

export function useAttrs(): { id?: string } {
  return {};
}

export function useId(): string {
  return "id";
}

export function readonly<T>(value: T): Readonly<T> {
  return value;
}

export function provide<T>(_key: InjectionKey<T>, _value: T): void {}
"#,
    );
    write_file(
        &project_root,
        "types/vize/components.ts",
        r#"export const NuxtPage = {};
export const NuxtLink = {};
"#,
    );
    write_file(
        &project_root,
        "types/vize/typed-router.ts",
        r#"export type TypedRouteLocationRawFromName<Name extends string = string> = {
  name: Name;
};

export function useRoute(): { name: string } {
  return { name: "home" };
}

export function useRouter(): {
  push(to: TypedRouteLocationRawFromName): void;
} {
  return { push() {} };
}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.vize.json",
            "--no-check-props",
            "--no-check-emits",
            "--no-check-template-bindings",
            "--format",
            "json",
            "app",
            "design",
            "types",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check should use explicit tsconfig paths for Nuxt aliases\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_nuxt2_options_api_component_event_payloads_resolve_through_aliases() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_project("nuxt2-options-api-emits-alias");

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
    "noEmit": true,
    "noUnusedLocals": true,
    "paths": {
      "~/*": ["*"],
      "@/*": ["*"]
    }
  },
  "include": ["src/**/*.vue"]
}"##,
    );
    write_file(
        &project_root,
        "src/app/purposes/Keyboards.vue",
        r##"<script setup lang="ts">
import EnglishKeyboard, {
  type ChoiceOption,
} from "~/src/shared/components/keyboards/EnglishKeyboard.vue";

const options: ChoiceOption[] = [{ value: "a" }];

function selectOption(incomingValue: ChoiceOption) {
  incomingValue.value.toUpperCase();
}
</script>

<template>
  <EnglishKeyboard :options="options" @input="selectOption" />
</template>
"##,
    );
    write_file(
        &project_root,
        "src/shared/components/keyboards/EnglishKeyboard.vue",
        r##"<script lang="ts">
import { defineComponent, type PropType } from "vue";

export type ChoiceOption = { value: string };

export default defineComponent({
  props: {
    options: {
      type: Array as PropType<ChoiceOption[]>,
      required: true,
    },
  },
  emits: {
    input(value: ChoiceOption) {
      return value.value.length > 0;
    },
  },
});
</script>

<template>
  <button type="button">{{ options.length }}</button>
</template>
"##,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/app/purposes/Keyboards.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "--no-config",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "Nuxt2 alias component emits should type-check\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}");

    let _ = std::fs::remove_dir_all(&project_root);
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
