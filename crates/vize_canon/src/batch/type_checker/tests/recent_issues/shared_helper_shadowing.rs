//! Batch-shared helper identities must not be capturable by authored types.
//!
//! A normal script is emitted at module scope, between the project helper file
//! and the generated setup/template code. Deliberately poison every public
//! helper name there: an unqualified generated reference becomes a wrong-arity
//! or non-callable error, while the reserved ambient module remains exact.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use crate::batch::VirtualProject;
use oxc_syntax::identifier::is_identifier_part;

const CHILD: &str = r#"<script setup lang="ts">
const props = defineProps<{
  value: string
  ariaLabel?: string
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
type $ComponentNavigationProps = never
type $EmitListeners = never
type $IsGeneratedComponent = never
type $HasFallthroughProps = never
type $FallthroughProps = never
type $RequiredFallthroughProps = never
type $ComponentRawProps = never
type $FallthroughValue = never
type $ComponentEvents = never
type $EmitOwnedKeys = never
type $IsLibraryGenericExport = never
type $PropChecker = never
type $PropValue = never
type $FallthroughSpreadChecker = never
type $CallableProp = never
type $PropsResolver = never
type $ResolvedContextProps = never
type $PropsSelector = never
type $ResolvedProp = never
type $ComponentRefCallback = never
type $StructuralSlots = never
type $NavigationSlots = never
type $RequiredSlots = never
type $SlotChildren = never
type $ProvidedSlotChildren = never
type $SlotHostSlots = never
type $SlotPayload = never
type $SlotHostSelector = never
type $SlotConstructor = never
type $SlotConstructorArgs = never
type $SlotConstructoré = never
type $SlotConstructor\u0041 = never
type $SlotsFactory = never
type $EmitPropsFactory = never
type $WidenTemplateRef = never
type $NotUndefined = never
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
const ariaLabel = undefined as string | undefined
</script>

<template>
  <Child
    v-bind="spread"
    :aria-label="ariaLabel"
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

fn exact_identifier_count(source: &str, identifier: &str) -> usize {
    source
        .split(|character: char| character != '\\' && !is_identifier_part(character))
        .filter(|candidate| *candidate == identifier)
        .count()
}

/// Keep prefix-related helper names independent (`Constructor` versus
/// `ConstructorArgs`) so the custody assertion measures lexical capture.
fn exact_qualified_count(source: &str, qualified: &str) -> usize {
    source
        .match_indices(qualified)
        .filter(|&(start, matched)| {
            source
                .get(start + matched.len()..)
                .and_then(|tail| tail.chars().next())
                .is_none_or(|next| next != '\\' && !is_identifier_part(next))
        })
        .count()
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
    let members = [
        "$ComponentRawProps",
        "$PropChecker",
        "$PropValue",
        "$PropsResolver",
        "$ResolvedContextProps",
        "$PropsSelector",
        "$ResolvedProp",
        "$ComponentEvents",
        "$EmitPropsFactory",
        "$StructuralSlots",
        "$RequiredSlots",
        "$SlotHostSelector",
        "$SlotConstructor",
        "$SlotConstructorArgs",
        "$SlotsFactory",
        "$SlotPayload",
        "$WidenTemplateRef",
        "__EmitProps",
    ];
    let mut activated = Vec::with_capacity(members.len());
    for member in members {
        let direct_reference = format!("import(\"vize:virtual-ts\").{member}");
        let direct_count = exact_qualified_count(generated, direct_reference.as_str());
        let total_count = exact_identifier_count(generated, member);
        assert_eq!(
            total_count,
            direct_count + 1,
            "only the authored poison may reference {member} without the reserved module:\n{generated}"
        );
        if direct_count != 0 {
            activated.push(member);
        }
    }
    assert_eq!(activated, members);
    assert_eq!(
        exact_identifier_count(generated, "$NotUndefined"),
        1,
        "only the authored poison may name $NotUndefined in hoisted mode:\n{generated}"
    );
    assert_eq!(
        generated
            .matches("function __vizeComponentAriaPropEntries")
            .count(),
        1,
        "the ARIA key/value witness must precede the hoisted-helper early return:\n{generated}"
    );
    assert_eq!(
        generated.matches("T extends undefined ? never : T").count(),
        2,
        "the ARIA witness must inline its collision-proof narrowing contract:\n{generated}"
    );
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
        snapshot,
        vec![(
            vize_carton::String::from("src/Parent.vue"),
            Some(2322),
            vize_carton::cstr!(
                "{expected_line}:11:error Type 'number' is not assignable to type 'string'."
            ),
        )]
    );
}
