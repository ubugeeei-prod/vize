use crate::rules::script::ReturnInEmitsValidator;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(ReturnInEmitsValidator));
    linter
}

#[test]
fn test_valid_method_returns() {
    let source = r#"
export default {
  emits: {
    submit(payload) {
      return !!payload
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_arrow_concise_body() {
    let source = r#"
export default {
  emits: {
    submit: (payload) => !!payload
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_arrow_block_returns() {
    let source = r#"
export default {
  emits: {
    submit: (payload) => {
      return payload != null
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_null_validator() {
    // `submit: null` means "no validation"; not a function, never flagged.
    let source = r#"
export default {
  emits: {
    submit: null
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_array_emits() {
    // Array form has no validators.
    let source = r#"
export default {
  emits: ['submit', 'cancel']
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_method_no_return() {
    let source = r#"
export default {
  emits: {
    submit(payload) {
      if (!payload) {
        console.log('bad')
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
fn test_invalid_arrow_block_no_return() {
    let source = r#"
export default {
  emits: {
    submit: (payload) => {
      console.log(payload)
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_bare_return() {
    // `return;` yields no value.
    let source = r#"
export default {
  emits: {
    submit(payload) {
      if (!payload) return
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_nested_function_return_does_not_count() {
    let source = r#"
export default {
  emits: {
    submit(payload) {
      const helper = () => { return true }
      helper()
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_component_emits() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  emits: {
    submit(payload) {
      console.log(payload)
    }
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_multiple_validators() {
    let source = r#"
export default {
  emits: {
    good(p) { return !!p },
    bad(p) { console.log(p) }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_no_emits_option() {
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
