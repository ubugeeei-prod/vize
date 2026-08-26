//! The fallthrough case table, split under the 350-line source budget.
//!
//! Closed-surface cases probe with `depressed`: real HTML attribute names
//! (`id`, `disabled`, …) are accepted as fallthrough everywhere since #4966,
//! so only a name no native element declares still distinguishes the shapes.

pub(crate) struct Case<'a> {
    pub(crate) id: &'a str,
    pub(crate) child: &'a str,
    pub(crate) app: &'a str,
    pub(crate) expected_diagnostics: &'a [&'a str],
}

const MONO_DIV_CHILD: &str = r#"<script setup lang="ts">defineProps<{ title: string }>();</script>
<template><div class="root">{{ title }}</div></template>
"#;

pub(crate) fn cases() -> [Case<'static>; 19] {
    [
        Case {
            id: "fallthrough-mono-ok",
            child: MONO_DIV_CHILD,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" id="outer" aria-haspopup="menu" data-test-id="child" /></template>
"#,
            expected_diagnostics: &[],
        },
        Case {
            id: "fallthrough-aria-camel-ok",
            child: MONO_DIV_CHILD,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" aria-label="Open" /></template>
"#,
            expected_diagnostics: &[],
        },
        Case {
            id: "fallthrough-unknown-attrs-ok",
            child: MONO_DIV_CHILD,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" aria-zzz="1" /></template>
"#,
            expected_diagnostics: &[],
        },
        Case {
            id: "fallthrough-generic-data-custom-ok",
            child: r#"<script setup lang="ts" generic="T extends string">defineProps<{ title: T }>();</script>
<template><div class="root">{{ title }}</div></template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" data-slot="root" data-scroll-anchor="1" /></template>
"#,
            expected_diagnostics: &[],
        },
        Case {
            id: "fallthrough-mono-false-bad",
            child: r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
defineProps<{ title: string }>();
</script>
<template><div class="root">{{ title }}</div></template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" depressed="x" class="card" style="color: red" /></template>
"#,
            expected_diagnostics: &[
                "error:4:29 [TS2353] Object literal may only specify known properties, and '\"depressed\"' does not exist in type '__VizeComponentCheckProps<Props, __VizeCheckTail>'.",
            ],
        },
        Case {
            id: "fallthrough-mono-false-native-binding-bad",
            child: r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
defineProps<{ title: string }>();
</script>
<template><button type="button">{{ title }}</button></template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" :depressed="'nope'" /></template>
"#,
            expected_diagnostics: &[
                "error:4:30 [TS2353] Object literal may only specify known properties, and '\"depressed\"' does not exist in type '__VizeComponentCheckProps<Props, __VizeCheckTail>'.",
            ],
        },
        Case {
            id: "fallthrough-options-api-inherit-attrs-false-bad",
            child: r#"<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  inheritAttrs: false,
  props: {
    title: { type: String, required: true },
  },
});
</script>
<template><div class="root">{{ title }}</div></template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" depressed="x" /></template>
"#,
            expected_diagnostics: &[
                "error:4:29 [TS2353] Object literal may only specify known properties, and '\"depressed\"' does not exist in type '__VizeComponentCheckProps<__Child_CheckProps_0, __VizeCheckTail>'.",
            ],
        },
        Case {
            id: "fallthrough-multi-bad",
            child: r#"<script setup lang="ts">
defineProps<{ title: string }>();
</script>
<template>
  <div class="a">{{ title }}</div>
  <span class="b">x</span>
</template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" depressed="x" /></template>
"#,
            expected_diagnostics: &[
                "error:4:29 [TS2353] Object literal may only specify known properties, and '\"depressed\"' does not exist in type '__VizeComponentCheckProps<Props, __VizeCheckTail>'.",
            ],
        },
        Case {
            id: "fallthrough-multi-false-bad",
            child: r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
defineProps<{ title: string }>();
</script>
<template>
  <div class="a">{{ title }}</div>
  <span class="b">x</span>
</template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" depressed="x" /></template>
"#,
            expected_diagnostics: &[
                "error:4:29 [TS2353] Object literal may only specify known properties, and '\"depressed\"' does not exist in type '__VizeComponentCheckProps<Props, __VizeCheckTail>'.",
            ],
        },
        Case {
            id: "fallthrough-native-type-bad",
            child: r#"<script setup lang="ts">
defineProps<{ title: string }>();
</script>
<template><button type="button">{{ title }}</button></template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" :disabled="'nope'" /></template>
"#,
            expected_diagnostics: &[
                "error:4:30 [TS2322] Type '\"nope\"' is not assignable to type 'Booleanish | undefined'.",
            ],
        },
        Case {
            id: "fallthrough-vif-both-mono-ok",
            child: r#"<script setup lang="ts">
defineProps<{ title: string; on: boolean }>();
</script>
<template>
  <div v-if="on" class="on">{{ title }}</div>
  <span v-else class="off">{{ title }}</span>
</template>
"#,
            app: r#"<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";
const on = ref(true);
</script>
<template><Child title="ok" :on="on" id="outer" /></template>
"#,
            expected_diagnostics: &[],
        },
        Case {
            id: "fallthrough-vif-both-mono-false-bad",
            child: r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
defineProps<{ title: string; on: boolean }>();
</script>
<template>
  <div v-if="on" class="on">{{ title }}</div>
  <span v-else class="off">{{ title }}</span>
</template>
"#,
            app: r#"<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";
const on = ref(true);
</script>
<template><Child title="ok" :on="on" depressed="x" /></template>
"#,
            expected_diagnostics: &[
                "error:6:38 [TS2353] Object literal may only specify known properties, and '\"depressed\"' does not exist in type '__VizeComponentCheckProps<Props, __VizeCheckTail>'.",
            ],
        },
        Case {
            id: "fallthrough-vif-mono-multi-bad",
            child: r#"<script setup lang="ts">
defineProps<{ title: string; on: boolean }>();
</script>
<template>
  <div v-if="on" class="on">{{ title }}</div>
  <template v-else>
    <p class="a">{{ title }}</p>
    <p class="b">x</p>
  </template>
</template>
"#,
            app: r#"<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";
const on = ref(true);
</script>
<template><Child title="ok" :on="on" depressed="x" /></template>
"#,
            expected_diagnostics: &[
                "error:6:38 [TS2353] Object literal may only specify known properties, and '\"depressed\"' does not exist in type '__VizeComponentCheckProps<Props, __VizeCheckTail>'.",
            ],
        },
        Case {
            id: "fallthrough-root-vfor-bad",
            child: r#"<script setup lang="ts">
defineProps<{ title: string; items: number[] }>();
</script>
<template>
  <div v-for="item in items" :key="item">{{ title }}</div>
</template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" :items="[1, 2]" depressed="x" /></template>
"#,
            expected_diagnostics: &[
                "error:4:45 [TS2353] Object literal may only specify known properties, and '\"depressed\"' does not exist in type '__VizeComponentCheckProps<Props, __VizeCheckTail>'.",
            ],
        },
        Case {
            id: "fallthrough-template-root-vfor-bad",
            child: r#"<script setup lang="ts">
defineProps<{ title: string; items: number[] }>();
</script>
<template>
  <template v-for="item in items" :key="item">
    <div>{{ title }}</div>
  </template>
</template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" :items="[1, 2]" depressed="x" /></template>
"#,
            expected_diagnostics: &[
                "error:4:45 [TS2353] Object literal may only specify known properties, and '\"depressed\"' does not exist in type '__VizeComponentCheckProps<Props, __VizeCheckTail>'.",
            ],
        },
        Case {
            id: "fallthrough-vif-static-ok",
            child: r#"<script setup lang="ts">
defineProps<{ title: string }>();
</script>
<template>
  <div v-if="true" class="on">{{ title }}</div>
  <span v-else class="off">{{ title }}</span>
</template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" id="outer" /></template>
"#,
            expected_diagnostics: &[],
        },
        Case {
            id: "fallthrough-vif-elseif-static-ok",
            child: r#"<script setup lang="ts">
defineProps<{ title: string; on: boolean }>();
</script>
<template>
  <div v-if="on" class="on">{{ title }}</div>
  <span v-else-if="true" class="off">{{ title }}</span>
</template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" :on="false" id="outer" /></template>
"#,
            expected_diagnostics: &[],
        },
        Case {
            id: "fallthrough-vif-static-multi-bad",
            child: r#"<script setup lang="ts">
defineProps<{ title: string }>();
</script>
<template>
  <template v-if="true">
    <div class="a">{{ title }}</div>
    <span class="b">x</span>
  </template>
</template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" depressed="x" /></template>
"#,
            expected_diagnostics: &[
                "error:4:29 [TS2353] Object literal may only specify known properties, and '\"depressed\"' does not exist in type '__VizeComponentCheckProps<Props, __VizeCheckTail>'.",
            ],
        },
        Case {
            id: "fallthrough-vif-static-prop-ok",
            child: r#"<script setup lang="ts">
defineProps<{ title: string; alwaysOn: true }>();
</script>
<template>
  <div v-if="alwaysOn" class="on">{{ title }}</div>
  <span v-else class="off">{{ title }}</span>
</template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" :alwaysOn="true" id="outer" /></template>
"#,
            expected_diagnostics: &[],
        },
    ]
}
