#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/nuxt_cli.rs"]
mod nuxt_cli;
#[path = "support/nuxt_stress.rs"]
mod nuxt_stress;
#[path = "check_nuxt_tsconfig_paths_cli/support.rs"]
mod support;

use std::process::Command;
use support::{
    create_project, required_iterations, resolve_test_corsa_path, run_nuxt2_alias_check, write_file,
};

#[test]
fn check_nuxt_sfc_virtual_ts_prefers_explicit_tsconfig_paths_over_fallback_modules() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
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
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
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

    for iteration in 0..required_iterations() {
        let output = run_nuxt2_alias_check(&project_root, &corsa_path);
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            output.status.success(),
            "Nuxt2 alias iteration {iteration} failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["errorCount"], 0, "{stdout}");
    }

    let source = project_root.join("src/app/purposes/Keyboards.vue");
    let broken = std::fs::read_to_string(&source)
        .unwrap()
        .replace("EnglishKeyboard.vue", "MissingKeyboard.vue");
    std::fs::write(source, broken).unwrap();
    let output = run_nuxt2_alias_check(&project_root, &corsa_path);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!output.status.success(), "{stdout}");
    assert!(
        stdout.contains(
            "error:4:8 [TS2307] Cannot find module \
             '~/src/shared/components/keyboards/MissingKeyboard.vue'"
        ),
        "{stdout}"
    );
    let _ = std::fs::remove_dir_all(&project_root);
}
