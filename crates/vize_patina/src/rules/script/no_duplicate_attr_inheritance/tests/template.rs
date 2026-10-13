//! Template half (#3414 C): a root `v-bind="$attrs"` with the default implicit.
//!
//! Because this half *creates* findings from template evidence, the over-match
//! probes are as load-bearing as the positive cases: each asserts the full
//! finding set, and the negative ones assert it is exactly empty.

use super::{findings, lint_sfc, none};
use crate::diagnostic::Severity;

const SPREAD_MESSAGE: &str = "Set \"inheritAttrs\" to false.";

/// The finding the `v-bind="$attrs"` written as `directive` produces.
fn duplicated(sfc: &str, directive: &str) -> (&'static str, Severity, u32, u32, &'static str) {
    let start = sfc.find(directive).expect("bind directive")
        + directive.find("$attrs").expect("forwarding reference");
    (
        "script/no-duplicate-attr-inheritance",
        Severity::Warning,
        start as u32,
        (start + "$attrs".len()) as u32,
        SPREAD_MESSAGE,
    )
}

// --- The recovered case: a root spread with the default left implicit ------

#[test]
fn reports_a_root_attrs_spread_issue_3414() {
    // Exact reproduction from #3414 C.
    let sfc = r#"<script setup lang="ts">
</script>

<template>
  <div v-bind="$attrs"></div>
</template>
"#;
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![duplicated(sfc, r#"v-bind="$attrs""#)]
    );
}

#[test]
fn reports_a_root_attrs_spread_with_an_options_api_block() {
    let sfc = r#"<script>
export default { name: 'Probe' };
</script>

<template>
  <div v-bind="$attrs"></div>
</template>
"#;
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![duplicated(sfc, r#"v-bind="$attrs""#)]
    );
}

#[test]
fn reports_a_root_attrs_spread_alongside_other_root_bindings() {
    let sfc = r#"<script setup lang="ts">
const id = 'x';
</script>

<template>
  <div :id="id" v-bind="$attrs" class="card"></div>
</template>
"#;
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![duplicated(sfc, r#"v-bind="$attrs""#)]
    );
}

// --- The documented opt-outs must stay silent ------------------------------

#[test]
fn ignores_a_root_attrs_spread_with_define_options_opt_out() {
    let sfc = r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>

<template>
  <div v-bind="$attrs"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_root_attrs_spread_with_an_options_api_opt_out() {
    let sfc = r#"<script>
export default { inheritAttrs: false };
</script>

<template>
  <div v-bind="$attrs"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn reports_the_forwarded_reference_when_inherit_attrs_is_explicitly_true() {
    let sfc = r#"<script setup lang="ts">
defineOptions({ inheritAttrs: true });
</script>

<template>
  <div v-bind="$attrs"></div>
</template>
"#;
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![duplicated(sfc, r#"v-bind="$attrs""#)]
    );
}

#[test]
fn ignores_a_root_attrs_spread_when_inherit_attrs_is_not_a_literal() {
    let sfc = r#"<script>
const inherit = false;
export default { inheritAttrs: inherit };
</script>

<template>
  <div v-bind="$attrs"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_root_attrs_spread_when_a_sibling_script_block_exists() {
    // Each block is linted separately, so the `inheritAttrs: false` in the
    // plain `<script>` is invisible to the `<script setup>` pass. Reporting
    // from either would be a false positive, and reporting from both would
    // double the diagnostic.
    let sfc = r#"<script>
export default { inheritAttrs: false };
</script>

<script setup lang="ts">
const id = 'x';
</script>

<template>
  <div :id="id" v-bind="$attrs"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- Over-match probes: none of these may manufacture a finding ------------

#[test]
fn ignores_an_attrs_spread_on_a_nested_element() {
    // Forwarding to an inner node is safe with explicit inheritance opt-out.
    let sfc = r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>

<template>
  <div><span v-bind="$attrs"></span></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_attrs_spread_in_a_multi_root_template() {
    // A fragment has no single element for the fallthrough attributes.
    let sfc = r#"<script setup lang="ts">
</script>

<template>
  <div v-bind="$attrs"></div>
  <p>second root</p>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_attrs_spread_inside_an_html_comment() {
    let sfc = r#"<script setup lang="ts">
</script>

<template>
  <!-- <div v-bind="$attrs"></div> -->
  <div></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_attrs_spread_in_a_text_node_or_plain_attribute() {
    let sfc = r#"<script setup lang="ts">
</script>

<template>
  <div title="v-bind=&quot;$attrs&quot;">v-bind="$attrs"</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_attrs_mention_inside_a_string_literal() {
    let sfc = r#"<script setup lang="ts">
</script>

<template>
  <div @click="console.log('v-bind=$attrs')"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_attrs_spread_inside_a_v_pre_region() {
    let sfc = r#"<script setup lang="ts">
</script>

<template>
  <pre v-pre><div v-bind="$attrs"></div></pre>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_identifier_that_merely_starts_with_attrs() {
    let sfc = r#"<script setup lang="ts">
const $attrsExtra = { id: 'x' };
</script>

<template>
  <div v-bind="$attrsExtra"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_argument_bind_of_attrs() {
    // `:id="$attrs"` binds one prop rather than spreading the fallthrough set.
    let sfc = r#"<script setup lang="ts">
</script>

<template>
  <div :id="$attrs"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn reports_a_real_attrs_reference_in_an_object_spread() {
    let sfc = r#"<script setup lang="ts">
</script>

<template>
  <div v-bind="{ ...$attrs }"></div>
</template>
"#;
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![duplicated(sfc, r#"v-bind="{ ...$attrs }""#)]
    );
}

#[test]
fn reports_inside_a_conditional_template_wrapper() {
    let sfc = r#"<script setup lang="ts">
const ok = true;
</script>

<template>
  <template v-if="ok"><div v-bind="$attrs"></div></template>
</template>
"#;
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![duplicated(sfc, r#"v-bind="$attrs""#)]
    );
}

#[test]
fn ignores_a_template_with_no_attrs_spread_at_all() {
    let sfc = r#"<script setup lang="ts">
</script>

<template>
  <div class="card"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn scriptless_disabled_rule_stays_disabled() {
    let result = crate::linter::Linter::new()
        .with_enabled_rules(Some(Vec::new()))
        .lint_sfc(
            "<template><div v-bind=\"$attrs\" /></template>",
            "Probe.vue",
        );
    assert_eq!(findings(&result), none());
}

#[test]
fn fatal_template_never_lends_partial_attrs_evidence() {
    let result = lint_sfc("<template><div v-bind=\"$attrs\"></></template>");
    assert!(result.error_count > 0);
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-duplicate-attr-inheritance")
    );
}

#[test]
fn escaped_vm_identifier_keeps_its_authored_full_span() {
    let sfc = r#"<script setup></script><template><div v-bind="$\u0061ttrs" /></template>"#;
    let token = r#"$\u0061ttrs"#;
    let start = sfc.find(token).unwrap() as u32;
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "script/no-duplicate-attr-inheritance",
            Severity::Warning,
            start,
            start + token.len() as u32,
            "Set \"inheritAttrs\" to false."
        )]
    );
}
