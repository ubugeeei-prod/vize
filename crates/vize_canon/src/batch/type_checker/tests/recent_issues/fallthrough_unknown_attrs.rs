//! Native-root fallthrough accepts userland attributes that are not declared as
//! component props or listed in Vue's native element table (#4461).
//!
//! Oracle: `voicevox` passes `:disable` to a single-`button` wrapper component.
//! `vue-tsc` treats it as a fallthrough attr, while Vize used to reject it as an
//! excess component prop.

use super::super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics, write_test_vue_stub,
};
use crate::batch::runtime_deps::VUE_RUNTIME_DOM_STUB_TYPES;
use std::path::PathBuf;
use vize_s0::{String, cstr};

const NATIVE_BUTTON_TYPES: &str = r#"export interface NativeElements {
  button: { disabled?: boolean | undefined; type?: "button" | "submit" | undefined };
}
"#;

const NATIVE_INPUT_TYPES: &str = r#"export interface NativeElements {
  input: { id?: string; 'aria-label'?: string; 'aria-activedescendant'?: string };
}
"#;

fn create_fallthrough_project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    create_fallthrough_project_with_native_elements(name, NATIVE_BUTTON_TYPES, files)
}

fn create_fallthrough_project_with_native_elements(
    name: &str,
    native_elements: &str,
    files: &[(&str, &str)],
) -> PathBuf {
    let project_root = create_project_case_without_node_modules(name, files);
    let node_modules = project_root.join("node_modules");
    write_test_vue_stub(&node_modules).expect("write isolated Vue stub");
    let vue_types = VUE_RUNTIME_DOM_STUB_TYPES.replace(
        "export type NativeElements = Record<string, Record<string, unknown>>;",
        native_elements,
    );
    std::fs::write(node_modules.join("@vue/runtime-dom/index.d.ts"), vue_types)
        .expect("pin native fallthrough props");
    project_root
}

#[test]
fn native_root_fallthrough_accepts_unknown_attrs_without_hiding_required_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_fallthrough_project(
        "fallthrough-unknown-attrs",
        &[
            (
                "src/NativeButton.vue",
                r#"<script setup lang="ts">
defineProps<{ label: string; disabled?: boolean }>()
</script>

<template>
  <button>{{ label }}</button>
</template>
"#,
            ),
            (
                "src/MultiRootButton.vue",
                r#"<script setup lang="ts">
defineProps<{ label: string }>()
</script>

<template>
  <button>{{ label }}</button>
  <span />
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import NativeButton from './NativeButton.vue'
import MultiRootButton from './MultiRootButton.vue'

const uiLocked = true
</script>

<template>
  <NativeButton label="Run" :disable="uiLocked" :disabled="true" />
  <NativeButton label="Run" :disabled="'bad'" />
  <NativeButton :disable="uiLocked" />
  <MultiRootButton label="Run" :disable="uiLocked" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    // One diagnostic per binding: the fresh literal's per-prop rendering
    // (`Type '"bad"'`) collapses into the widened elaboration `vue-tsc`
    // reports (#4966). The check-tail text carries the pinned native attr
    // surface plus the custom `data-*` map that same fix admits.
    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/Parent.vue"),
                Some(2322),
                cstr!("10:30:error Type 'string' is not assignable to type 'boolean | undefined'."),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2345),
                String::from(
                    "11:4:error Argument of type '{ disable: boolean; }' is not assignable to parameter of type '__VizeComponentCheckProps<Props, __VizePublicComponentAttrs & { disabled?: unknown; type?: unknown; } & { [x: `data${string}`]: unknown; } & Record<string, unknown>>'.\nProperty 'label' is missing in type '{ disable: boolean; }' but required in type '{ readonly label: string; readonly disabled?: boolean | undefined; }'."
                ),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2353),
                cstr!(
                    "12:33:error Object literal may only specify known properties, and '\"disable\"' does not exist in type '__VizeComponentCheckProps<Props, __VizePublicComponentAttrs & {{ disabled?: unknown; type?: unknown; }} & {{ [x: `data${{string}}`]: unknown; }}>'."
                ),
            ),
        ],
        "unknown attrs fall through only when the child has a fallthrough target"
    );
}

#[test]
fn generic_native_root_fallthrough_accepts_unknown_attrs() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_fallthrough_project(
        "generic-fallthrough-unknown-attrs",
        &[
            (
                "src/GenericButton.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{ value: T; label: string }>()
</script>

<template>
  <button>{{ label }} {{ value }}</button>
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import GenericButton from './GenericButton.vue'

const value = { id: 1 }
const uiLocked = true
</script>

<template>
  <GenericButton :value="value" label="Run" :disable="uiLocked" />
  <GenericButton :value="value" label="Run" :disabled="'bad'" />
  <GenericButton :value="value" :label="123" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert!(
        snapshot
            .iter()
            .all(|(_, _, message)| !message.contains("'disable'")),
        "generic native-root fallthrough must keep unknown attrs open: {snapshot:#?}"
    );
    assert_eq!(
        snapshot,
        vec![(
            String::from("src/Parent.vue"),
            Some(2322),
            cstr!("11:34:error Type 'number' is not assignable to type 'string'."),
        )],
        "generic native-root fallthrough must stay value-open without hiding declared prop checks"
    );
}

#[test]
fn generic_closed_component_accepts_global_html_attrs() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_fallthrough_project_with_native_elements(
        "generic-closed-global-html-attrs",
        NATIVE_INPUT_TYPES,
        &[
            (
                "src/GenericField.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{ value: T; label: string }>()
</script>

<template>
  <label>{{ label }}</label>
  <input :value="String(value)" />
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import GenericField from './GenericField.vue'

const value = { id: 1 }
</script>

<template>
  <GenericField :value="value" label="Search" id="search" />
  <GenericField :value="value" label="Search" aria-label="Search" />
  <GenericField :value="value" label="Search" aria-activedescendant="row-1" />
  <GenericField :value="value" label="Search" data-test-id="search" />
  <GenericField :value="value" label="Search" dataTestId="search" />
  <GenericField :value="value" label="Search" database="main" />
  <GenericField :value="value" label="Search" depressed />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    for attr in ["id", "ariaLabel", "ariaActivedescendant", "dataTestId"] {
        let complaint = cstr!("'\"{attr}\"' does not exist");
        assert!(
            snapshot
                .iter()
                .all(|(_, _, message)| !message.contains(complaint.as_str())),
            "generic closed components must accept global HTML attr {attr}: {snapshot:#?}"
        );
    }
    assert_eq!(snapshot.len(), 2, "{snapshot:#?}");
    for (attr, (file, code, message)) in ["database", "depressed"].into_iter().zip(snapshot.iter())
    {
        assert_eq!(file.as_str(), "src/Parent.vue");
        assert_eq!(*code, Some(2353));
        assert!(
            message.contains(cstr!("'\"{attr}\"' does not exist in type").as_str()),
            "unknown attrs must stay strict on closed generic components: {message}"
        );
    }
}
