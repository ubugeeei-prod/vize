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
fn test_define_props_type_based_is_ignored() {
    // No runtime descriptor exists, so nothing is checked.
    let source = r#"
const props = defineProps<{ name?: string }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
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
