use super::DefineMacrosOrder;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(DefineMacrosOrder));
    linter
}

// --- Valid: macros in canonical order ---

#[test]
fn test_valid_full_canonical_order() {
    let source = r#"
defineOptions({ name: 'MyComponent' })
const model = defineModel<string>()
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
defineSlots<{ default(props: {}): any }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_subset_in_order() {
    // Not every macro must be present; the present ones just need to be ordered.
    let source = r#"
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_single_macro() {
    let source = "const props = defineProps<{ count: number }>()\n";
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_with_defaults_wrapper_in_order() {
    // `withDefaults(defineProps(...))` is still a defineProps macro statement.
    let source = r#"
const model = defineModel<string>()
const props = withDefaults(defineProps<{ count?: number }>(), { count: 0 })
const emit = defineEmits<{ change: [value: string] }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_imports_and_types_before_macros() {
    // Imports and type aliases are exempt and may precede the macros.
    let source = r#"
import { ref } from 'vue'
type Props = { count: number }
const props = defineProps<Props>()
const emit = defineEmits<{ change: [] }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_type_alias_between_macros_is_ignored() {
    // A type-only statement between macros is not a runtime boundary.
    let source = r#"
const props = defineProps<{ count: number }>()
type Emits = { change: [] }
const emit = defineEmits<Emits>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_no_macros_at_all() {
    let source = r#"
import { ref } from 'vue'
const count = ref(0)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_other_statements_after_macros() {
    // Runtime code after the whole macro block is fine.
    let source = r#"
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [] }>()
const doubled = props.count * 2
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

// --- Invalid: macros out of canonical order ---

#[test]
fn test_invalid_props_before_model() {
    let source = r#"
const props = defineProps<{ count: number }>()
const model = defineModel<string>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_emits_before_props() {
    let source = r#"
const emit = defineEmits<{ change: [] }>()
const props = defineProps<{ count: number }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_options_after_props() {
    let source = r#"
const props = defineProps<{ count: number }>()
defineOptions({ name: 'MyComponent' })
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_slots_before_emits() {
    let source = r#"
defineSlots<{ default(props: {}): any }>()
const emit = defineEmits<{ change: [] }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_with_defaults_props_out_of_order() {
    // The `withDefaults` wrapper is unwrapped to a defineProps macro, which here
    // appears after defineEmits.
    let source = r#"
const emit = defineEmits<{ change: [] }>()
const props = withDefaults(defineProps<{ count?: number }>(), { count: 0 })
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_fully_reversed_order() {
    // defineSlots, defineEmits, defineProps, defineModel, defineOptions — every
    // macro after the first is out of order, so four are reported.
    let source = r#"
defineSlots<{ default(props: {}): any }>()
const emit = defineEmits<{ change: [] }>()
const props = defineProps<{ count: number }>()
const model = defineModel<string>()
defineOptions({ name: 'MyComponent' })
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 4);
}

// --- Invalid: macro after a non-macro runtime statement ---

#[test]
fn test_invalid_macro_after_runtime_statement() {
    let source = r#"
const value = ref(0)
const props = defineProps<{ count: number }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_macro_after_function_declaration() {
    let source = r#"
const props = defineProps<{ count: number }>()
function helper() {}
const emit = defineEmits<{ change: [] }>()
"#;
    let result = create_linter().lint(source, 0);
    // Only the trailing emit is after the function statement; order is otherwise
    // canonical, so exactly one diagnostic is produced.
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_both_out_of_order_and_after_statement() {
    // defineModel is after a runtime statement *and* the macros are out of order
    // relative to each other; both problems are reported.
    let source = r#"
const props = defineProps<{ count: number }>()
const value = ref(0)
const model = defineModel<string>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}

// --- Edge cases ---

#[test]
fn test_valid_macro_like_call_not_a_macro() {
    // A non-macro call between macros is a runtime statement, but here it comes
    // after the whole macro block, so it is fine.
    let source = r#"
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [] }>()
useSomething()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_macro_name_only_in_string() {
    // The macro names appearing only inside a string literal must not trip the
    // AST-based detection.
    let source = r#"
const label = 'defineProps then defineEmits'
const order = 'defineOptions'
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}
