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

/// The reference rule's other forbidden node types, in all three positions.
#[test]
fn test_invalid_non_string_type_expressions() {
    for (source, expected) in [
        (
            "export default { props: { a: Number | String } }",
            vec!["Number | String"],
        ),
        (
            "export default { props: { a: { type: String | Object, default: '' } } }",
            vec!["String | Object"],
        ),
        (
            "export default { props: { a: [1, 2, 3] } }",
            vec!["1", "2", "3"],
        ),
        (
            "export default { props: { a: 1, b: true } }",
            vec!["1", "true"],
        ),
        (
            "export default { props: { a: `String`, b: { type: [Number, `Boolean`] } } }",
            vec!["`String`", "`Boolean`"],
        ),
        (
            "export default { props: { a: { type: count++ } } }",
            vec!["count++"],
        ),
        (
            "export default defineComponent({ props: { a: Foo + '' } })",
            vec!["Foo + ''"],
        ),
    ] {
        let result = create_linter().lint(source, 0);
        let reported: Vec<&str> = result
            .diagnostics
            .iter()
            .map(|diagnostic| &source[diagnostic.start as usize..diagnostic.end as usize])
            .collect();
        assert_eq!(reported, expected, "{source}");
        assert_eq!(result.error_count, expected.len(), "{source}");
        assert!(result.diagnostics.iter().all(|diagnostic| {
            diagnostic.rule_name == "script/require-prop-type-constructor"
                && diagnostic.message.ends_with("should be a constructor.")
        }));
    }
}

#[test]
fn test_valid_null_logical_and_computed_types() {
    for source in [
        "export default { props: { a: null, b: { type: null } } }",
        "export default { props: { a: [String, null] } }",
        "export default { props: { a: Foo || String, b: { type: types.Custom } } }",
        "export default { props: { a: { type: () => String }, b: Object as PropType<Foo> } }",
        "export default { props: { a: { type: Number, default: 1 }, b: { type: Boolean, default: true } } }",
        "export default { props: { a: { validator: (value) => value > 1 } } }",
    ] {
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0, "{source}: {:?}", result.diagnostics);
    }
}
