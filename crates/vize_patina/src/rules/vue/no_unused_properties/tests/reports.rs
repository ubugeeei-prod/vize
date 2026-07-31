//! The positive direction: a declared prop referenced nowhere.

use super::{lint_sfc, owned, unused};

// --- The recovered case: an unassigned defineProps -------------------------

#[test]
fn anchors_an_unused_prop_at_its_declaration_issue_3520() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "msg", "msg: string")]
    );
}

#[test]
fn anchors_after_a_template_that_precedes_script_setup() {
    let sfc = r#"<template>
  <div>🦀</div>
</template>

<script setup lang="ts">
defineProps<{ trailing: string }>();
</script>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "trailing", "trailing: string")]
    );
}

#[test]
fn anchors_a_split_script_prop_after_normalizing_merged_macro_offsets() {
    let sfc = r#"<script>
export const moduleValue = 1;
</script>

<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template><div>hi</div></template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "msg", "msg: string")]
    );
}

#[test]
fn split_script_indirect_type_falls_back_to_the_normalized_macro_call() {
    let sfc = r#"<script>
export const moduleValue = 1;
</script>

<script setup lang="ts">
defineProps<Pick<{ msg: string }, 'msg'>>();
</script>

<template><div>hi</div></template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(
            sfc,
            "msg",
            "defineProps<Pick<{ msg: string }, 'msg'>>()"
        )]
    );
}

#[test]
fn split_script_runtime_spread_falls_back_to_the_normalized_macro_call() {
    let sfc = r#"<script>
export const moduleValue = 1;
</script>

<script setup>
const common = { 'msg-prop': String }
defineProps({ ...common })
</script>

<template><div>hi</div></template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "msg-prop", "defineProps({ ...common })")]
    );
}

#[test]
fn split_script_usage_scan_excludes_the_normalized_macro_call() {
    let sfc = r#"<script>
export default { computed: { used() { return this.used } } }
</script>

<script setup lang="ts">
defineProps<{ used: string; unused: string }>();
</script>

<template><div>hi</div></template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "unused", "unused: string")]
    );
}

#[test]
fn split_script_prop_honors_eslint_disable_forms() {
    for sfc in [
        r#"<script>
export const moduleValue = 1;
</script>

<script setup lang="ts">
// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>

<template><div>hi</div></template>
"#,
        r#"<script>
export const moduleValue = 1;
</script>

<script setup lang="ts">
defineProps<{ msg: string }>(); // eslint-disable-line vue/no-unused-properties
</script>

<template><div>hi</div></template>
"#,
        r#"<script>
export const moduleValue = 1;
</script>

<script setup lang="ts">
// eslint-disable vue/no-unused-properties
defineProps<{ msg: string }>();
// eslint-enable vue/no-unused-properties
</script>

<template><div>hi</div></template>
"#,
    ] {
        assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
    }
}

#[test]
fn sfc_absolute_prop_ignores_eslint_markers_in_strings() {
    let sfc = r#"<script setup lang="ts">
const marker = "eslint-disable-next-line vue/no-unused-properties"
defineProps<{ msg: string }>();
</script>

<template><div>hi</div></template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "msg", "msg: string")]
    );
}

#[test]
fn sfc_absolute_prop_honors_vize_expected() {
    for sfc in [
        r#"<script setup lang="ts">
// @vize:expected
defineProps<{ msg: string }>();
</script>

<template><div>hi</div></template>
"#,
        r#"<script setup lang="ts">
/* @vize:expected */
defineProps<{ msg: string }>();
</script>

<template><div>hi</div></template>
"#,
    ] {
        assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
    }
}

#[test]
fn sfc_absolute_prop_honors_vize_ignore_region_but_not_string_contents() {
    let ignored = r#"<script setup lang="ts">
// @vize:ignore-start
defineProps<{ msg: string }>();
// @vize:ignore-end
</script>

<template><div>hi</div></template>
"#;
    assert_eq!(owned(&lint_sfc(ignored)), Vec::new());

    let string = r#"<script setup lang="ts">
const marker = "@vize:expected"
defineProps<{ msg: string }>();
</script>

<template><div>hi</div></template>
"#;
    assert_eq!(
        owned(&lint_sfc(string)),
        vec![unused(string, "msg", "msg: string")]
    );
}

#[test]
fn sfc_absolute_prop_honors_every_vize_level() {
    for (level, severity) in [
        ("off", None),
        ("warn", Some(crate::diagnostic::Severity::Warning)),
        ("error", Some(crate::diagnostic::Severity::Error)),
    ] {
        let sfc = format!(
            r#"<script setup lang="ts">
// @vize:level({level})
defineProps<{{ msg: string }}>();
</script>

<template><div>hi</div></template>
"#
        );
        let result = lint_sfc(&sfc);
        let Some(severity) = severity else {
            assert_eq!(owned(&result), Vec::new(), "@vize:level({level})");
            continue;
        };
        let mut expected = unused(&sfc, "msg", "msg: string");
        expected.1 = severity;
        assert_eq!(owned(&result), vec![expected], "@vize:level({level})");
    }
}

#[test]
fn reports_only_the_prop_the_template_never_reads() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string; unused: number }>();
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "unused", "unused: number")]
    );
}

#[test]
fn reports_a_prop_of_a_runtime_array_declaration() {
    let sfc = r#"<script setup>
defineProps(['msg'])
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), vec![unused(sfc, "msg", "'msg'")]);
}

#[test]
fn reports_a_prop_of_a_runtime_object_declaration() {
    let sfc = r#"<script setup>
defineProps({ msg: String })
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "msg", "msg: String")]
    );
}

#[test]
fn reports_a_prop_the_destructuring_pattern_omits() {
    let sfc = r#"<script setup lang="ts">
const { msg } = defineProps<{ msg: string; other: number }>();
const upper = msg.toUpperCase();
</script>

<template>
  <div>{{ upper }}</div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "other", "other: number")]
    );
}

// --- A name that is not a reference must still report ---------------------

#[test]
fn reports_a_prop_named_only_in_an_html_comment_or_text() {
    // Neither is a compiled expression, so neither is a reference.
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <!-- msg -->
  <p title="msg">msg</p>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "msg", "msg: string")]
    );
}

#[test]
fn reports_a_prop_named_only_inside_a_v_pre_region() {
    // Vue never compiles a `v-pre` region, so `{{ msg }}` there is literal text.
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <pre v-pre>{{ msg }}</pre>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "msg", "msg: string")]
    );
}
