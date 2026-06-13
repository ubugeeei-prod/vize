use super::ReturnInComputedProperty;
use crate::rules::script::{ScriptLintResult, ScriptLinter};

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(ReturnInComputedProperty));
    linter.lint(source, 0)
}

// --- Composition API ---

#[test]
fn test_valid_composition_returns() {
    // Concise arrow, block arrow with return, function expression with return.
    assert_eq!(
        lint("const d = computed(() => count.value * 2)").error_count,
        0
    );
    assert_eq!(
        lint("const d = computed(() => { return count.value })").error_count,
        0
    );
    assert_eq!(
        lint("const d = computed(function () { return count.value })").error_count,
        0
    );
}

#[test]
fn test_invalid_block_arrow_without_return() {
    let result = lint("const d = computed(() => { const v = count.value })");
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_function_expression_without_return() {
    assert_eq!(
        lint("const d = computed(function () { const v = count.value })").error_count,
        1
    );
}

#[test]
fn test_writable_getter() {
    // Only the getter is checked; the setter never needs to return.
    let invalid =
        "const v = computed({ get() { const n = count.value }, set(x) { count.value = x } })";
    assert_eq!(lint(invalid).error_count, 1);
    let valid = "const v = computed({ get() { return count.value }, set(x) { count.value = x } })";
    assert_eq!(lint(valid).error_count, 0);
}

#[test]
fn test_invalid_return_only_in_nested_function() {
    // The only `return` belongs to a nested function, not the getter.
    assert_eq!(
        lint("const d = computed(() => { const h = () => { return count.value } })").error_count,
        1
    );
}

#[test]
fn test_valid_return_in_branch() {
    assert_eq!(
        lint("const s = computed(() => { if (c.value) { return 'p' } return 'n' })").error_count,
        0
    );
}

#[test]
fn test_invalid_bare_return_without_value() {
    // `return;` does not produce a value.
    assert_eq!(
        lint("const d = computed(() => { if (!c.value) { return } })").error_count,
        1
    );
}

#[test]
fn test_ignores_non_computed_call() {
    assert_eq!(
        lint("const r = useFoo(() => { const x = 1 })").error_count,
        0
    );
}

#[test]
fn test_multiple_invalid_computed_reported() {
    assert_eq!(
        lint("const a = computed(() => { let x = 1 })\nconst b = computed(() => { let y = 2 })")
            .error_count,
        2
    );
}

// --- Options API ---

#[test]
fn test_valid_options_getters() {
    // Method getter with return and concise-arrow getter.
    assert_eq!(
        lint("export default { computed: { total() { return this.a } } }").error_count,
        0
    );
    assert_eq!(
        lint("export default { computed: { total: () => this.a } }").error_count,
        0
    );
}

#[test]
fn test_invalid_options_getter_without_return() {
    let result = lint("export default { computed: { total() { const s = this.a } } }");
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_options_accessor_getter_without_return() {
    let source = "export default { computed: { value: { get() { const n = this.count }, set(v) { this.count = v } } } }";
    assert_eq!(lint(source).error_count, 1);
}

#[test]
fn test_define_component_options_getter() {
    let source = "import { defineComponent } from 'vue'\nexport default defineComponent({ computed: { total() { const s = this.a } } })";
    assert_eq!(lint(source).error_count, 1);
}

#[test]
fn test_identifier_export_options_getter() {
    let source = "const c = { computed: { total() { const s = this.a } } }\nexport default c";
    assert_eq!(lint(source).error_count, 1);
}

#[test]
fn test_valid_no_getter_to_check() {
    // No `computed` option, and a spread (`...mapGetters`) is ignored.
    assert_eq!(
        lint("export default { data() { return { count: 0 } } }").error_count,
        0
    );
    assert_eq!(
        lint("export default { computed: { ...mapGetters(['count']) } }").error_count,
        0
    );
}

#[test]
fn test_multiple_options_getters_mixed() {
    let source =
        "export default { computed: { good() { return this.a }, bad() { const x = this.b } } }";
    assert_eq!(lint(source).error_count, 1);
}
