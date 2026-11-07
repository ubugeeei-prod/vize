use crate::linter::Linter;
use crate::rules::script::RequireDefaultProp;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(RequireDefaultProp));
    linter
}

#[test]
fn test_valid_default_present() {
    let source = r#"
export default {
  props: {
    name: { type: String, default: '' }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_required_prop() {
    let source = r#"
export default {
  props: {
    id: { type: Number, required: true }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_boolean_shorthand() {
    let source = r#"
export default {
  props: {
    enabled: Boolean
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_boolean_descriptor() {
    let source = r#"
export default {
  props: {
    enabled: { type: Boolean }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_array_form_is_ignored() {
    // Array-form props carry neither type nor default; require-prop-types owns
    // them. require-default-prop must not flag them.
    let source = r#"
export default {
  props: ['name', 'age']
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_shorthand_string() {
    let source = r#"
export default {
  props: {
    name: String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_descriptor_without_default() {
    let source = r#"
export default {
  props: {
    age: { type: Number }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_optional_explicit_required_false() {
    let source = r#"
export default {
  props: {
    age: { type: Number, required: false }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_props_runtime_object_invalid() {
    let source = r#"
const props = defineProps({
  name: { type: String }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_props_runtime_object_valid_with_default() {
    let source = r#"
const props = defineProps({
  name: { type: String, default: '' }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn options_listener_prop_requires_a_default() {
    let source = r#"
export default {
  props: {
    "onUpdate:modelValue": defineListenerProp<(value: string) => void>()
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn test_plain_type_only_define_props_has_no_default_surface() {
    let source = r#"
const props = defineProps<{ name?: string }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn generic_props_from_sibling_script_use_the_sibling_location() {
    let source = r#"<script lang="ts">
interface Props<T> {
  label?: T
  active?: boolean
}
</script>
<script setup lang="ts" generic="T extends string">
withDefaults(defineProps<Props<T>>(), { active: false })
</script>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["script/require-default-prop".into()]))
        .lint_sfc(source, "Probe.vue");
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "label?: T"
    );
}

#[test]
fn test_typed_destructure_requires_defaults_for_optional_non_boolean_props() {
    let source = r#"
interface Props {
  title?: string
  enabled?: boolean
  count: number
}
const { title, enabled, count } = defineProps<Props>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "title?: string"
    );
}

#[test]
fn typed_boolean_literal_props_do_not_require_defaults() {
    let source = r#"
const { enabled, disabled } = defineProps<{
  enabled?: true
  disabled?: false
}>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_typed_destructure_and_with_defaults_honor_declared_defaults() {
    let destructure = r#"
const { title = '' } = defineProps<{ title?: string }>()
"#;
    assert_eq!(create_linter().lint(destructure, 0).error_count, 0);

    let wrapper = r#"
const props = withDefaults(defineProps<{
  title?: string
  count?: number
}>(), { title: '' })
"#;
    let result = create_linter().lint(wrapper, 0);
    assert_eq!(result.error_count, 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &wrapper[diagnostic.start as usize..diagnostic.end as usize],
        "count?: number"
    );
}

#[test]
fn with_defaults_spread_does_not_claim_a_default_is_missing() {
    let source = r#"
interface Props {
  title?: string
  count?: number
}
const props = withDefaults(defineProps<Props>(), {
  ...importedDefaults,
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn unknown_defaults_spreads_do_not_hide_inline_intersection_props() {
    let source = r#"
type Props<T> = ImportedStatefulProps<T> & ImportedValidationProps<T> & {
  clearValue?: T
}
const props = withDefaults(defineProps<Props<string>>(), {
  ...statefulPropsDefaults,
  ...validationPropsDefaults,
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "clearValue?: T"
    );
}

#[test]
fn with_defaults_resolves_a_same_file_sibling_script_spread() {
    let source = r#"<script lang="ts">
export const defaults = {
  title: '',
  count: 0,
}
export interface Props {
  title?: string
  count?: number
  label?: string
}
</script>
<script setup lang="ts">
const props = withDefaults(defineProps<Props>(), {
  ...defaults,
})
</script>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["script/require-default-prop".into()]))
        .lint_sfc(source, "Probe.vue");
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "label?: string"
    );
}

#[test]
fn union_props_report_each_shared_missing_default_once() {
    let source = r#"
type Base = { label?: string; enabled?: boolean }
type Props = (Base & { mode?: 'a' }) | (Base & { mode?: 'b' })
const props = withDefaults(defineProps<Props>(), { mode: 'a' })
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "label?: string;"
    );
}

#[test]
fn test_runtime_custom_constructors_and_shorthand_are_ignored_like_upstream() {
    let source = r#"
const custom = {}
export default {
  props: {
    shorthand,
    call: makeType(),
    member: custom.Type,
    unknown: CustomType
  }
}
"#;
    assert_eq!(create_linter().lint(source, 0).error_count, 0);
}

#[test]
fn test_define_component_props() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    name: String
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_local_define_component_props() {
    let source = r#"
const Child = defineComponent({
  props: {
    name: String
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn test_boolean_array_type_is_ignored() {
    let source = r#"
export default {
  props: {
    enabled: { type: [Boolean] }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_multi_type_array_with_boolean_still_needs_default() {
    // A union type that includes Boolean but also another type is not a pure
    // Boolean prop and should declare a default.
    let source = r#"
export default {
  props: {
    value: { type: [Boolean, String] }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_multiple_props_reported() {
    let source = r#"
export default {
  props: {
    name: String,
    age: { type: Number },
    enabled: Boolean,
    id: { type: Number, required: true }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_no_props_option() {
    let source = r#"
export default {
  data() {
    return { count: 0 }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}
