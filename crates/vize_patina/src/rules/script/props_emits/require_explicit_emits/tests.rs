use crate::rules::script::RequireExplicitEmits;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(RequireExplicitEmits));
    linter
}

// --- defineEmits: array form -----------------------------------------------

#[test]
fn test_valid_array_all_declared() {
    let source = r#"
const emit = defineEmits(['change', 'input'])
emit('change')
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_array_undeclared() {
    let source = r#"
const emit = defineEmits(['change'])
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_emit_inside_nested_scope() {
    let source = r#"
const emit = defineEmits(['change'])
function onClick() {
  emit('input')
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_custom_binding_name_tracked() {
    // The captured binding (not necessarily named `emit`) is what is matched.
    let source = r#"
const myEmit = defineEmits(['change'])
myEmit('change')
myEmit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

// --- defineEmits: object form ----------------------------------------------

#[test]
fn test_valid_object_all_declared() {
    let source = r#"
const emit = defineEmits({ change: null, input: (v) => true })
emit('change')
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_object_undeclared() {
    let source = r#"
const emit = defineEmits({ change: null })
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

// --- defineEmits: type form ------------------------------------------------

#[test]
fn test_valid_type_property_form() {
    let source = r#"
const emit = defineEmits<{ change: [id: number]; input: [value: string] }>()
emit('change')
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_type_property_form() {
    let source = r#"
const emit = defineEmits<{ change: [id: number] }>()
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_valid_type_call_signature_form() {
    let source = r#"
const emit = defineEmits<{
  (e: 'change', id: number): void
  (e: 'input', value: string): void
}>()
emit('change')
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

// --- Soundness: unresolvable declarations suppress all reports -------------

#[test]
fn test_no_define_emits_not_flagged() {
    // No emits declaration at all: emitting anything is fine (the component may
    // declare emits elsewhere, or intentionally not at all).
    let source = r#"
const foo = 1
function f() {}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_emit_without_define_emits_not_flagged() {
    // An emit call but no declaration in this script: cannot decide soundly.
    let source = r#"
const emit = useEmitter()
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_array_spread_is_unknown() {
    // A spread makes the declared set unknowable, so nothing is reported.
    let source = r#"
const names = ['change']
const emit = defineEmits([...names])
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_object_spread_is_unknown() {
    let source = r#"
const base = { change: null }
const emit = defineEmits({ ...base })
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_object_computed_key_is_unknown() {
    let source = r#"
const key = 'change'
const emit = defineEmits({ [key]: null })
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_type_reference_is_unknown() {
    // `defineEmits<Emits>()` — the named type is not resolvable here.
    let source = r#"
type Emits = { change: [id: number] }
const emit = defineEmits<Emits>()
emit('input')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_dynamic_emit_name_not_flagged() {
    // A non-string-literal emit name cannot be matched and is skipped.
    let source = r#"
const emit = defineEmits(['change'])
const name = 'input'
emit(name)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_empty_array_flags_any_emit() {
    // `defineEmits([])` is a known, empty declaration: any emit is undeclared.
    let source = r#"
const emit = defineEmits([])
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

// --- this.$emit: Options API -----------------------------------------------

#[test]
fn test_valid_options_api_array_this_emit() {
    let source = r#"
export default {
  emits: ['change', 'input'],
  methods: {
    onClick() {
      this.$emit('change')
      this.$emit('input')
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_options_api_array_this_emit() {
    let source = r#"
export default {
  emits: ['change'],
  methods: {
    onClick() {
      this.$emit('input')
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_valid_options_api_object_this_emit() {
    let source = r#"
export default {
  emits: { change: null },
  methods: {
    onClick() {
      this.$emit('change')
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_options_api_object_this_emit() {
    let source = r#"
export default {
  emits: { change: null },
  methods: {
    onClick() {
      this.$emit('input')
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_options_api_no_emits_option_not_flagged() {
    // No `emits` option: undeclared `this.$emit` is not our call to make.
    let source = r#"
export default {
  methods: {
    onClick() {
      this.$emit('change')
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_define_component_emits_this_emit() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  emits: ['change'],
  methods: {
    onClick() {
      this.$emit('input')
    }
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_this_emit_dynamic_name_not_flagged() {
    let source = r#"
export default {
  emits: ['change'],
  methods: {
    onClick(name) {
      this.$emit(name)
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_offset_is_applied() {
    let source = r#"
const emit = defineEmits(['change'])
emit('input')
"#;
    // The reported span is shifted by the block offset.
    let result = create_linter().lint(source, 100);
    assert_eq!(result.warning_count, 1);
    assert!(result.diagnostics[0].start >= 100);
}
