use super::*;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn lint(rule: Box<dyn Rule>, source: &str) -> crate::linter::LintResult {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry).lint_sfc(source, "Component.vue")
}

#[test]
fn reports_style_mode_and_content_outside_blocks() {
    let style = lint(
        Box::new(EnforceStyleAttribute::default()),
        "<template><div /></template><style module>.x{}</style>",
    );
    assert_eq!(style.warning_count, 1);
    let outside = lint(
        Box::new(NoContentOutsideBlock),
        "stray<!-- kept --><template><div /></template>",
    );
    assert_eq!(outside.warning_count, 1);
}

#[test]
fn reports_setup_macro_compatibility_failures() {
    let model = lint(
        Box::new(NoDefineModelDefault),
        "<script setup>defineModel({ default: 1 })</script><template />",
    );
    assert_eq!(model.warning_count, 1);
    let variable = lint(
        Box::new(RequireMacroVariableName),
        "<script setup>const p = defineProps({ value: String })</script><template />",
    );
    assert_eq!(variable.warning_count, 1);
    let shadow = lint(
        Box::new(NoPropsShadow),
        "<script setup>defineProps({ value: String }); const value = 1</script><template />",
    );
    assert_eq!(shadow.warning_count, 1);
}

#[test]
fn reports_next_tick_native_events_and_unsorted_options() {
    let next_tick = lint(
        Box::new(NextTickStyle),
        "<script setup>nextTick(() => ready())</script><template />",
    );
    assert_eq!(next_tick.warning_count, 1);
    let event = lint(
        Box::new(NoShadowNativeEvents),
        "<script setup>const emit = defineEmits(['click'])</script><template />",
    );
    assert_eq!(event.warning_count, 1);
    let sorted = lint(
        Box::new(SortKeys::default()),
        "<script>export default { methods: { zebra() {}, alpha() {} } }</script><template />",
    );
    assert_eq!(sorted.warning_count, 1);
}
