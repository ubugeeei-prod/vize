use super::ComponentOptionsNameCasing;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(ComponentOptionsNameCasing));
    linter
}

#[test]
fn test_valid_pascal_case_name() {
    let source = r#"
export default {
  name: 'MyComponent'
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_single_word_pascal_case() {
    let source = r#"
export default {
  name: 'Button'
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_pascal_case_with_digits() {
    let source = r#"
export default {
  name: 'Heading2'
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_no_name_option() {
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
fn test_valid_no_options_object() {
    let source = r#"
import { ref } from 'vue'
const count = ref(0)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_non_string_literal_name_skipped() {
    // A computed / identifier-valued `name` is not a string literal and is
    // intentionally skipped.
    let source = r#"
const componentName = 'whatever'
export default {
  name: componentName
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_computed_name_key_skipped() {
    let source = r#"
const key = 'name'
export default {
  [key]: 'my-component'
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_kebab_case_name() {
    let source = r#"
export default {
  name: 'my-component'
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_camel_case_name() {
    let source = r#"
export default {
  name: 'myComponent'
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_snake_case_name() {
    let source = r#"
export default {
  name: 'my_component'
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_lowercase_single_word() {
    let source = r#"
export default {
  name: 'button'
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_define_component_kebab_case() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'my-component'
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_identifier_bound_options_kebab_case() {
    let source = r#"
const component = {
  name: 'my-component'
}

export default component
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_string_key_name_camel_case() {
    let source = r#"
export default {
  'name': 'myComponent'
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_ts_satisfies_wrapper_kebab_case() {
    let source = r#"
import { defineComponent } from 'vue'

export default {
  name: 'my-component'
} satisfies { name: string }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_define_options_kebab_case() {
    let source = r#"
defineOptions({
  name: 'my-component'
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_valid_define_options_pascal_case() {
    let source = r#"
defineOptions({
  name: 'MyComponent'
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_define_options_non_string_name_skipped() {
    let source = r#"
const componentName = 'whatever'
defineOptions({
  name: componentName
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}
