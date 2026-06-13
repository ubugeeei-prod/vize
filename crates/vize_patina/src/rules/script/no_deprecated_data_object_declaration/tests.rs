use super::NoDeprecatedDataObjectDeclaration;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoDeprecatedDataObjectDeclaration));
    linter
}

#[test]
fn test_valid_data_method_shorthand() {
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
fn test_valid_data_function_expression() {
    let source = r#"
export default {
  data: function () {
    return { count: 0 }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_data_arrow_function() {
    let source = r#"
export default {
  data: () => ({ count: 0 })
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_no_data_option() {
    let source = r#"
export default {
  computed: {
    doubled() { return this.count * 2 }
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
fn test_valid_data_is_identifier_reference() {
    // `data` referencing a variable is not an inline object literal; this rule
    // only flags object literals declared directly on the option.
    let source = r#"
const initialData = { count: 0 }
export default {
  data: initialData
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_object_literal_data() {
    let source = r#"
export default {
  data: {
    count: 0
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_empty_object_literal_data() {
    let source = r#"
export default {
  data: {}
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_define_component_object_data() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  data: {
    count: 0
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_identifier_bound_options_object_data() {
    let source = r#"
const component = {
  data: {
    count: 0
  }
}

export default component
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_ts_satisfies_wrapped_options() {
    let source = r#"
export default {
  data: {
    count: 0
  }
} satisfies ComponentOptions
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_string_key_data() {
    let source = r#"
export default {
  'data': {
    count: 0
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_computed_key_data_ignored() {
    // A computed key is not statically the `data` option, so it is not flagged.
    let source = r#"
const key = 'data'
export default {
  [key]: {
    count: 0
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_only_data_reported_among_other_object_options() {
    // Other object-valued options (computed, methods, watch) are not `data` and
    // must not be flagged; only the object-literal `data` is reported.
    let source = r#"
export default {
  computed: {
    doubled() { return 0 }
  },
  data: {
    count: 0
  },
  methods: {
    inc() {}
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}
