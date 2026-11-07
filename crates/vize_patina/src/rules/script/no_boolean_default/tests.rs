use super::NoBooleanDefault;
use crate::linter::Linter;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoBooleanDefault));
    linter
}

#[test]
fn test_valid_boolean_without_default() {
    let source = r#"
export default {
  props: {
    disabled: { type: Boolean }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_boolean_shorthand() {
    let source = r#"
export default {
  props: {
    disabled: Boolean
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_union_type_with_default() {
    let source = r#"
export default {
  props: {
    value: { type: [Boolean, String], default: '' }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_non_boolean_with_default() {
    let source = r#"
export default {
  props: {
    count: { type: Number, default: 0 }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_array_form_props() {
    let source = r#"
export default {
  props: ['disabled', 'checked']
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_no_options_object() {
    let source = r#"
import { ref } from 'vue'
const open = ref(false)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_type_as_boolean_in_array_only() {
    let source = r#"
export default {
  props: {
    value: { type: [String, Boolean], default: 'x' }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_no_type_property() {
    let source = r#"
export default {
  props: {
    value: { default: false }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_boolean_default_true() {
    let source = r#"
export default {
  props: {
    disabled: { type: Boolean, default: true }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_boolean_default_false() {
    let source = r#"
export default {
  props: {
    checked: { type: Boolean, default: false }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_define_component() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    active: { type: Boolean, default: true }
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_nested_define_components() {
    let source = r#"
import { defineComponent } from 'vue'

const First = defineComponent({
  props: { active: { type: Boolean, default: true } }
})
const Second = defineComponent({
  props: { disabled: { type: Boolean, default: false } }
})
export default { components: { First, Second } }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_invalid_runtime_define_props() {
    let source = r#"
const props = defineProps({
  active: { type: Boolean, default: true }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    let start = source.find("true").unwrap() as u32;
    assert_eq!(result.diagnostics[0].start, start);
    assert_eq!(result.diagnostics[0].end, start + 4);
}

#[test]
fn test_invalid_runtime_define_props_destructure_default() {
    let source = r#"
const { active = false } = defineProps({ active: Boolean })
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    let start = source.find("false").unwrap() as u32;
    assert_eq!(
        (result.diagnostics[0].start, result.diagnostics[0].end),
        (start, start + 5)
    );
}

#[test]
fn test_invalid_typed_boolean_defaults() {
    let destructure = r#"
interface Props { active?: boolean; label?: string }
const { active = false, label = '' } = defineProps<Props>()
"#;
    let result = create_linter().lint(destructure, 0);
    assert_eq!(result.warning_count, 1);
    let start = destructure.find("false").unwrap() as u32;
    assert_eq!(
        (result.diagnostics[0].start, result.diagnostics[0].end),
        (start, start + 5)
    );

    let wrapper = r#"
const props = withDefaults(defineProps<{ active?: boolean; count?: number }>(), {
  active: true,
  count: 0
})
"#;
    let result = create_linter().lint(wrapper, 0);
    assert_eq!(result.warning_count, 1);
    let start = wrapper.find("true").unwrap() as u32;
    assert_eq!(
        (result.diagnostics[0].start, result.diagnostics[0].end),
        (start, start + 4)
    );
}

#[test]
fn typed_boolean_literal_union_default_is_reported_once() {
    let source = r#"
interface Enabled { enabled?: true }
interface Disabled { enabled?: false }
type Props = Enabled | Disabled
const props = withDefaults(defineProps<Props>(), { enabled: false })
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn generic_boolean_prop_from_sibling_script_is_resolved() {
    let source = r#"<script lang="ts">
interface Props<T> {
  value?: T
  active?: boolean
}
</script>
<script setup lang="ts" generic="T extends string">
withDefaults(defineProps<Props<T>>(), { active: false })
</script>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["script/no-boolean-default".into()]))
        .lint_sfc(source, "Probe.vue");
    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "false"
    );
}

#[test]
fn test_invalid_identifier_bound_options() {
    let source = r#"
const component = {
  props: {
    open: { default: false, type: Boolean }
  }
}

export default component
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_multiple_boolean_defaults_reported() {
    let source = r#"
export default {
  props: {
    a: { type: Boolean, default: true },
    b: { type: Boolean, default: false },
    c: { type: Boolean }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_string_key_default_reported() {
    let source = r#"
export default {
  props: {
    disabled: { type: Boolean, "default": true }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_spread_in_props_is_ignored() {
    let source = r#"
export default {
  props: {
    ...baseProps,
    disabled: { type: Boolean }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}
