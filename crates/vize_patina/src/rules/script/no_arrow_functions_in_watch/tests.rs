use super::NoArrowFunctionsInWatch;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoArrowFunctionsInWatch));
    linter
}

#[test]
fn test_valid_function_shorthand_handler() {
    let source = r#"
export default {
  watch: {
    value(newValue, oldValue) {
      this.doSomething()
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_object_form_function_handler() {
    let source = r#"
export default {
  watch: {
    value: {
      handler(newValue) {
        this.doSomething()
      },
      deep: true
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_object_form_function_expression_handler() {
    let source = r#"
export default {
  watch: {
    value: {
      handler: function (newValue) {
        this.doSomething()
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_string_method_name_handler() {
    // A string handler name references a method; it is not an arrow function.
    let source = r#"
export default {
  methods: {
    onValueChange() {}
  },
  watch: {
    value: 'onValueChange'
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_no_watch_option() {
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
import { watch, ref } from 'vue'
const value = ref(0)
watch(value, () => {
  console.log('changed')
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_composition_api_watch_arrow_callback() {
    // The Composition API `watch(...)` callback has no `this` expectation, so
    // an arrow function there is correct even alongside an options object.
    let source = r#"
import { watch, ref } from 'vue'

const count = ref(0)
watch(count, (newValue) => {
  console.log(newValue)
})

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
fn test_invalid_shorthand_arrow_handler() {
    let source = r#"
export default {
  watch: {
    value: () => {
      this.doSomething()
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_object_form_arrow_handler() {
    let source = r#"
export default {
  watch: {
    value: {
      handler: () => {
        this.doSomething()
      },
      deep: true
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_concise_body_arrow_handler() {
    let source = r#"
export default {
  watch: {
    value: () => this.doSomething()
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_string_key_arrow_handler() {
    let source = r#"
export default {
  watch: {
    'nested.value': () => {}
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_define_component_arrow_handler() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  watch: {
    value: () => {
      this.doSomething()
    }
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_identifier_bound_options_arrow_handler() {
    let source = r#"
const component = {
  watch: {
    value: () => {}
  }
}

export default component
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_array_form_arrow_handler() {
    let source = r#"
export default {
  watch: {
    value: [
      () => {},
      'onValueChange'
    ]
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_multiple_arrow_handlers_reported() {
    let source = r#"
export default {
  watch: {
    a: () => {},
    b: {
      handler: () => {}
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_mixed_valid_and_invalid_handlers() {
    let source = r#"
export default {
  watch: {
    ok(newValue) {
      this.doSomething()
    },
    bad: () => {}
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_object_form_non_handler_arrow_ignored() {
    // Only the `handler` property is a watch callback; any other arrow-valued
    // property in the object form is not a handler and must not be flagged.
    let source = r#"
export default {
  watch: {
    value: {
      handler(newValue) {},
      deep: true,
      other: () => {}
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_spread_in_watch_object_ignored() {
    let source = r#"
export default {
  watch: {
    ...mapWatchers(['count'])
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}
