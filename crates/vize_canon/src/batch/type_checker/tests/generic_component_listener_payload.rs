use super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

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

    assert!(
        snapshot
            .iter()
            .all(|(file, _, message)| !(file == "src/App.vue" && message.contains("login"))),
        "compatible generic component listener should not report diagnostics, got: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/App.vue" && *code == Some(2345) && message.contains("username: number")
        }),
        "incompatible generic component listener should still report TS2345, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
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
