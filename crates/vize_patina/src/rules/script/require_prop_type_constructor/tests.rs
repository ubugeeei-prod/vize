use super::RequirePropTypeConstructor;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(RequirePropTypeConstructor));
    linter
}

#[test]
fn test_valid_constructor_types() {
    let source = r#"
export default {
  props: {
    name: String,
    age: { type: Number, required: true },
    active: Boolean
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_array_of_constructors() {
    let source = r#"
export default {
  props: {
    id: { type: [String, Number] }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_array_form_props() {
    // Array-form props declare only names, never types.
    let source = r#"
export default {
  props: ['name', 'age']
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_no_options_object() {
    let source = r#"
import { ref } from 'vue'
const count = ref(0)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_imported_type_identifier() {
    // A non-string, non-array type expression (e.g. an imported validator or
    // PropType cast target) must not be flagged.
    let source = r#"
export default {
  props: {
    cb: { type: Function },
    custom: { type: MyType }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_shorthand_string_type() {
    let source = r#"
export default {
  props: {
    name: "String"
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_explicit_type_string() {
    let source = r#"
export default {
  props: {
    age: { type: "Number", required: true }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_array_of_string_types() {
    let source = r#"
export default {
  props: {
    id: { type: ["String", "Number"] }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_invalid_mixed_array_types() {
    // Only the string-literal entry is flagged; the constructor is fine.
    let source = r#"
export default {
  props: {
    id: { type: [String, "Number"] }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_define_component() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    title: "String"
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_identifier_bound_options() {
    let source = r#"
const component = {
  props: {
    title: { type: "String" }
  }
}

export default component
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_multiple_invalid_props_reported() {
    let source = r#"
export default {
  props: {
    name: "String",
    age: { type: "Number" }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_valid_string_default_value_is_not_a_type() {
    // A string `default` is a value, not a type; only `type` (or the
    // shorthand type position) is checked.
    let source = r#"
export default {
  props: {
    label: { type: String, default: "hello" }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}
