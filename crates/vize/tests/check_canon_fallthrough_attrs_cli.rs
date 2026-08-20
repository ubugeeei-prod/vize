#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "check_canon_fallthrough_attrs_cli/support.rs"]
mod support;
use support::{
    assert_clean, assert_error_mentions, create_case, create_case_with_files,
    resolve_test_corsa_path, run_check_json,
};
#[test]
fn check_fallthrough_attrs_follow_inherit_attrs_and_root_shape() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };

    struct Case<'a> {
        id: &'a str,
        child: &'a str,
        app: &'a str,
        expected_error_fragments: &'a [&'a str],
    }

    let cases = [
        Case {
            id: "fallthrough-mono-ok",
            child: r#"<script setup lang="ts">
defineProps<{ title: string }>();
</script>
<template><div class="root">{{ title }}</div></template>
"#,
            app: r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" id="outer" aria-haspopup="menu" /></template>
"#,
            expected_error_fragments: &[],
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
<template><Child title="ok" id="outer" class="card" style="color: red" /></template>
"#,
            expected_error_fragments: &["id"],
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
<template><Child title="ok" :disabled="'nope'" /></template>
"#,
            expected_error_fragments: &["disabled"],
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
<template><Child title="ok" id="outer" /></template>
"#,
            expected_error_fragments: &["id"],
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
<template><Child title="ok" id="outer" /></template>
"#,
            expected_error_fragments: &["id"],
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
<template><Child title="ok" id="outer" /></template>
"#,
            expected_error_fragments: &["id"],
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
            expected_error_fragments: &["\"nope\"", "Booleanish"],
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
            expected_error_fragments: &[],
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
<template><Child title="ok" :on="on" id="outer" /></template>
"#,
            expected_error_fragments: &["id"],
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
<template><Child title="ok" :on="on" id="outer" /></template>
"#,
            expected_error_fragments: &["id"],
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
<template><Child title="ok" :items="[1, 2]" id="outer" /></template>
"#,
            expected_error_fragments: &["id"],
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
<template><Child title="ok" :items="[1, 2]" id="outer" /></template>
"#,
            expected_error_fragments: &["id"],
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
            expected_error_fragments: &[],
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
            expected_error_fragments: &[],
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
<template><Child title="ok" id="outer" /></template>
"#,
            expected_error_fragments: &["id"],
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
            expected_error_fragments: &[],
        },
    ];

    for case in cases {
        let project_root = create_case(case.id, case.child, case.app);
        let report = run_check_json(&project_root, &corsa_path);
        if case.expected_error_fragments.is_empty() {
            assert_clean(case.id, &report);
        } else {
            assert_error_mentions(case.id, &report, case.expected_error_fragments);
        }
        let _ = std::fs::remove_dir_all(project_root);
    }
}

#[test]
fn check_fallthrough_attrs_keep_single_component_root_forwarding_open() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };

    let project_root = create_case_with_files(
        "fallthrough-component-root-open",
        r#"<script setup lang="ts">
import BaseInput from "./BaseInput.vue";

defineProps<{ title: string }>();
</script>
<template><BaseInput /></template>
"#,
        r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" model-value="draft" w-48px /></template>
"#,
        &[(
            "BaseInput.vue",
            r#"<script setup lang="ts">
defineProps<{ modelValue?: string }>();
</script>
<template><input :value="modelValue" /></template>
"#,
        )],
    );

    let report = run_check_json(&project_root, &corsa_path);
    assert_clean("fallthrough-component-root-open", &report);
    let _ = std::fs::remove_dir_all(project_root);
}
