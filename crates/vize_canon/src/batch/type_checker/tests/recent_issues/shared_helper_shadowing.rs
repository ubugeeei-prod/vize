//! Batch-shared helper identities must not be capturable by authored types.
//!
//! A normal script is emitted at module scope, between the project helper file
//! and the generated setup/template code. Deliberately poison every public
//! helper name there: an unqualified generated reference becomes a wrong-arity
//! or non-callable error, while the reserved ambient module remains exact.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use crate::batch::VirtualProject;

const CHILD: &str = r#"<script setup lang="ts">
const props = defineProps<{
  value: string
  transform: (value: string) => string
}>()
const emit = defineEmits<{ save: [value: string] }>()
defineSlots<{
  default(props: { item: string }): any
  named(props: { label: string }): any
}>()
defineExpose({ focus: () => undefined })
</script>

<template>
  <button @click="emit('save', props.value)">
    <slot :item="props.value" />
    <slot name="named" label="label" />
  </button>
</template>
"#;

const POISONED_MODULE_TYPES: &str = r#"<script lang="ts">
type __VizeComponentNavigationProps = never
type __VizeEmitListeners = never
type __VizeIsGeneratedComponent = never
type __VizeHasFallthroughProps = never
type __VizeFallthroughProps = never
type __VizeRequiredFallthroughProps = never
type __VizeComponentRawProps = never
type __VizeFallthroughValue = never
type __VizeComponentEvents = never
type __VizeEmitOwnedKeys = never
type __VizeIsLibraryGenericExport = never
type __VizePropChecker = never
type __VizePropValue = never
type __VizeFallthroughSpreadChecker = never
type __VizeCallableProp = never
type __VizePropsResolver = never
type __VizeResolvedContextProps = never
type __VizePropsSelector = never
type __VizeResolvedProp = never
type __VizeComponentRefCallback = never
type __VizeStructuralSlots = never
type __VizeNavigationSlots = never
type __VizeRequiredSlots = never
type __VizeSlotChildren = never
type __VizeProvidedSlotChildren = never
type __VizeSlotHostSlots = never
type __VizeSlotPayload = never
type __VizeSlotHostSelector = never
type __VizeSlotConstructor = never
type __VizeSlotConstructorArgs = never
type __VizeSlotsFactory = never
type __VizeEmitPropsFactory = never
type __VizeWidenTemplateRef = never
type __EmitProps = never
export const moduleTypePoison = true
</script>
"#;

fn parent_source(value: &str) -> std::string::String {
    format!(
        r#"{POISONED_MODULE_TYPES}
<script setup lang="ts">
import {{ ref }} from 'vue'
import Child from './Child.vue'

defineEmits<{{ local: [value: string] }}>()
const spread = {{ value: {value} }}
const literal = ref<'ok'>('ok')
</script>

<template>
  <Child
    v-bind="spread"
    :transform="item => item.toUpperCase()"
    :ref="instance => instance?.focus()"
    @save="item => item.toUpperCase()"
  >
    <template #default="{{ item }}">{{{{ item.toUpperCase() }}}} {{{{ literal }}}}</template>
    <template #named="{{ label }}">{{{{ label.toUpperCase() }}}}</template>
  </Child>
</template>
"#
    )
}

fn negative_parent_source() -> std::string::String {
    format!(
        r#"{POISONED_MODULE_TYPES}
<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child :value="123" :transform="value => value">
    <template #default="{{ item }}">{{{{ item.toUpperCase() }}}}</template>
    <template #named="{{ label }}">{{{{ label.toUpperCase() }}}}</template>
  </Child>
</template>
"#
    )
}

#[test]
fn authored_module_types_cannot_capture_batch_helpers() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let parent = parent_source("'ok'");
    let project_root = create_project_case(
        "shared-helper-shadowing",
        &[("src/Child.vue", CHILD), ("src/Parent.vue", &parent)],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let Some(snapshot) = snapshot else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    assert!(
        snapshot.is_empty(),
        "authored helper-shaped aliases must not affect batch checking: {snapshot:#?}"
    );

    let parent_path = project_root.join("src/Parent.vue");
    let child_path = project_root.join("src/Child.vue");
    let mut project = VirtualProject::new(&project_root).expect("virtual project should open");
    project
        .register_paths(&[child_path, parent_path.clone()])
        .expect("collision fixture should register");
    let generated = &project
        .find_by_original(&parent_path)
        .expect("parent virtual file should exist")
        .content;
    for member in [
        "__VizeComponentRawProps",
        "__VizePropChecker",
        "__VizePropValue",
        "__VizePropsResolver",
        "__VizeResolvedContextProps",
        "__VizePropsSelector",
        "__VizeResolvedProp",
        "__VizeComponentEvents",
        "__VizeEmitPropsFactory",
        "__VizeStructuralSlots",
        "__VizeRequiredSlots",
        "__VizeSlotHostSelector",
        "__VizeSlotConstructor",
        "__VizeSlotConstructorArgs",
        "__VizeSlotsFactory",
        "__VizeSlotPayload",
        "__VizeWidenTemplateRef",
        "__EmitProps",
    ] {
        assert!(
            generated.contains(&format!("import(\"vize:virtual-ts\").{member}")),
            "activated helper {member} must use the collision-proof module:\n{generated}"
        );
    }
    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn helper_isolation_preserves_the_authored_prop_diagnostic_anchor() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let parent = negative_parent_source();
    let expected_line = parent
        .lines()
        .position(|line| line.contains("<Child :value"))
        .expect("fixture component line")
        + 1;
    let project_root = create_project_case(
        "shared-helper-shadowing-negative",
        &[("src/Child.vue", CHILD), ("src/Parent.vue", &parent)],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };
    assert_eq!(
        snapshot.len(),
        1,
        "one authored mismatch only: {snapshot:#?}"
    );
    let (file, code, message) = &snapshot[0];
    assert_eq!(file.as_str(), "src/Parent.vue");
    assert_eq!(*code, Some(2322));
    assert!(
        message.starts_with(&format!("{expected_line}:11:error")),
        "the value attribute must retain its authored UTF-16 anchor: {message}"
    );
}
