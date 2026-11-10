use super::NoRefAsOperand;
use crate::linter::Linter;
use crate::rules::script::{ScriptLintResult, ScriptLinter};

const IMPORTS: &str = "import { ref, computed, shallowRef, toRef, customRef } from \"vue\";\n";

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoRefAsOperand));
    linter.lint(&format!("{IMPORTS}{source}"), 0)
}

// --- Valid: proper `.value` access, or passing the ref itself ---

#[test]
fn test_valid_value_access() {
    assert_eq!(lint("const count = ref(0)\ncount.value++").error_count, 0);
    assert_eq!(
        lint("const count = ref(0)\nconsole.log(count.value + 1)").error_count,
        0
    );
    assert_eq!(
        lint("const count = ref(0)\nif (count.value) {}").error_count,
        0
    );
}

#[test]
fn test_valid_passing_ref_itself() {
    // Passing the ref as a call argument (not an operand) is fine.
    assert_eq!(
        lint("const count = ref(0)\nwatch(count, () => {})").error_count,
        0
    );
    assert_eq!(lint("const count = ref(0)\nuseFoo(count)").error_count, 0);
    assert_eq!(
        lint("const count = ref(0)\nreturn { count }").error_count,
        0
    );
}

#[test]
fn emit_payload_must_unwrap_a_ref() {
    let result = lint(
        "const emit = defineEmits(['change'])\nconst active = computed(() => true)\nemit('change', active)",
    );
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn setup_resolves_ref_factories_imported_by_sibling_script() {
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["script/no-ref-as-operand".into()]))
        .lint_sfc(
            r#"<script lang="ts">import { computed } from 'vue'</script>
<script setup lang="ts">
const emit = defineEmits(['change'])
const active = computed(() => true)
emit('change', active)
</script>"#,
            "Probe.vue",
        );
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn test_valid_ref_replacement() {
    assert_eq!(lint("let count = ref(0)\ncount = ref(1)").error_count, 0);
}

#[test]
fn test_invalid_plain_value_assignment() {
    assert_eq!(lint("let count = ref(0)\ncount = 1").error_count, 1);
}

#[test]
fn test_valid_non_ref_binding() {
    // Not initialized from a ref factory: never tracked.
    assert_eq!(lint("const count = 0\ncount + 1").error_count, 0);
    assert_eq!(
        lint("const state = reactive({ n: 0 })\nstate.n + 1").error_count,
        0
    );
}

#[test]
fn test_invalid_typeof_and_void() {
    // `typeof count === "number"` tests the ref object and is always false.
    assert_eq!(
        lint("const count = ref(0)\nif (typeof count === 'number') {}\nvoid count").error_count,
        2
    );
}

// --- Invalid: update / unary / binary / logical / conditional / condition ---

#[test]
fn test_invalid_update_expression() {
    let result = lint("let count = ref(0)\ncount++");
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_prefix_update() {
    assert_eq!(lint("let count = ref(0)\n--count").error_count, 1);
}

#[test]
fn test_invalid_binary_operand() {
    assert_eq!(
        lint("const count = ref(0)\nconsole.log(count + 1)").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = 1 + count").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = count === 0").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = count < 5").error_count,
        1
    );
}

#[test]
fn test_invalid_template_literal_interpolation() {
    assert_eq!(
        lint("const count = ref(0)\nconst text = `${count}`").error_count,
        1
    );
}

#[test]
fn test_invalid_unary_operand() {
    assert_eq!(
        lint("const flag = ref(false)\nconst x = !flag").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = -count").error_count,
        1
    );
}

#[test]
fn test_invalid_logical_operand() {
    assert_eq!(
        lint("const flag = ref(false)\nconst x = flag && other").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = (count) ?? 1").error_count,
        1
    );
}

#[test]
fn test_valid_logical_right_operand_and_let_left_operand() {
    // The right operand is the selected value; a `let` ref may be reassigned.
    assert_eq!(
        lint("const count = ref(0)\nconst x = other ?? count\nconst y = other || count")
            .error_count,
        0
    );
    assert_eq!(
        lint("let count = ref(0)\nconst x = count || 1\ncount = 2").error_count,
        1
    );
}

#[test]
fn test_tagged_template_values_are_not_stringified() {
    assert_eq!(
        lint("const count = ref(0)\nconst a = tag`${count}`\nconst b = `${count}`").error_count,
        1
    );
}

#[test]
fn test_array_destructured_define_model_binds_the_model_ref() {
    assert_eq!(
        lint("const [model, modifiers] = defineModel()\nif (model) {}\nconst text = model + modifiers").error_count,
        2
    );
}

#[test]
fn test_shadowed_define_model_does_not_bind_a_ref() {
    assert_eq!(
        lint("const defineModel = () => [0, {}]\nconst [model] = defineModel()\nif (model) {}")
            .error_count,
        0
    );
}

#[test]
fn test_invalid_conditional_test() {
    assert_eq!(
        lint("const flag = ref(false)\nconst x = flag ? 1 : 2").error_count,
        1
    );
}

#[test]
fn test_invalid_if_condition() {
    let result = lint("const flag = ref(false)\nif (flag) {}");
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_while_condition() {
    assert_eq!(
        lint("const flag = ref(true)\nwhile (flag) {}").error_count,
        1
    );
    assert_eq!(
        lint("const flag = ref(true)\ndo {} while (flag)").error_count,
        1
    );
}

#[test]
fn test_invalid_for_condition() {
    assert_eq!(
        lint("const flag = ref(true)\nfor (;flag;) {}").error_count,
        1
    );
}

#[test]
fn test_invalid_compound_assignment() {
    assert_eq!(lint("let count = ref(0)\ncount += 1").error_count, 1);
    assert_eq!(lint("let count = ref(0)\ncount *= 2").error_count, 1);
}

// --- Factory variants ---

#[test]
fn test_invalid_other_factories() {
    assert_eq!(lint("const c = computed(() => 1)\nc + 1").error_count, 1);
    assert_eq!(lint("const c = shallowRef(0)\nc + 1").error_count, 1);
    assert_eq!(lint("const c = toRef(props, 'x')\nc + 1").error_count, 1);
    assert_eq!(
        lint("const c = customRef(() => ({}))\nc + 1").error_count,
        1
    );
}

#[test]
fn define_model_template_refs_and_switch_discriminants_are_operands() {
    let source = r#"
import { useTemplateRef } from 'vue'
const mode = ref('list')
const open = defineModel('open')
const page = defineModel('page')
const input = useTemplateRef('input')
const next = page + 1
if (open) input.value?.focus()
if (!input) mode.value = 'list'
switch (mode) { case 'grid': break }
"#;
    assert_eq!(lint(source).error_count, 4);
}

#[test]
fn to_refs_members_and_explicit_vue_ref_return_types_are_refs() {
    let source = r#"
import { toRefs } from 'vue'
import type { Ref } from 'vue'
const { active } = toRefs(props)
declare function useOpen(): Ref<boolean>
const open = useOpen()
if (active) console.log('active')
const label = open ? 'open' : 'closed'
"#;
    assert_eq!(lint(source).error_count, 2);
}

#[test]
fn test_factory_with_ts_cast() {
    assert_eq!(
        lint("const count = ref(0) as Ref<number>\ncount + 1").error_count,
        1
    );
}

// --- Scoping / soundness ---

#[test]
fn test_shadowing_param_not_flagged() {
    // The outer `count` is a ref, but the inner `count` parameter shadows it
    // and is not a ref, so `count + 1` inside `f` must not be flagged.
    let source = "const count = ref(0)\nfunction f(count) { return count + 1 }";
    assert_eq!(lint(source).error_count, 0);
}

#[test]
fn test_shadowing_local_not_flagged() {
    let source = "const count = ref(0)\nfunction f() { const count = 1; return count + 1 }";
    assert_eq!(lint(source).error_count, 0);
}

#[test]
fn test_shadowing_block_local_not_flagged() {
    let source = "\
const maxHeight = ref(0)
const previewUrl = computed(() => {
  const maxHeight = parseInt(paramClass.get('maxHeight'))
  return maxHeight === 0 ? '500' : Math.min(maxHeight, 700).toString()
})";
    assert_eq!(lint(source).error_count, 0);
}

#[test]
fn test_block_shadowing_does_not_leak() {
    let source = "const count = ref(0)\n{ const count = 1; count + 1 }\ncount + 1";
    assert_eq!(lint(source).error_count, 1);
}

#[test]
fn test_catch_param_shadowing_not_flagged() {
    let source = "const error = ref(false)\ntry {} catch (error) { if (error) {} }";
    assert_eq!(lint(source).error_count, 0);
}

#[test]
fn test_for_of_binding_shadowing_not_flagged() {
    let source = "const item = ref(0)\nfor (const item of items) { item + 1 }";
    assert_eq!(lint(source).error_count, 0);
}

#[test]
fn test_ref_used_inside_nested_function() {
    // A top-level ref used as an operand inside a nested function is flagged.
    let source = "const count = ref(0)\nfunction f() { return count + 1 }";
    assert_eq!(lint(source).error_count, 1);
}

#[test]
fn test_multiple_invalid_reported() {
    let source = "const a = ref(0)\nconst b = ref(0)\nconst x = a + b";
    assert_eq!(lint(source).error_count, 2);
}

#[test]
fn test_no_refs_no_reports() {
    assert_eq!(
        lint("const x = 1\nconst y = x + 1\nif (y) {}").error_count,
        0
    );
}

#[test]
fn real_app_loop_shadow_does_not_hide_a_true_ref_operand() {
    let source = "const options = ref([]);\nfor (let i = 10; i < 36; i++) { const value = i.toString(36) + i; options.value.push({ label: `Long Label: ${value}`, value }); }\nconst value = ref(['a']);\nconst broken = `value: ${value}`;";
    let result = lint(source);
    assert_eq!(result.error_count, 1);
    let start = (IMPORTS.len() + source.rfind("value}").unwrap()) as u32;
    assert_eq!(
        (result.diagnostics[0].start, result.diagnostics[0].end),
        (start, start + 5)
    );
}
