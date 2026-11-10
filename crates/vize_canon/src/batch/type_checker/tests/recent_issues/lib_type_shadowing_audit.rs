//! Final lib/global-name audit for the per-SFC and embedded projections
//! (#12536). Every active global below is poisoned at the authored module
//! boundary; generated references must keep the clean project's diagnostics.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use crate::batch::VirtualProject;
use oxc_syntax::identifier::is_identifier_part;
use std::path::Path;

const LIB_TYPES: &[&str] = &[
    "AnimationEvent",
    "ArrayConstructor",
    "Awaited",
    "BooleanConstructor",
    "Capitalize",
    "ClipboardEvent",
    "CompositionEvent",
    "Date",
    "DateConstructor",
    "DragEvent",
    "Element",
    "ErrorEvent",
    "Event",
    "Exclude",
    "Extract",
    "FocusEvent",
    "FormDataEvent",
    "Function",
    "FunctionConstructor",
    "GlobalEventHandlersEventMap",
    "HashChangeEvent",
    "HTMLElementTagNameMap",
    "InputEvent",
    "InstanceType",
    "Iterable",
    "KeyboardEvent",
    "Lowercase",
    "MathMLElementTagNameMap",
    "MessageEvent",
    "MouseEvent",
    "NoInfer",
    "NonNullable",
    "NumberConstructor",
    "ObjectConstructor",
    "Omit",
    "Parameters",
    "Partial",
    "Pick",
    "PointerEvent",
    "PopStateEvent",
    "PropertyKey",
    "Readonly",
    "Record",
    "Required",
    "ReturnType",
    "SecurityPolicyViolationEvent",
    "StorageEvent",
    "StringConstructor",
    "SubmitEvent",
    "SVGElementTagNameMap",
    "ToggleEvent",
    "TouchEvent",
    "TransitionEvent",
    "Uncapitalize",
    "WheelEvent",
];

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ value: string; transform: (value: string) => string }>()
const emit = defineEmits<{ save: [value: string] }>()
defineSlots<{ default(props: { item: string }): any }>()
defineExpose({ focus: () => undefined })
</script>

<template>
  <button @click="emit('save', value)"><slot :item="value" /></button>
</template>
"#;

#[derive(Clone, Copy, PartialEq)]
enum Poison {
    None,
    Declared,
    Imported,
}

fn module_script(poison: Poison) -> std::string::String {
    let mut script = std::string::String::from("<script lang=\"ts\">\n");
    for name in LIB_TYPES {
        match poison {
            Poison::None => script.push_str(&format!("type Unshadowed{name} = never\n")),
            Poison::Declared => script.push_str(&format!("type {name} = never\n")),
            Poison::Imported => {
                script.push_str(&format!("import type {{ {name} }} from './lib-types'\n"));
            }
        }
    }
    script.push_str(
        "export const libTypePoison = true\nexport default { name: 'LibTypeAudit' }\n</script>\n",
    );
    script
}

fn parent_source(poison: Poison) -> std::string::String {
    format!(
        r#"<!-- @inferComponentDollarEl true -->
{}
<script setup lang="ts">
import {{ ref, useCssModule, useTemplateRef }} from 'vue'
import Child from './Child.vue'

const mode = ref(0)
const value = ref('ok')
const slotName = ref('default')
const button = useTemplateRef('button')
const styles = useCssModule()
const onEvent = (_event: unknown) => undefined
const onSave = (saved: string) => saved.toUpperCase()
void button
</script>

<template>
  <div
    v-if="mode === 0"
    :class="styles.root"
    @dblclick="onEvent"
    @click="onEvent"
    @touchstart="onEvent"
    @keydown="onEvent"
    @focus="onEvent"
    @input="onEvent"
    @compositionstart="onEvent"
    @submit="onEvent"
    @change="onEvent"
    @drag="onEvent"
    @copy="onEvent"
    @wheel="onEvent"
    @animationstart="onEvent"
    @transitionend="onEvent"
    @error="onEvent"
    @toggle="onEvent"
    @formdata="onEvent"
    @popstate="onEvent"
    @hashchange="onEvent"
    @message="onEvent"
    @storage="onEvent"
    @securitypolicyviolation="onEvent"
  >
    <Child
      :value="value"
      :transform="item => item.toUpperCase()"
      :ref="instance => instance?.focus()"
      @save="onSave"
    >
      <template #[slotName]="{{ item }}">{{{{ item.toUpperCase() }}}}</template>
    </Child>
    <Transition @enter="onEvent"><button ref="button">ok</button></Transition>
    <GlobalWidget :value="value" @save="onSave" />
    <slot :name="slotName" v-bind="{{ value }}" />
  </div>
  <svg v-else-if="mode === 1"><circle /></svg>
  <math v-else><mi>{{{{ value }}}}</mi></math>
</template>

<style module>.root {{ color: red; }}</style>
"#,
        module_script(poison)
    )
}

fn lib_types_module() -> std::string::String {
    LIB_TYPES
        .iter()
        .map(|name| format!("export type {name} = never\n"))
        .collect()
}

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

fn unqualified_lines<'a>(source: &'a str, name: &str) -> Vec<&'a str> {
    source
        .lines()
        .filter(|line| unqualified_count(line, name) != 0)
        .collect()
}

fn assert_every_generated_reference_is_owned(generated: &str) {
    for name in LIB_TYPES {
        assert_eq!(
            unqualified_count(generated, name),
            1,
            "only the authored poison may spell {name} unqualified: {:#?}",
            unqualified_lines(generated, name)
        );
        let qualified = generated.matches(&format!("globalThis.{name}")).count();
        let owned_pick = (*name == "Pick")
            .then(|| generated.matches("__vizePick<").count())
            .unwrap_or_default();
        assert_ne!(
            qualified + owned_pick,
            0,
            "the fixture must activate a generated {name} reference"
        );
    }
}

#[test]
fn embedded_and_per_sfc_projections_own_every_lib_type_reference() {
    let options = crate::batch::ContentMapperTransformOptions::default()
        .with_external_component_attrs_module(false)
        .with_external_component_type_helpers(false)
        .with_shared_virtual_module(false);
    for poison in [Poison::Declared, Poison::Imported] {
        let transform = crate::batch::generate_vue_content_mapper_transform_with_options(
            Path::new("/project/src/Parent.vue"),
            &parent_source(poison),
            options,
        )
        .expect("audit host should project");
        assert_every_generated_reference_is_owned(&transform.text);
    }
}

#[test]
fn authored_lib_types_do_not_change_batch_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let lib_types = lib_types_module();
    let mut snapshots = Vec::new();
    for (case, poison) in [
        ("lib-type-audit-clean", Poison::None),
        ("lib-type-audit-declared", Poison::Declared),
        ("lib-type-audit-imported", Poison::Imported),
    ] {
        let parent = parent_source(poison);
        let project_root = create_project_case(
            case,
            &[
                ("src/Child.vue", CHILD),
                ("src/lib-types.ts", &lib_types),
                ("src/Parent.vue", &parent),
            ],
        );
        let snapshot = snapshot_project_diagnostics(&project_root);
        if poison != Poison::None {
            let parent_path = project_root.join("src/Parent.vue");
            let mut project =
                VirtualProject::new(&project_root).expect("virtual project should open");
            project
                .register_paths(&[project_root.join("src/Child.vue"), parent_path.clone()])
                .expect("audit fixture should register");
            let generated = &project
                .find_by_original(&parent_path)
                .expect("parent virtual file should exist")
                .content;
            for name in LIB_TYPES {
                assert_eq!(
                    unqualified_count(generated, name),
                    1,
                    "only the authored poison may spell {name} unqualified: {:#?}",
                    unqualified_lines(generated, name)
                );
            }
        }
        let _ = std::fs::remove_dir_all(&project_root);
        let Some(snapshot) = snapshot else {
            return;
        };
        snapshots.push(snapshot);
    }
    assert_eq!(snapshots[0], [], "the clean audit fixture is valid");
    assert!(
        snapshots.iter().all(|snapshot| *snapshot == snapshots[0]),
        "shadowing lib types must not change diagnostics: {snapshots:#?}"
    );
}
