use crate::rules::script::RequirePropTypes;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(RequirePropTypes));
    linter
}

#[test]
fn test_valid_shorthand_type() {
    let source = r#"
export default {
  props: {
    status: String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_descriptor_type() {
    let source = r#"
export default {
  props: {
    status: { type: Number, default: 0 }
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
    status: [String, Number]
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_array_form() {
    let source = r#"
export default {
  props: ['status']
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_valid_null_value() {
    let source = r#"
export default {
  props: {
    status: null
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_empty_descriptor() {
    let source = r#"
export default {
  props: {
    status: {}
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_descriptor_without_type() {
    let source = r#"
export default {
  props: {
    status: { required: true }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn validator_counts_as_a_type_contract() {
    let source = r#"
export default {
  props: {
    status: { validator: value => value !== null }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_define_props_array_form() {
    let source = r#"
const props = defineProps(['status', 'kind'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_define_props_object_typed() {
    let source = r#"
const props = defineProps({
  status: String
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_define_props_type_based_ignored() {
    let source = r#"
const props = defineProps<{ status: string }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_array_and_object_mixed_counts() {
    let source = r#"
export default {
  props: {
    typed: String,
    bad1: null,
    bad2: {}
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
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

#[test]
fn test_validator_function_is_typed() {
    // A function value is treated as the type position (a validator/PropType
    // factory); not flagged.
    let source = r#"
export default {
  props: {
    custom: makeType()
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn spread_descriptor_does_not_prove_a_type() {
    let source = r#"
export default {
  props: {
    offset: { ...shared.offset, default: () => [2, 0] }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn plain_modules_need_define_component() {
    use crate::{LintPreset, Linter};
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let options = "export default {\n  props: { contain: { hidden: true } }\n}\n";
    let plain = linter.lint_script(options, "api-options.ts");
    assert!(
        plain
            .diagnostics
            .iter()
            .all(|d| d.rule_name != "script/require-prop-types")
    );
    let component = linter.lint_script(
        "import { defineComponent } from 'vue'\nexport default defineComponent({ props: ['icon'] })\n",
        "Icon.ts",
    );
    assert!(
        component
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "script/require-prop-types")
    );
}

#[test]
fn define_model_without_runtime_or_type_argument_requires_a_type() {
    let source = "const show = defineModel('show', { default: false })";
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "{ default: false }"
    );
}

#[test]
fn typed_define_model_and_runtime_type_are_valid() {
    for source in [
        "const show = defineModel<boolean>('show', { default: false })",
        "const show = defineModel('show', { type: Boolean, default: false })",
    ] {
        assert_eq!(create_linter().lint(source, 0).error_count, 0, "{source}");
    }
}
