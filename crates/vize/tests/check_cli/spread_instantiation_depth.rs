use super::{Command, create_cli_project, resolve_test_corsa_path};

// vue-vben-admin `examples/count-to`: spreading a deeply unwrapped `reactive()`
// object is a TS2589 in plain TypeScript too, so it is reported on the tag.
const TYPES: &str = r#"type Css = { [key: string]: string | number | undefined }
export type StyleValue = false | null | undefined | string | Css | Array<StyleValue>
export type TransitionPresets = 'easeInSine' | 'easeOutQuart' | 'easeOutExpo'
export interface CountToProps {
  startVal?: number; endVal: number
  transition?: [number, number, number, number] | ((n: number) => number) | TransitionPresets
  mainStyle?: StyleValue; decimalStyle?: StyleValue; prefixStyle?: StyleValue; suffixStyle?: StyleValue
}
// The shape of Vue's `UnwrapNestedRefs` that makes the spread deep.
type Unwrap<T> = T extends Function ? T : T extends object ? { [K in keyof T]: Unwrap<T[K]> } : T
export declare function reactive<T extends object>(value: T): Unwrap<T>
"#;

const CHILD: &str = r#"<script setup lang="ts">
import type { CountToProps } from './types'
const props = defineProps<CountToProps>()
defineEmits(['started'])
</script>

<template><div :style="props.mainStyle">{{ props.endVal }}</div></template>
"#;

const APP: &str = r#"<script setup lang="ts">
import CountTo from './CountTo.vue'
import { reactive, type CountToProps, type TransitionPresets } from './types'
const props = reactive<CountToProps & { transition: TransitionPresets }>({ endVal: 1, transition: 'easeOutQuart' })
function onStarted() {}
</script>

<template>
  <div><CountTo v-bind="props" @started="onStarted" /></div>
</template>
"#;

#[test]
fn deep_spread_instantiation_is_reported_on_the_component_tag() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "spread-instantiation-depth",
        &[
            ("src/types.ts", TYPES),
            ("src/CountTo.vue", CHILD),
            ("src/App.vue", APP),
        ],
    );
    // The depth comes from Vue's own component and ref types, so use the
    // frozen Playground Vue rather than the stub.
    let vue = super::workspace_root().join("playground/node_modules/vue");
    super::symlink_path(
        &vue.canonicalize()
            .expect("the depth check requires frozen Playground Vue"),
        &project_root.join("node_modules/vue"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&project_root);
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let diagnostics: Vec<_> = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().clone())
        .collect();
    assert_eq!(
        diagnostics,
        [serde_json::json!(
            "error:9:9 [TS2589] Type instantiation is excessively deep and possibly infinite. (source: <div><CountTo v-bind=\"props\" @started=\"onStarted\" /></div>)"
        )]
    );
}
