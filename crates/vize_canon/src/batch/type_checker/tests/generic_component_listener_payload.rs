use super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::String;

mod callback_prop_shorthand;
mod dynamic_member_component_events;
mod enum_member_prop;
mod overload_depth;

#[test]
fn batch_type_checker_accepts_component_fallthrough_touch_listener() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "component-fallthrough-touch-listener",
        &[(
            "src/Foo.vue",
            r#"<script setup lang="ts">
import { defineComponent, h } from "vue";

const Child = defineComponent(() => () => h("button"));

function onPress(event: MouseEvent) {
  event.preventDefault();
}
</script>

<template>
  <Child @touchstart="onPress" />
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Foo.vue" && *code == Some(2345) && message.contains("TouchEvent"))
        }),
        "component fallthrough listeners should not be checked as native DOM listeners, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_infers_generic_component_listener_payload_from_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-component-listener-payload",
        &[
            (
                "src/Form.vue",
                r#"<script setup lang="ts" generic="FormShape extends object">
export interface FormSubmitEvent<FormShape extends object> {
  data: FormShape
}

export interface FormEmits<FormShape extends object> {
  (e: "submit", payload: FormSubmitEvent<FormShape>): void
  (e: "reset"): void
}

defineProps<{
  initialState: FormShape
}>()

defineEmits<FormEmits<FormShape>>()
</script>

<template>
  <form />
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Form, { type FormSubmitEvent } from './Form.vue'

function login(payload: FormSubmitEvent<{ username: string; password: string }>) {
  payload.data.username.toUpperCase()
  payload.data.password.toUpperCase()
}

function wrong(payload: FormSubmitEvent<{ username: number; password: string }>) {
  payload.data.username.toFixed()
}
</script>

<template>
  <Form
    :initial-state="{ username: '', password: '' }"
    @submit="login"
    @reset="() => undefined"
  />
  <Form
    :initial-state="{ username: '', password: '' }"
    @submit="wrong"
  />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    let _ = std::fs::remove_dir_all(&project_root);

    // The compatible listener reports nothing; the incompatible one is a
    // `TS2322` at the `submit` of `@submit`, matching how vue-tsc assigns the
    // handler to the child's `onSubmit` prop (#3462). The payload type still
    // comes from the generic instantiated by `:initial-state`, which is what
    // this case exists to pin. Corsa names the target rest parameter `args`
    // inside the elaboration; the diagnostic code, anchor and instantiated
    // payload are the parity surface here.
    assert_eq!(
        snapshot,
        vec![(
            String::from("src/App.vue"),
            Some(2322),
            String::from(
                "22:6:error Type '(payload: FormSubmitEvent<{ username: number; password: string; }>) => void' \
                 is not assignable to type '(payload: FormSubmitEvent<{ username: string; password: string; }>) => any'.\n\
                 Types of parameters 'payload' and 'payload' are incompatible.\n\
                 Type 'FormSubmitEvent<{ username: string; password: string; }>' is not assignable to type 'FormSubmitEvent<{ username: number; password: string; }>'.\n\
                 Type '{ username: string; password: string; }' is not assignable to type '{ username: number; password: string; }'.\n\
                 Types of property 'username' are incompatible.\n\
                 Type 'string' is not assignable to type 'number'."
            ),
        )]
    );
}

#[test]
fn batch_type_checker_accepts_hyphenated_component_emit_listeners() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "hyphenated-component-emit-listeners",
        &[
            (
                "src/OverlayDialog.vue",
                r#"<script setup lang="ts">
export interface FolderPayload {
  id: string
}

defineEmits<{
  (e: "click-folder", payload: FolderPayload): void
  (e: "update:is-opened-overlay-loading", value: boolean): void
  (e: "input:math-key", key: string): void
}>()
</script>

<template>
  <button type="button">Open</button>
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import OverlayDialog, { type FolderPayload } from './OverlayDialog.vue'

function handleFolder(payload: FolderPayload) {
  payload.id.toUpperCase()
}

function handleOverlay(open: boolean) {
  open.valueOf()
}

function handleMathKey(key: string) {
  key.toUpperCase()
}
</script>

<template>
  <OverlayDialog
    @click-folder="handleFolder"
    @update:is-opened-overlay-loading="handleOverlay"
    @input:math-key="handleMathKey"
  />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "hyphenated component emit listeners should not report diagnostics, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_infers_dynamic_component_kebab_listener_payload() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "dynamic-component-kebab-listener-payload",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineEmits<{ escapeKeyDown: [event: KeyboardEvent] }>();
</script>

<template>
  <button type="button" @keydown.escape="$emit('escapeKeyDown', $event)">
    Close
  </button>
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from "./Child.vue";

function handleEscape(event: KeyboardEvent) {
  event.preventDefault();
}
</script>

<template>
  <component :is="Child" @escape-key-down="handleEscape" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "dynamic component kebab-case listener should infer child emit payload, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_infers_event_name_from_define_emits_helper_argument() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "define-emits-helper-event-name-inference",
        &[(
            "src/Foo.vue",
            r#"<script setup lang="ts">
import type { MaybeRefOrGetter } from "vue";

const props = defineProps<{ open?: boolean }>();
const emit = defineEmits<{ "update:open": [value: boolean] }>();

function createEventBridge<
  State extends Record<string, unknown>,
  EventName extends string
>(
  state: MaybeRefOrGetter<State>,
  dispatch: (event: EventName, ...args: any[]) => void
) {
  return { state, dispatch };
}

createEventBridge(props, emit);
</script>

<template>
  <button @click="emit('update:open', !props.open)" />
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Foo.vue"
                && *code == Some(2345)
                && message.contains("Type 'string' is not assignable to type"))
        }),
        "typed defineEmits should infer the helper's EventName, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
