//! Provenance parity for template ref unwrapping (#4146).
//!
//! The same authored template must type identically whether a composable is
//! imported by the SFC or injected by a framework auto-import: after the
//! auto-import transform runs, the name *is* a `<script setup>` import and Vue
//! unwraps its refs in the template. Every assertion below compares the
//! COMPLETE diagnostic list (code, message, authored line and column) of the
//! two provenances, and pins the exact authored diagnostics for the broken and
//! repaired variants.
//!
//! `vue-tsc` cannot serve as the oracle for the auto-import variant — it types
//! a template identifier against the component instance and reports
//! `Property 'x' does not exist` for every framework global (issue #913 owns
//! that divergence). The explicitly-imported variant is the vue-tsc-checked
//! side; `tests/snapshots/check/template-ref-unwrap-oracle.ts` runs that
//! comparison against the real `vue-tsc`.

#[path = "support/auto_import_project.rs"]
mod support;

use support::{Diagnostic, Project, corsa_available};

const COMPOSABLES: &str = r#"import {
  computed,
  readonly,
  ref,
  shallowRef,
  type ComputedRef,
  type Ref,
  type ShallowRef,
  type WritableComputedRef,
} from 'vue'

export interface Media { id: string; url: string }
export interface UserLogin { account: { id: string }; server: string; token?: string }

export const mediaList: Ref<Media[]> = ref([])
export const currentUser: ComputedRef<UserLogin | undefined> = computed(() => undefined)
export const draftName: WritableComputedRef<string> = computed({ get: () => 'draft', set: () => {} })
export const shallowList: ShallowRef<Media[]> = shallowRef([])
export const readonlyCount = readonly(ref(0))
export const nullableUser: Ref<UserLogin | null> = ref(null)
export const unionValue: Ref<string | number> = ref('a')
export const flag = ref(false)

// Not a ref: a plain constant that merely has a `value` property (#3767).
export const OPTION = { text: 'Login info', value: 'LOGIN_INFO' } as const
"#;

const CHILD: &str = r#"<script setup lang="ts">
import type { Media } from './composables'

defineProps<{ items: Media[]; label: string; count: number }>()
</script>

<template>
  <div />
</template>
"#;

/// Line 3 of `App.vue` is the only difference between the two provenances, and
/// both spellings are exactly one line, so every authored line and column in
/// the template is identical across them.
const EXPLICIT_LINE: &str = "import { OPTION, currentUser, draftName, flag, mediaList, nullableUser, readonlyCount, shallowList, unionValue } from './composables'";
const AUTO_LINE: &str = "// Auto-imported: OPTION currentUser draftName flag mediaList nullableUser readonlyCount shallowList unionValue";

fn app(provenance_line: &str, template: &str) -> String {
    format!(
        "<script setup lang=\"ts\">\nimport Child from './Child.vue'\n{provenance_line}\nconst scriptOnlyCount = mediaList.value.length\n</script>\n\n<template>\n{template}</template>\n"
    )
}

/// Reads, writes, nested member access, optional chaining, array length and
/// indexing, comparisons, component props, `v-if`, `v-for` and `v-model`, plus
/// the two negative controls (a plain `{ value }` constant and a script-only
/// `.value` read).
const CLEAN_TEMPLATE: &str = r#"  <div>{{ mediaList.length }}</div>
  <div>{{ mediaList[0]?.url }}</div>
  <div>{{ currentUser?.account.id }}</div>
  <div>{{ currentUser?.token }}</div>
  <div v-if="draftName === 'draft'">named</div>
  <div>{{ shallowList.length }}</div>
  <div>{{ readonlyCount + 1 }}</div>
  <div>{{ nullableUser?.server }}</div>
  <div>{{ unionValue }}</div>
  <div>{{ OPTION.value }}</div>
  <div>{{ scriptOnlyCount }}</div>
  <div v-if="flag">on</div>
  <div v-for="item in mediaList" :key="item.id">{{ item.url }}</div>
  <input v-model="draftName">
  <button @click="flag = !flag">toggle</button>
  <Child :items="mediaList" :label="draftName" :count="readonlyCount" />
"#;

/// The unwrapped array is a `Media[]`, so a misspelled member is `TS2551` on
/// `Media[]` — not `TS2339` on `Ref<Media[]>`, which is the #4146 RED.
const BROKEN_MEMBER_TEMPLATE: &str = "  <div>{{ mediaList.lenght }}</div>\n";

/// Unwrapping must not hide `.value` misuse inside a template.
const BROKEN_VALUE_TEMPLATE: &str = "  <div>{{ mediaList.value }}</div>\n";

/// A plain `{ text, value }` constant keeps its own `value` property, so
/// reading a member that does not exist on it still reports.
const BROKEN_PLAIN_OBJECT_TEMPLATE: &str = "  <div>{{ OPTION.valeu }}</div>\n";

/// The names `AUTO_LINE` stands for. Shared so the negative control below can
/// never drift onto a different binding set than the positive tests.
const AUTO_IMPORT_NAMES: &[&str] = &[
    "OPTION",
    "currentUser",
    "draftName",
    "flag",
    "mediaList",
    "nullableUser",
    "readonlyCount",
    "shallowList",
    "unionValue",
];

fn project(provenance_line: &str, template: &str) -> Project {
    let auto_import = provenance_line == AUTO_LINE;
    let mut project = Project::new();
    project.write("src/composables.ts", COMPOSABLES);
    project.write("src/Child.vue", CHILD);
    project.write("src/App.vue", &app(provenance_line, template));
    if auto_import {
        project.declare_auto_imports(AUTO_IMPORT_NAMES);
    }
    project
}

fn diagnostics(provenance_line: &str, template: &str) -> Vec<Diagnostic> {
    project(provenance_line, template).diagnostics("src/App.vue")
}

#[test]
fn clean_template_is_diagnostic_free_for_both_provenances() {
    if !corsa_available() {
        return;
    }
    let explicit = diagnostics(EXPLICIT_LINE, CLEAN_TEMPLATE);
    assert_eq!(explicit, Vec::<Diagnostic>::new());
    let auto = diagnostics(AUTO_LINE, CLEAN_TEMPLATE);
    assert_eq!(auto, Vec::<Diagnostic>::new());
}

#[test]
fn misspelled_member_reports_the_unwrapped_type_for_both_provenances() {
    if !corsa_available() {
        return;
    }
    let expected = vec![Diagnostic {
        code: Some(2551),
        line: 8,
        column: 21,
        message: "Property 'lenght' does not exist on type 'Media[]'. Did you mean 'length'?"
            .into(),
    }];
    assert_eq!(diagnostics(EXPLICIT_LINE, BROKEN_MEMBER_TEMPLATE), expected);
    assert_eq!(diagnostics(AUTO_LINE, BROKEN_MEMBER_TEMPLATE), expected);
}

#[test]
fn explicit_value_access_in_a_template_still_reports_for_both_provenances() {
    if !corsa_available() {
        return;
    }
    let expected = vec![Diagnostic {
        code: Some(2551),
        line: 8,
        column: 21,
        message: "Property 'value' does not exist on type 'Media[]'. Did you mean 'values'?".into(),
    }];
    assert_eq!(diagnostics(EXPLICIT_LINE, BROKEN_VALUE_TEMPLATE), expected);
    assert_eq!(diagnostics(AUTO_LINE, BROKEN_VALUE_TEMPLATE), expected);
}

#[test]
fn plain_value_objects_are_not_unwrapped_for_either_provenance() {
    if !corsa_available() {
        return;
    }
    let expected = vec![Diagnostic {
        code: Some(2551),
        line: 8,
        column: 18,
        message: "Property 'valeu' does not exist on type '{ readonly text: \"Login info\"; readonly value: \"LOGIN_INFO\"; }'. Did you mean 'value'?".into(),
    }];
    assert_eq!(
        diagnostics(EXPLICIT_LINE, BROKEN_PLAIN_OBJECT_TEMPLATE),
        expected
    );
    assert_eq!(
        diagnostics(AUTO_LINE, BROKEN_PLAIN_OBJECT_TEMPLATE),
        expected
    );
}

#[test]
fn repairing_the_template_restores_the_clean_verdict_for_both_provenances() {
    if !corsa_available() {
        return;
    }
    for provenance in [EXPLICIT_LINE, AUTO_LINE] {
        let broken = diagnostics(provenance, BROKEN_MEMBER_TEMPLATE);
        assert_eq!(broken.len(), 1, "{broken:#?}");
        let repaired = diagnostics(provenance, CLEAN_TEMPLATE);
        assert_eq!(repaired, Vec::<Diagnostic>::new());
    }
}

#[test]
fn auto_imported_refs_are_not_unwrapped_outside_the_template() {
    if !corsa_available() {
        return;
    }
    // `mediaList.value` in the script is correct; unwrapping there would break
    // it, and unwrapping is what makes `mediaList.length` correct in the
    // template. The clean fixture asserts both at once, so this pins the
    // failure mode explicitly: dropping `.value` in the script must report.
    let broken = app(AUTO_LINE, CLEAN_TEMPLATE).replace(
        "const scriptOnlyCount = mediaList.value.length",
        "const scriptOnlyCount = mediaList.length",
    );
    let mut project = Project::new();
    project.write("src/composables.ts", COMPOSABLES);
    project.write("src/Child.vue", CHILD);
    project.write("src/App.vue", &broken);
    project.declare_auto_imports(AUTO_IMPORT_NAMES);
    assert_eq!(
        project.diagnostics("src/App.vue"),
        vec![Diagnostic {
            code: Some(2339),
            line: 4,
            column: 35,
            message: "Property 'length' does not exist on type 'Ref<Media[], Media[]>'.".into(),
        }]
    );
}
