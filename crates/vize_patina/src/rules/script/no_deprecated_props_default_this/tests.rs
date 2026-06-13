use super::NoDeprecatedPropsDefaultThis;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoDeprecatedPropsDefaultThis));
    linter
}
#[test]
fn test_valid_default_uses_props_argument() {
    let source = r#"
export default {
  props: {
    size: {
      type: Number,
      default(props) {
        return props.baseSize
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_validator_uses_value_argument() {
    let source = r#"
export default {
  props: {
    value: {
      type: Number,
      validator(value) {
        return value > 0
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_literal_default() {
    let source = r#"
export default {
  props: {
    size: {
      type: Number,
      default: 10
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_shorthand_array_props() {
    let source = r#"
export default {
  props: ['foo', 'bar']
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_shorthand_type_props() {
    let source = r#"
export default {
  props: {
    foo: Number,
    bar: String
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
const size = ref(0)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_no_props_option() {
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
fn test_valid_this_in_nested_function_rebinds() {
    // A non-arrow nested function gets its own `this`.
    let source = r#"
export default {
  props: {
    size: {
      default() {
        return [1, 2].map(function () {
          return this.x
        })
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_this_outside_props() {
    // `this` in a method is fine; only props default/validator is restricted.
    let source = r#"
export default {
  props: {
    size: {
      type: Number,
      default: 1
    }
  },
  methods: {
    grow() {
      return this.size + 1
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_this_in_default_method() {
    let source = r#"
export default {
  props: {
    size: {
      type: Number,
      default() {
        return this.defaultSize
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_this_in_default_function_expression() {
    let source = r#"
export default {
  props: {
    size: {
      type: Number,
      default: function () {
        return this.defaultSize
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_this_in_default_arrow() {
    // An arrow has no own `this`; `this` here captures the module scope and is
    // not the component instance — still a migration bug.
    let source = r#"
export default {
  props: {
    size: {
      type: Number,
      default: () => this.defaultSize
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_this_in_validator() {
    let source = r#"
export default {
  props: {
    value: {
      type: Number,
      validator() {
        return this.value > 0
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_this_member_chain_in_default() {
    let source = r#"
export default {
  props: {
    size: {
      default() {
        return this.config.defaultSize
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_this_in_nested_arrow_captures() {
    // A nested arrow does NOT rebind `this`; it captures the default factory's
    // `this`, so the access is still flagged.
    let source = r#"
export default {
  props: {
    size: {
      default() {
        return [1, 2].map(() => this.factor)
      }
    }
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
    size: {
      type: Number,
      default() {
        return this.defaultSize
      }
    }
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
    size: {
      default() {
        return this.defaultSize
      }
    }
  }
}

export default component
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_multiple_this_in_both_default_and_validator() {
    let source = r#"
export default {
  props: {
    size: {
      default() {
        return this.base
      },
      validator() {
        return this.max > 0
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_invalid_string_key_default() {
    let source = r#"
export default {
  props: {
    size: {
      'default'() {
        return this.defaultSize
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_spread_in_prop_declaration_ignored() {
    let source = r#"
export default {
  props: {
    size: {
      ...sharedPropConfig
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}
