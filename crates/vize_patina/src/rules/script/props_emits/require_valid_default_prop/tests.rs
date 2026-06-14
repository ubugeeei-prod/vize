use crate::rules::script::RequireValidDefaultProp;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(RequireValidDefaultProp));
    linter
}

#[test]
fn test_valid_number_default() {
    let source = r#"
export default {
  props: {
    count: { type: Number, default: 0 }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_string_default() {
    let source = r#"
export default {
  props: {
    label: { type: String, default: 'hi' }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_boolean_default() {
    let source = r#"
export default {
  props: {
    enabled: { type: Boolean, default: false }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_object_factory() {
    let source = r#"
export default {
  props: {
    config: { type: Object, default: () => ({}) }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_array_factory() {
    let source = r#"
export default {
  props: {
    items: { type: Array, default: () => [] }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_array_factory_function_expression() {
    let source = r#"
export default {
  props: {
    items: { type: Array, default: function () { return [] } }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_union_type_matching_default() {
    let source = r#"
export default {
  props: {
    value: { type: [String, Number], default: '' }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_no_default() {
    // No `default` to validate.
    let source = r#"
export default {
  props: {
    count: { type: Number }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_shorthand_type_is_ignored() {
    // Shorthand has no default to validate.
    let source = r#"
export default {
  props: {
    count: Number
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_array_form_is_ignored() {
    let source = r#"
export default {
  props: ['count', 'label']
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_non_native_type_is_ignored() {
    // An imported/custom type cannot be validated against a literal.
    let source = r#"
export default {
  props: {
    when: { type: Date, default: makeDate() }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_identifier_default_is_ignored() {
    // The default's kind cannot be determined statically.
    let source = r#"
export default {
  props: {
    count: { type: Number, default: FALLBACK }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_function_default_for_scalar() {
    // A function default is a factory Vue calls; accepted for any type.
    let source = r#"
export default {
  props: {
    count: { type: Number, default: () => 0 }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_string_default_for_number() {
    let source = r#"
export default {
  props: {
    count: { type: Number, default: '0' }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_number_default_for_boolean() {
    let source = r#"
export default {
  props: {
    enabled: { type: Boolean, default: 1 }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_object_literal_default() {
    let source = r#"
export default {
  props: {
    config: { type: Object, default: {} }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_array_literal_default() {
    let source = r#"
export default {
  props: {
    items: { type: Array, default: [] }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_boolean_default_for_string() {
    let source = r#"
export default {
  props: {
    label: { type: String, default: true }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_union_type_no_match() {
    // Default is Boolean but neither String nor Number is satisfied.
    let source = r#"
export default {
  props: {
    value: { type: [String, Number], default: true }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_props_runtime_invalid() {
    let source = r#"
const props = defineProps({
  count: { type: Number, default: '0' }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_props_runtime_valid() {
    let source = r#"
const props = defineProps({
  items: { type: Array, default: () => [] }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_define_props_type_based_is_ignored() {
    let source = r#"
const props = defineProps<{ count?: number }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_define_component_invalid() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    config: { type: Object, default: {} }
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_multiple_invalid_props_reported() {
    let source = r#"
export default {
  props: {
    count: { type: Number, default: '0' },
    items: { type: Array, default: [] },
    label: { type: String, default: '' },
    enabled: { type: Boolean, default: true }
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
