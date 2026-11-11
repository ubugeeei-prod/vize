use super::{findings, last_span_for, lint_sfc, lint_sfc_with_rule, none, span_for};
use crate::diagnostic::Severity;
use crate::rules::vue::{NoMutatingProps, NoMutatingPropsOptions};

// --- The recovered case: an assignment inside an inline handler ------------

#[test]
fn reports_prop_assignment_in_an_inline_handler() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ msg: string }>();
</script>

<template>
  <button @click="props.msg = 'x'">{{ props.msg }}</button>
</template>
"#;
    let (start, end) = span_for(sfc, "props.msg = 'x'");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            start,
            end,
            "Unexpected mutation of prop 'props.msg' in an inline handler",
        )]
    );
}

#[test]
fn reports_bare_prop_assignment_in_an_inline_handler() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <button @click="msg = 'x'">go</button>
</template>
"#;
    let (start, end) = span_for(sfc, "msg = 'x'");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            start,
            end,
            "Unexpected mutation of prop 'msg' in an inline handler",
        )]
    );
}

#[test]
fn reports_a_prop_update_expression() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ count: number }>();
</script>

<template>
  <button @click="props.count++">go</button>
</template>
"#;
    let (start, end) = span_for(sfc, "props.count++");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            start,
            end,
            "Unexpected mutation of prop 'props.count' in an inline handler",
        )]
    );
}

#[test]
fn reports_each_prop_mutation_of_a_multi_statement_handler() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string; count: number }>();
</script>

<template>
  <button @click="msg = 'x'; count = 1; msg = 'y'">go</button>
</template>
"#;
    let (first_msg_start, first_msg_end) = span_for(sfc, "msg = 'x'");
    let (count_start, count_end) = span_for(sfc, "count = 1");
    let (second_msg_start, second_msg_end) = span_for(sfc, "msg = 'y'");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![
            (
                "vue/no-mutating-props",
                Severity::Error,
                first_msg_start,
                first_msg_end,
                "Unexpected mutation of prop 'msg' in an inline handler",
            ),
            (
                "vue/no-mutating-props",
                Severity::Error,
                count_start,
                count_end,
                "Unexpected mutation of prop 'count' in an inline handler",
            ),
            (
                "vue/no-mutating-props",
                Severity::Error,
                second_msg_start,
                second_msg_end,
                "Unexpected mutation of prop 'msg' in an inline handler",
            ),
        ]
    );
}

#[test]
fn reports_delete_and_mutating_calls_in_an_inline_handler() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ items: string[]; profile: { name?: string } }>();
</script>

<template>
  <button @click="items.push('x'); items['push']('y'); props.items.splice(0, 1); props.items['splice'](0, 1); Object['assign'](props.profile, { name: 'Ada' }); delete props.profile.name">go</button>
</template>
"#;
    let (push_start, push_end) = span_for(sfc, "items.push('x')");
    let (bracket_push_start, bracket_push_end) = span_for(sfc, "items['push']('y')");
    let (splice_start, splice_end) = span_for(sfc, "props.items.splice(0, 1)");
    let (bracket_splice_start, bracket_splice_end) = span_for(sfc, "props.items['splice'](0, 1)");
    let (assign_start, assign_end) =
        span_for(sfc, "Object['assign'](props.profile, { name: 'Ada' })");
    let (delete_start, delete_end) = span_for(sfc, "delete props.profile.name");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![
            (
                "vue/no-mutating-props",
                Severity::Error,
                push_start,
                push_end,
                "Unexpected mutation of prop 'items' in an inline handler",
            ),
            (
                "vue/no-mutating-props",
                Severity::Error,
                bracket_push_start,
                bracket_push_end,
                "Unexpected mutation of prop 'items' in an inline handler",
            ),
            (
                "vue/no-mutating-props",
                Severity::Error,
                splice_start,
                splice_end,
                "Unexpected mutation of prop 'props.items' in an inline handler",
            ),
            (
                "vue/no-mutating-props",
                Severity::Error,
                bracket_splice_start,
                bracket_splice_end,
                "Unexpected mutation of prop 'props.items' in an inline handler",
            ),
            (
                "vue/no-mutating-props",
                Severity::Error,
                assign_start,
                assign_end,
                "Unexpected mutation of prop 'props.profile' in an inline handler",
            ),
            (
                "vue/no-mutating-props",
                Severity::Error,
                delete_start,
                delete_end,
                "Unexpected mutation of prop 'props.profile.name' in an inline handler",
            ),
        ]
    );
}

#[test]
fn ignores_non_literal_computed_calls_in_an_inline_handler() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ items: string[]; options: Record<string, boolean> }>();
const method = 'push';
const assign = 'assign';
</script>

<template>
  <button @click="props.items[method]('x'); Object[assign](props.options, { enabled: true })">go</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_copies_and_shallow_nested_mutations() {
    let copied = r#"<script setup lang="ts">
const props = defineProps<{ items: string[] }>();
const copy = [...props.items];
</script>
<template>
  <button @click="copy.push('x'); (() => props.items.push('y'))()">go</button>
  <button @click="props.items.slice().sort(); (items) => items.push('z')">copy</button>
</template>"#;
    let (start, end) = span_for(copied, "props.items.push('y')");
    assert_eq!(
        findings(&lint_sfc(copied)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            start,
            end,
            "Unexpected mutation of prop 'props.items' in an inline handler",
        )]
    );

    let shallow = r#"<script setup lang="ts">
const props = defineProps<{ items: string[] }>();
</script>
<template>
  <button @click="props.items.splice(0, 1); props.items = []">go</button>
</template>"#;
    let (start, end) = span_for(shallow, "props.items = []");
    let result = lint_sfc_with_rule(
        shallow,
        NoMutatingProps::new(NoMutatingPropsOptions { shallow_only: true }),
    );
    assert_eq!(
        findings(&result),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            start,
            end,
            "Unexpected mutation of prop 'props.items' in an inline handler",
        )]
    );
}

#[test]
fn reports_mutations_in_every_template_expression_kind() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ rows: string[] }>();
</script>
<template>
  <ul><li v-for="row in rows.sort()" :key="row">{{ row }}</li></ul>
  <p>{{ rows.reverse().join(", ") }}</p>
  <MyList :items="rows.splice(0, 3)" />
  <p v-if="rows.fill('x')">filled</p>
</template>"#;
    let result = lint_sfc(sfc);
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.start, diagnostic.end))
            .collect::<Vec<_>>(),
        [
            "rows.sort()",
            "rows.reverse()",
            "rows.splice(0, 3)",
            "rows.fill('x')",
        ]
        .map(|expression| span_for(sfc, expression))
    );
}

#[test]
fn ignores_copied_shadowed_and_shallow_template_mutations() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ rows: string[]; groups: string[][] }>();
</script>
<template>
  <p>{{ [...rows].sort() }}</p>
  <p>{{ rows.slice().sort() }}</p>
  <p>{{ groups.map(rows => rows.sort()) }}</p>
  <MyList :items="rows.splice(0, 3)" />
</template>"#;
    let result = lint_sfc_with_rule(
        sfc,
        NoMutatingProps::new(NoMutatingPropsOptions { shallow_only: true }),
    );
    assert_eq!(findings(&result), none());
}

// --- The template does not mutate a prop: exactly zero findings ------------

#[test]
fn ignores_a_handler_that_reads_a_prop() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ msg: string }>();
const emit = defineEmits<{ go: [string] }>();
</script>

<template>
  <button @click="emit('go', props.msg)">{{ props.msg }}</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_assignment_to_local_state() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue';
defineProps<{ msg: string }>();
const draft = ref('');
</script>

<template>
  <button @click="draft = 'x'">go</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- Over-match probes: none of these may manufacture a finding ------------

#[test]
fn ignores_a_prop_assignment_inside_a_string_literal() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <button @click="console.log('msg = 1')">go</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_identifier_that_merely_starts_with_a_prop_name() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue';
defineProps<{ msg: string }>();
const msgExtra = ref('');
</script>

<template>
  <button @click="msgExtra = 'x'">go</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_v_for_alias_that_shadows_a_prop_name() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string; rows: string[] }>();
</script>

<template>
  <ul>
    <li v-for="msg in rows" :key="msg">
      <button @click="msg = 'x'">go</button>
    </li>
  </ul>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_slot_variable_that_shadows_a_prop_name() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <Child v-slot="{ msg }">
    <button @click="msg = 'x'">go</button>
  </Child>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn still_reports_a_prop_after_a_shadowing_subtree_ends() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string; rows: string[] }>();
</script>

<template>
  <ul>
    <li v-for="msg in rows" :key="msg">
      <button @click="msg = 'x'">shadowed</button>
    </li>
  </ul>
  <button @click="msg = 'y'">not shadowed</button>
</template>
"#;
    let (start, end) = last_span_for(sfc, "msg = 'y'");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            start,
            end,
            "Unexpected mutation of prop 'msg' in an inline handler",
        )]
    );
}
