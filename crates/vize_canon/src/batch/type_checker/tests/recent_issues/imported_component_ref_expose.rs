//! Template refs to an imported component keep the child's public surface
//! (#4150).
//!
//! `vue-tsc` compiles a plain SFC to a `DefineComponent`, so `typeof Child`
//! inherits the `[key: string]: any` that `ComponentOptionsBase` picks up from
//! `LegacyOptions`. Misskey's `MkImgPreviewDialog.vue` relies on it:
//!
//! ```ts
//! const modal = ref<typeof MkModalWindow | null>(null);
//! modal.value?.close();
//! ```
//!
//! Vize's generated options object declared only the known runtime option keys,
//! so every member of the child's expose surface reported `TS2339`.
//!
//! The expectations below are the exact output of
//! `vue-tsc --noEmit` (vue 3.6.0-beta.10) on the same sources; see the PR for
//! the transcript. Only the printed type name differs, because each tool names
//! its own generated instance type.

use super::super::{
    BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::TypeChecker;

type ReportedDiagnostic = (String, Option<u32>, String, u32, u32);

fn check(case: &str, files: &[(&str, &str)]) -> Vec<ReportedDiagnostic> {
    let project_root = create_project_case(case, files);
    let mut checker = BatchTypeChecker::new(&project_root).expect("batch checker construction");
    checker.scan_project().expect("project scan");
    let result = checker.check_project().expect("project check");
    let mut diagnostics: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file).into(),
                diagnostic.code,
                diagnostic.message.into(),
                diagnostic.line + 1,
                diagnostic.column + 1,
            )
        })
        .collect();
    diagnostics.sort();
    let _ = std::fs::remove_dir_all(&project_root);
    diagnostics
}

fn shared_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("src/MkModalWindow.vue", MK_MODAL_WINDOW),
        ("src/TypedExpose.vue", TYPED_EXPOSE),
        ("src/GenericList.vue", GENERIC_LIST),
        ("src/PropsChild.vue", PROPS_CHILD),
        ("src/pkg-widget.d.ts", PKG_WIDGET_TYPES),
        ("src/MkImgPreviewDialog.vue", MK_IMG_PREVIEW_DIALOG),
        ("src/valid.ts", VALID),
    ]
}

#[test]
fn imported_component_refs_match_the_vue_tsc_public_surface() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let mut files = shared_files();
    files.push(("src/invalid.ts", INVALID));
    files.push(("src/NegativeControls.vue", NEGATIVE_CONTROLS));

    assert_eq!(
        check("issue-4150-imported-component-ref-expose", &files),
        vec![
            (
                "src/NegativeControls.vue".into(),
                Some(2322),
                "Type 'number' is not assignable to type 'string'.".into(),
                6,
                16,
            ),
            (
                "src/NegativeControls.vue".into(),
                Some(2322),
                "Type 'string' is not assignable to type 'number'.".into(),
                8,
                27,
            ),
            (
                "src/NegativeControls.vue".into(),
                Some(2345),
                "Argument of type '{}' is not assignable to parameter of type '{ readonly label: string; readonly count?: number | undefined; } & __VizePublicComponentAttrs & { [x: string]: unknown; } & Partial<{}>'.\nProperty 'label' is missing in type '{}' but required in type '{ readonly label: string; readonly count?: number | undefined; }'.".into(),
                7,
                4,
            ),
            (
                "src/invalid.ts".into(),
                Some(2339),
                "Property 'absentMember' does not exist on type '__VizeComponentInstance'.".into(),
                10,
                28,
            ),
            (
                "src/invalid.ts".into(),
                Some(2339),
                "Property 'absentMember' does not exist on type '__VizeComponentInstance'.".into(),
                12,
                33,
            ),
            (
                "src/invalid.ts".into(),
                Some(2339),
                "Property 'absentMember' does not exist on type '__VizeComponentInstance'.".into(),
                14,
                38,
            ),
            (
                "src/invalid.ts".into(),
                Some(2339),
                "Property 'secret' does not exist on type '__VizeComponentInstance'.".into(),
                7,
                28,
            ),
        ],
        "imported component refs must report exactly what vue-tsc reports"
    );
}

#[test]
fn repairing_the_member_names_clears_every_diagnostic() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let mut files = shared_files();
    files.push(("src/invalid.ts", REPAIRED));
    files.push(("src/NegativeControls.vue", REPAIRED_CONTROLS));

    assert_eq!(
        check("issue-4150-imported-component-ref-expose-repaired", &files),
        vec![],
        "the repaired sources are clean under vue-tsc, so they must be clean here"
    );
}

const MK_MODAL_WINDOW: &str = r#"<script setup lang="ts">
import { ref } from "vue";

const showing = ref(true);
const secret = ref("private");

function close(): void {
  showing.value = false;
}

defineExpose({ close });
</script>

<template>
  <div v-if="showing">{{ secret }}</div>
</template>
"#;

const TYPED_EXPOSE: &str = r#"<script setup lang="ts">
defineExpose<{ close(): void }>();
</script>

<template>
  <div />
</template>
"#;

/// Consumed by nothing but the project itself, on purpose: a generic SFC carries
/// no options index signature, so its own generated module must still check clean
/// in both lanes above (neither expectation carries a `GenericList.vue` row).
/// Member access on a generic component is not asserted here, because
/// `InstanceType<typeof GenericList>` is a recorded vue-tsc divergence (vue-tsc
/// reports `TS2344` for it, vize resolves the non-generic instance) while this
/// suite asserts byte-exact vue-tsc parity. The exclusion rule itself is asserted
/// on the generated module by
/// `component_options_index_signature::generic_component_options_omit_the_string_index_signature`.
const GENERIC_LIST: &str = r#"<script setup lang="ts" generic="T extends { id: string }">
const props = defineProps<{ items: T[] }>();

function pick(index: number): T {
  return props.items[index];
}

defineExpose({ pick });
</script>

<template>
  <div>{{ items.length }}</div>
</template>
"#;

const PROPS_CHILD: &str = r#"<script setup lang="ts">
defineProps<{ label: string; count?: number }>();

function reset(): void {}

defineExpose({ reset });
</script>

<template>
  <div>{{ label }}</div>
</template>
"#;

const PKG_WIDGET_TYPES: &str = r#"declare module "pkg-widget" {
  const PkgWidget: new () => { dismiss(): void };
  export default PkgWidget;
}
"#;

/// Misskey's `MkImgPreviewDialog.vue`, reduced to the reported shape.
const MK_IMG_PREVIEW_DIALOG: &str = r#"<script setup lang="ts">
import { ref } from "vue";
import MkModalWindow from "./MkModalWindow.vue";

const modal = ref<typeof MkModalWindow | null>(null);

function onClose(): void {
  modal.value?.close();
}
</script>

<template>
  <MkModalWindow ref="modal" @click="onClose" />
</template>
"#;

const VALID: &str = r#"import { ref, shallowRef, useTemplateRef } from "vue";
import MkModalWindow from "./MkModalWindow.vue";
import TypedExpose from "./TypedExpose.vue";
import PkgWidget from "pkg-widget";

// Component value type: Vue's own `LegacyOptions` index signature makes every
// member access resolve, so none of these may report.
declare const value: typeof MkModalWindow;
export const v1 = value.close;
export const v2 = value.secret;
export const v3 = value.name;

// Nullable template refs written against the component value type (Misskey).
const nullableRef = ref<typeof MkModalWindow | null>(null);
export const r1 = nullableRef.value?.close();
const typedNullableRef = ref<typeof TypedExpose | null>(null);
export const r2 = typedNullableRef.value?.close();

// Public instance surfaces resolve the declared expose members.
declare const instance: InstanceType<typeof MkModalWindow>;
export const i1 = instance.close();
declare const typedInstance: InstanceType<typeof TypedExpose>;
export const i2 = typedInstance.close();
declare const pkgInstance: InstanceType<typeof PkgWidget>;
export const i3 = pkgInstance.dismiss();

const instanceRef = ref<InstanceType<typeof MkModalWindow> | null>(null);
export const i4 = instanceRef.value?.close();
const shallow = shallowRef<InstanceType<typeof MkModalWindow> | null>(null);
export const i5 = shallow.value?.close();
const templateRef = useTemplateRef<InstanceType<typeof MkModalWindow>>("modal");
export const i6 = templateRef.value?.close();
"#;

const INVALID: &str = r#"import { ref } from "vue";
import MkModalWindow from "./MkModalWindow.vue";
import TypedExpose from "./TypedExpose.vue";

// A private setup binding is not part of the expose surface.
declare const instance: InstanceType<typeof MkModalWindow>;
export const p1 = instance.secret;

// A genuinely absent member still reports; it is never widened to `any`.
export const p2 = instance.absentMember;
declare const typedInstance: InstanceType<typeof TypedExpose>;
export const p3 = typedInstance.absentMember;
const instanceRef = ref<InstanceType<typeof MkModalWindow> | null>(null);
export const p4 = instanceRef.value?.absentMember();
"#;

const REPAIRED: &str = r#"import { ref } from "vue";
import MkModalWindow from "./MkModalWindow.vue";
import TypedExpose from "./TypedExpose.vue";

// Repaired: every member below is on the declared expose surface.
declare const instance: InstanceType<typeof MkModalWindow>;
export const p1 = instance.close;

export const p2 = instance.close();
declare const typedInstance: InstanceType<typeof TypedExpose>;
export const p3 = typedInstance.close;
const instanceRef = ref<InstanceType<typeof MkModalWindow> | null>(null);
export const p4 = instanceRef.value?.close();
"#;

/// Prop checking must stay exact: the options index signature lives on the
/// component value, never on the constructor's parameter.
const NEGATIVE_CONTROLS: &str = r#"<script setup lang="ts">
import PropsChild from "./PropsChild.vue";
</script>

<template>
  <PropsChild :label="1" />
  <PropsChild />
  <PropsChild label="ok" :count="'no'" />
  <PropsChild label="ok" :count="2" />
</template>
"#;

const REPAIRED_CONTROLS: &str = r#"<script setup lang="ts">
import PropsChild from "./PropsChild.vue";
</script>

<template>
  <PropsChild label="ok" />
  <PropsChild label="ok" :count="2" />
</template>
"#;
