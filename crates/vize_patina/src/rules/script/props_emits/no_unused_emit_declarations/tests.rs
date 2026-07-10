use crate::rules::script::NoUnusedEmitDeclarations;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoUnusedEmitDeclarations));
    linter
}

#[test]
fn test_valid_all_emitted_array() {
    let source = r#"
const emit = defineEmits(['change', 'update'])
emit('change')
emit('update')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_one_unused_array() {
    let source = r#"
const emit = defineEmits(['change', 'unused'])
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_none_emitted() {
    let source = r#"
const emit = defineEmits(['change', 'update'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_valid_emitted_inside_function() {
    let source = r#"
const emit = defineEmits(['change'])
function onClick() {
  emit('change')
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_unassigned_define_emits_not_tracked() {
    // Without a binding we cannot track usage, so nothing is reported even
    // though no emit call exists.
    let source = r#"
defineEmits(['change', 'update'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_type_based_property_form() {
    let source = r#"
const emit = defineEmits<{ change: [id: number]; unused: [] }>()
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_type_based_call_signature_form() {
    let source = r#"
const emit = defineEmits<{
  (e: 'change', id: number): void
  (e: 'unused'): void
}>()
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_type_based_all_used() {
    let source = r#"
const emit = defineEmits<{ change: [id: number] }>()
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_runtime_object_form() {
    let source = r#"
const emit = defineEmits({
  change: null,
  unused: null
})
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_custom_emit_binding_name() {
    let source = r#"
const myEmit = defineEmits(['change', 'unused'])
myEmit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_dynamic_emit_argument_does_not_mark() {
    // A non-string-literal argument can't be matched to a declared name, so the
    // declared events remain unused.
    let source = r#"
const emit = defineEmits(['change'])
const name = 'change'
emit(name)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_no_define_emits() {
    let source = r#"
const props = defineProps<{ name: string }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}
