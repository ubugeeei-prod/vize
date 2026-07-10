use super::NoBooleanDefault;
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
