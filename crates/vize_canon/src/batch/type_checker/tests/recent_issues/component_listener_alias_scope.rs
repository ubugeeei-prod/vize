//! #10112: stable component-listener aliases cannot cross sibling generated
//! `if` blocks, even when both blocks carry the same authored guard.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn component_listener_aliases_stay_inside_guard_scope() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = super::real_vue::create_project_case(
        "component-listener-alias-scope",
        &[
            (
                "src/MyButton.vue",
                r#"<script setup lang="ts">
defineProps<{ color?: string }>();
defineEmits<{ click: [event: MouseEvent] }>();
</script>

<template>
  <button type="button" @click="$emit('click', $event)"><slot /></button>
</template>
"#,
            ),
            (
                "src/MyMenu.vue",
                r#"<script setup lang="ts">
defineSlots<{
  activator(props: { on: Record<string, (event: Event) => void>; attrs: Record<string, unknown> }): unknown;
  default(): unknown;
}>();
</script>

<template>
  <div>
    <slot name="activator" :on="{}" :attrs="{}" />
    <slot />
  </div>
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import { ref } from "vue";
import MyButton from "./MyButton.vue";
import MyMenu from "./MyMenu.vue";

const shown = ref(true);
function open(): void {}
function save(): void {}
</script>

<template>
  <div v-if="shown">
    <MyButton @click="open">open</MyButton>
    <MyMenu>
      <template #activator><span>more</span></template>
      <MyButton @click="save">save</MyButton>
    </MyMenu>
  </div>
</template>
"#,
            ),
        ],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noEmit": true,
    "jsx": "preserve",
    "skipLibCheck": true,
    "verbatimModuleSyntax": true,
    "types": []
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}"#,
    )
    .unwrap();

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        snapshot,
        Some(Vec::new()),
        "sibling listener guards must not reuse block-local aliases: {snapshot:#?}"
    );
}
