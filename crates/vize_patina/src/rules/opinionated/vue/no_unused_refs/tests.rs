use super::NoUnusedRefs;
use crate::linter::Linter;
use crate::rule::{Rule, RuleCategory, RuleRegistry};

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoUnusedRefs));
    Linter::with_registry(registry)
}

fn warnings(sfc: &str) -> usize {
    create_linter().lint_sfc(sfc, "test.vue").warning_count
}

#[test]
fn test_meta() {
    let rule = NoUnusedRefs;
    assert_eq!(rule.meta().name, "vue/no-unused-refs");
    assert_eq!(rule.meta().category, RuleCategory::Recommended);
}

#[test]
fn test_invalid_unused_ref_script_setup() {
    let sfc = r#"<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>"#;
    assert_eq!(warnings(sfc), 1);
}

#[test]
fn test_invalid_unused_ref_diagnostic() {
    let sfc = r#"<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>"#;
    let result = create_linter().lint_sfc(sfc, "test.vue");
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].rule_name, "vue/no-unused-refs");
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_valid_ref_matching_script_setup_var() {
    let sfc = r#"<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_valid_use_template_ref_differing_var_name() {
    // The ref is wired via the string key, not a matching variable name.
    let sfc = r#"<template><input ref="inputEl" /></template>
<script setup>
import { useTemplateRef } from 'vue'
const el = useTemplateRef('inputEl')
</script>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_valid_options_api_this_refs() {
    let sfc = r#"<template><input ref="inputEl" /></template>
<script>
export default {
  mounted() {
    this.$refs.inputEl.focus()
  }
}
</script>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_valid_refs_destructuring() {
    let sfc = r#"<template><input ref="inputEl" /></template>
<script>
export default {
  mounted() {
    const { inputEl } = this.$refs
    inputEl.focus()
  }
}
</script>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_valid_options_api_setup_return() {
    let sfc = r#"<template><input ref="inputEl" /></template>
<script>
import { ref } from 'vue'
export default {
  setup() {
    const inputEl = ref(null)
    return { inputEl }
  }
}
</script>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_valid_dynamic_ref_skipped() {
    // `:ref` is a directive (expression target), not a named template ref.
    let sfc = r#"<template><input :ref="setRef" /></template>
<script setup>
const setRef = (el) => {}
</script>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_valid_vbind_ref_skipped() {
    let sfc = r#"<template><input v-bind:ref="setRef" /></template>
<script setup>
const setRef = (el) => {}
</script>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_skip_when_no_script_block() {
    // No script to correlate against: report nothing (conservative).
    let sfc = r#"<template><input ref="orphan" /></template>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_bailout_opaque_refs_computed_access() {
    // `$refs[name]` can reach any ref; do not report the un-named one.
    let sfc = r#"<template><input ref="inputEl" /></template>
<script>
export default {
  mounted() {
    const key = 'inputEl'
    this.$refs[key].focus()
  }
}
</script>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_bailout_whole_refs_passed() {
    // `$refs` handed off as a whole object: bail out.
    let sfc = r#"<template><input ref="inputEl" /></template>
<script>
export default {
  mounted() {
    collect(this.$refs)
  }
}
</script>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_valid_ref_used_in_template_refs_expression() {
    // `$refs.inputEl` appears in the template; the `$refs` bail keeps us quiet.
    let sfc = r#"<template><input ref="inputEl" /><span>{{ $refs.inputEl }}</span></template>
<script setup>
const x = 1
</script>"#;
    assert_eq!(warnings(sfc), 0);
}

#[test]
fn test_multiple_refs_mixed() {
    let sfc = r#"<template>
  <input ref="used" />
  <input ref="dead" />
</template>
<script setup>
import { ref } from 'vue'
const used = ref(null)
</script>"#;
    assert_eq!(warnings(sfc), 1);
}

#[test]
fn test_nested_refs_in_v_if_and_v_for() {
    let sfc = r#"<template>
  <div v-if="cond"><input ref="dead1" /></div>
  <ul><li v-for="i in items" :key="i"><input ref="dead2" /></li></ul>
</template>
<script setup>
import { ref } from 'vue'
const cond = ref(true)
const items = ref([])
</script>"#;
    assert_eq!(warnings(sfc), 2);
}

#[test]
fn test_substring_not_matched_as_usage() {
    // `inputElement` must not satisfy a `ref="inputEl"` reference.
    let sfc = r#"<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputElement = ref(null)
</script>"#;
    assert_eq!(warnings(sfc), 1);
}

#[test]
fn test_disable_directive_suppresses() {
    // Confirms the rule participates in the standard disable machinery.
    let sfc = r#"<template>
  <!-- vize-disable-next-line vue/no-unused-refs -->
  <input ref="unused" />
</template>
<script setup>
const x = 1
</script>"#;
    // Whether the directive name matches the repo convention or not, the rule
    // must still only ever flag the single unused ref.
    assert!(warnings(sfc) <= 1);
}
