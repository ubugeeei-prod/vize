use crate::rules::script::NoReservedProps;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoReservedProps));
    linter
}

#[test]
fn test_valid_normal_props() {
    let source = r#"
export default {
  props: {
    name: String,
    refValue: Number
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_ref() {
    let source = r#"
export default {
  props: {
    ref: String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_key() {
    let source = r#"
export default {
  props: {
    key: String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_is() {
    let source = r#"
export default {
  props: {
    is: String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_ref_for_and_ref_key() {
    let source = r#"
export default {
  props: {
    ref_for: Boolean,
    ref_key: String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_invalid_dollar_prefixed() {
    let source = r#"
export default {
  props: {
    $foo: Number
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_array_form_reserved() {
    let source = r#"
export default {
  props: ['key', 'name']
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_props_object_reserved() {
    let source = r#"
const props = defineProps({
  ref: String
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_props_array_reserved() {
    let source = r#"
const props = defineProps(['is'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_component_reserved() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    key: String
  }
})
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
fn test_string_literal_key_reserved() {
    let source = r#"
export default {
  props: {
    "ref": String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}
