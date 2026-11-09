//! Batch-shared helper identities must not be capturable by authored types.
//!
//! A normal script is emitted at module scope, between the project helper file
//! and the generated setup/template code. Deliberately poison every public
//! helper name there: an unqualified generated reference becomes a wrong-arity
//! or non-callable error, while the reserved ambient module remains exact.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use crate::batch::VirtualProject;
use oxc_syntax::identifier::is_identifier_part;
use std::path::Path;

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
        3,
        "the ARIA key/value witness must precede the hoisted-helper early return:\n{generated}"
    );
    assert_eq!(
        generated.matches("T extends undefined ? never : T").count(),
        4,
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
/// Lib types the projection reaches through `globalThis` once the SFC binds
/// the same name (#12536): a same-arity `never` poison, and how many
/// references the parent fixture's projection generates.
const QUALIFIED_LIB_TYPES: &[(&str, &str, usize)] = &[
    ("Awaited", "type Awaited<T> = never", 2),
    (
        "Parameters",
        "type Parameters<T extends (...args: any) => any> = never",
        4,
    ),
    (
        "ReturnType",
        "type ReturnType<T extends (...args: any) => any> = never",
        3,
    ),
];

const LIB_CHILD: &str = r#"<script setup lang="ts" generic="T extends string | number">
const props = withDefaults(defineProps<{
  items: T[]
  modelValue: T
  label?: string
}>(), { label: 'none' })
const emit = defineEmits<{ pick: [item: T, index: number]; 'update:modelValue': [value: T] }>()
defineSlots<{ row(props: { item: T; index: number }): any }>()
</script>

<template>
  <ul :aria-label="props.label">
    <li v-for="(item, index) in props.items" :key="index" @click="emit('pick', item, index); emit('update:modelValue', item)">
      <slot name="row" :item="item" :index="index" />
    </li>
  </ul>
</template>
"#;

#[derive(Clone, Copy, PartialEq)]
enum LibPoison {
    /// The poison lines under names nothing references.
    None,
    Declared,
    Imported,
}

fn lib_module_script(poison: LibPoison) -> std::string::String {
    let mut script = std::string::String::from("<script lang=\"ts\">\n");
    for (name, declaration, _) in QUALIFIED_LIB_TYPES {
        let line = match poison {
            LibPoison::None => declaration.replacen(name, &format!("Unshadowed{name}"), 1),
            LibPoison::Declared => std::string::String::from(*declaration),
            LibPoison::Imported => format!("import type {{ {name} }} from './lib-types'"),
        };
        script.push_str(&line);
        script.push('\n');
    }
    script.push_str("export const libTypePoison = true\n</script>\n");
    script
}

fn lib_types_module() -> std::string::String {
    QUALIFIED_LIB_TYPES
        .iter()
        .map(|(_, declaration, _)| format!("export {declaration}\n"))
        .collect()
}

fn lib_parent_source(poison: LibPoison) -> std::string::String {
    format!(
        r#"{}
<script setup lang="ts">
import {{ ref, useTemplateRef }} from 'vue'
import Child from './Child.vue'
import Picker from './Picker.vue'

const chosen = ref<'a' | 'b'>('a')
const options = ['a', 'b'] as const
const count = ref(0)
const slotName = ref('extra')
const button = useTemplateRef('button')
function onPick(item: 'a' | 'b', index: number) {{ count.value = index; void item }}
export type PublicCount = typeof count
</script>

<template>
  <Child value="ok" :transform="item => item.toUpperCase()" @save="item => item.toUpperCase()">
    <template #default="{{ item }}">{{{{ item.toUpperCase() }}}}<slot name="inner" :item="item" /></template>
    <template #named="{{ label }}">{{{{ label.toUpperCase() }}}}</template>
  </Child>
  <Picker v-model="chosen" :items="[...options]" label="pick" @pick="onPick">
    <template #row="{{ item, index }}">{{{{ item.toUpperCase() }}}} {{{{ index.toFixed() }}}}</template>
  </Picker>
  <slot name="extra" v-bind="{{ count }}" />
  <slot :name="slotName" v-bind="{{ count }}" />
  <button ref="button" v-if="count > 0" @click="button?.focus()">{{{{ count }}}}</button>
</template>
"#,
        lib_module_script(poison)
    )
}

/// References to `name` that are neither qualified (`globalThis.name`) nor
/// part of a longer identifier.
fn unqualified_count(source: &str, name: &str) -> usize {
    source
        .match_indices(name)
        .filter(|&(start, matched)| {
            let before = source
                .get(..start)
                .and_then(|head| head.chars().next_back());
            let after = source
                .get(start + matched.len()..)
                .and_then(|tail| tail.chars().next());
            before.is_none_or(|c| c != '.' && c != '$' && !is_identifier_part(c))
                && after.is_none_or(|c| c != '$' && !is_identifier_part(c))
        })
        .count()
}

/// The poison is the only unqualified spelling, and every generated reference
/// is qualified.
fn assert_only_the_poison_is_unqualified(generated: &str) {
    for (name, _, references) in QUALIFIED_LIB_TYPES {
        let qualified = format!("globalThis.{name}<");
        assert_eq!(
            (
                unqualified_count(generated, name),
                generated.matches(qualified.as_str()).count()
            ),
            (1, *references),
            "{name}: (unqualified, qualified) references:\n{generated}"
        );
    }
}

#[test]
fn the_content_mapper_projection_names_shadowed_lib_types_through_global_this() {
    // The installed SDK's shared module holds the helpers; inline helpers are
    // the fallback for an unqualified install.
    let options = crate::batch::ContentMapperTransformOptions::default()
        .with_external_component_attrs_module(true)
        .with_external_component_type_helpers(true)
        .with_shared_virtual_module(true);
    for poison in [LibPoison::Declared, LibPoison::Imported] {
        let transform = crate::batch::generate_vue_content_mapper_transform_with_options(
            Path::new("/project/src/Parent.vue"),
            &lib_parent_source(poison),
            options,
        )
        .expect("parent should project");
        // Listener closures plus the static and dynamic slot outlet tuples.
        assert_only_the_poison_is_unqualified(&transform.text);
    }
}

#[test]
fn authored_lib_types_cannot_capture_the_projection() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let lib_types = lib_types_module();
    let mut snapshots = Vec::new();
    for (case, poison) in [
        ("lib-type-shadowing-clean", LibPoison::None),
        ("lib-type-shadowing-declared", LibPoison::Declared),
        ("lib-type-shadowing-imported", LibPoison::Imported),
    ] {
        let parent = lib_parent_source(poison);
        let project_root = create_project_case(
            case,
            &[
                ("src/Child.vue", CHILD),
                ("src/Picker.vue", LIB_CHILD),
                ("src/lib-types.ts", &lib_types),
                ("src/Parent.vue", &parent),
            ],
        );
        let snapshot = snapshot_project_diagnostics(&project_root);
        if poison != LibPoison::None {
            let parent_path = project_root.join("src/Parent.vue");
            let mut project =
                VirtualProject::new(&project_root).expect("virtual project should open");
            project
                .register_paths(&[
                    project_root.join("src/Child.vue"),
                    project_root.join("src/Picker.vue"),
                    parent_path.clone(),
                ])
                .expect("fixture should register");
            let generated = &project
                .find_by_original(&parent_path)
                .expect("parent virtual file should exist")
                .content;
            assert_only_the_poison_is_unqualified(generated);
        }
        let _ = std::fs::remove_dir_all(&project_root);
        let Some(snapshot) = snapshot else {
            return;
        };
        snapshots.push(snapshot);
    }
    assert_eq!(snapshots[0], [], "the clean fixture is valid");
    assert!(
        snapshots.iter().all(|snapshot| *snapshot == snapshots[0]),
        "shadowing a lib type must not change any diagnostic: {snapshots:#?}"
    );
}
