use super::{Command, create_cli_project, resolve_test_corsa_path};

const SUSPENSE: &str = r#"<script setup lang="ts" generic="T">
import { ref } from 'vue'
const props = defineProps<{ p: () => Promise<T> }>()
const emit = defineEmits<{ (ev: 'resolved', result: T): void }>()
const result = ref<T | null>(null)
props.p().then((value) => emit('resolved', value))
</script>

<template><div v-if="result != null"><slot :result="result as T" /></div></template>
"#;

// Misskey registers globals from a module (`components/index.ts`); a hand-written
// declaration file must keep `typeof MkSuspense` naming the generic component
// once the annotation is copied out of the file.
const REGISTRATION: &str = r#"import MkSuspense from './Suspense.vue'
declare module 'vue' {
  export interface GlobalComponents {
    MkSuspense: typeof MkSuspense
  }
}
export {}
"#;

fn diagnostics(registration: &str, handler_read: &str) -> Vec<serde_json::Value> {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return Vec::new();
    };
    let app = vize_l0::cstr!(
        r#"<script setup lang="ts">
import {{ ref }} from 'vue'
function load() {{
  return Promise.resolve({{ file: 'a', info: 1 }})
}}
const file = ref<string | null>(null)
</script>

<template>
  <MkSuspense :p="load" @resolved="(result) => file = result.{handler_read}" />
</template>
"#
    );
    let project_root = create_cli_project(
        &vize_l0::cstr!("global-component-value-import-{handler_read}"),
        &[
            ("src/Suspense.vue", SUSPENSE),
            (registration, REGISTRATION),
            ("src/App.vue", app.as_str()),
        ],
    );
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&project_root);
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().clone())
        .collect()
}

#[test]
fn global_generic_component_infers_its_event_payload_like_vue_tsc() {
    for registration in ["src/global-components.ts", "src/components.d.ts"] {
        assert_eq!(
            diagnostics(registration, "file"),
            Vec::<serde_json::Value>::new(),
            "{registration}"
        );
        let broken = diagnostics(registration, "nope");
        if broken.is_empty() {
            return;
        }
        assert_eq!(
            broken,
            [serde_json::json!(
                "error:10:62 [TS2339] Property 'nope' does not exist on type '{ file: string; info: number; }'."
            )],
            "{registration}"
        );
    }
}
