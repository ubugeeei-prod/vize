use super::NoBrowserGlobalsInSsr;
use crate::Linter;
use crate::context::{LintContext, SsrMode};
use crate::rule::{Rule, RuleRegistry};
use vize_l0::CompactString;

fn lint_with_ssr(source: &str) -> Vec<CompactString> {
    let mut registry = RuleRegistry::new();
    registry.add(Box::new(NoBrowserGlobalsInSsr));
    let _linter = Linter::with_registry(registry);

    // Create allocator and context
    use vize_l0::Allocator;
    let allocator = Allocator::with_capacity(1024);
    let mut ctx = LintContext::with_locale(
        &allocator,
        source,
        "test.vue",
        crate::Linter::default().locale(),
    );
    ctx.set_ssr_mode(SsrMode::Enabled);

    let parser = vize_armature::Parser::new(&allocator, source);
    let (root, _) = parser.parse();

    let rules: Vec<Box<dyn Rule>> = vec![Box::new(NoBrowserGlobalsInSsr)];
    let rule_names = [rules[0].meta().name];
    let mut visitor = crate::visitor::LintVisitor::new(&mut ctx, &rules, &rule_names, true);
    visitor.visit_root(&root);

    ctx.into_diagnostics()
        .into_iter()
        .map(|d| d.message)
        .collect()
}

#[test]
fn test_detects_window_in_interpolation() {
    let result = lint_with_ssr("<div>{{ window.innerWidth }}</div>");
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_detects_document_in_interpolation() {
    let result = lint_with_ssr("<div>{{ document.title }}</div>");
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_detects_navigator_in_directive() {
    let result = lint_with_ssr("<div :class=\"navigator.userAgent\"></div>");
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_allows_local_variable() {
    // If 'window' is a local variable (e.g., from v-for), it should be allowed
    let result = lint_with_ssr("<div v-for=\"window in windows\">{{ window }}</div>");
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_allows_scoped_slot_binding_named_open() {
    let result = lint_with_ssr(
        r#"<Dropdown v-slot="{ open }"><button :class="{ active: open }">{{ open }}</button></Dropdown>"#,
    );
    assert!(
        result.is_empty(),
        "Should not flag scoped slot variables, got: {:?}",
        result
    );
}

#[test]
fn test_detects_localstorage() {
    let result = lint_with_ssr("<div>{{ localStorage.getItem('key') }}</div>");
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_allows_globalthis_in_interpolation() {
    let result = lint_with_ssr("<div>{{ globalThis }}</div>");
    assert!(
        result.is_empty(),
        "Should not flag globalThis in SSR, got: {:?}",
        result
    );
}

#[test]
fn test_allows_self_in_directive() {
    let result = lint_with_ssr(r#"<div :data-target="self"></div>"#);
    assert!(
        result.is_empty(),
        "Should not flag self in SSR, got: {:?}",
        result
    );
}

#[test]
fn test_ignores_css_property_names_in_style_object() {
    // { top: 0 } - 'top' is an object key, not a reference to window.top
    let result = lint_with_ssr(r#"<div :style="{ position: 'absolute', top: 0, left: 0 }"></div>"#);
    assert!(
        result.is_empty(),
        "Should not flag CSS property names in style objects, got: {:?}",
        result
    );
}

#[test]
fn test_ignores_string_literal_values() {
    // 'window' is a string literal, not a reference to the window global
    let result = lint_with_ssr(r#"<div :class="'window'"></div>"#);
    assert!(
        result.is_empty(),
        "Should not flag string literals, got: {:?}",
        result
    );
}

#[test]
fn test_ignores_property_access() {
    // obj.top - 'top' is a property access, not a reference to window.top
    let result = lint_with_ssr(r#"<div>{{ obj.top }}</div>"#);
    // Only 'obj' should be checked, not 'top'
    assert!(
        result.is_empty(),
        "Should not flag property accesses, got: {:?}",
        result
    );
}

#[test]
fn test_detects_actual_global_in_style_value() {
    // { top: window.scrollY } - 'window' is a real global reference
    let result = lint_with_ssr(r#"<div :style="{ top: window.scrollY + 'px' }"></div>"#);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_ignores_typeof_window_guard() {
    let result = lint_with_ssr(r#"<div>{{ typeof window === 'undefined' }}</div>"#);
    assert!(
        result.is_empty(),
        "Should not flag direct typeof guards, got: {:?}",
        result
    );
}

#[test]
fn test_ignores_parenthesized_typeof_document_guard_in_directive() {
    let result =
        lint_with_ssr(r#"<div :class="typeof (document) === 'undefined' ? 'ssr' : 'dom'"></div>"#);
    assert!(
        result.is_empty(),
        "Should not flag parenthesized direct typeof guards, got: {:?}",
        result
    );
}

#[test]
fn test_detects_typeof_member_access() {
    let result = lint_with_ssr(r#"<div>{{ typeof window.innerWidth }}</div>"#);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_ignores_regex_literal_with_browser_global_name() {
    let result = lint_with_ssr(r#"<div>{{ /window|document/.test(name) }}</div>"#);
    assert!(
        result.is_empty(),
        "Should not flag browser global names inside regex literals, got: {:?}",
        result
    );
}

#[test]
fn test_ignores_block_comment_with_browser_global_name() {
    let result = lint_with_ssr(r#"<div>{{ value /* window */ }}</div>"#);
    assert!(
        result.is_empty(),
        "Should not flag browser global names inside comments, got: {:?}",
        result
    );
}

#[test]
fn test_detects_division_by_browser_global() {
    let result = lint_with_ssr(r#"<div>{{ width / window.innerWidth }}</div>"#);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_ignores_type_only_browser_names_in_template_handlers() {
    let result = lint_with_ssr(
        r#"<button @click="(event: MouseEvent) => emit(event as MouseEvent)">Go</button>"#,
    );
    assert!(result.is_empty(), "{result:?}");
}

#[test]
fn test_still_reports_runtime_browser_name_next_to_type() {
    let result = lint_with_ssr(
        r#"<button @click="(event: MouseEvent) => window.alert(event.type)">Go</button>"#,
    );
    assert_eq!(result.len(), 1, "{result:?}");
}

fn lint_sfc_with_only_this_rule(source: &str) -> Vec<CompactString> {
    let mut registry = RuleRegistry::new();
    registry.add(Box::new(NoBrowserGlobalsInSsr));
    Linter::with_registry(registry)
        .lint_sfc(source, "Panel.vue")
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.message)
        .collect()
}

#[test]
fn script_setup_open_and_close_are_not_browser_globals() {
    let result = lint_sfc_with_only_this_rule(
        r#"<script setup lang="ts">
const { open = false } = defineProps<{ open?: boolean }>();
const emit = defineEmits<{ close: [] }>();
function close(): void {
  emit("close");
}
</script>
<template>
  <div v-if="open">
    <button type="button" @click="close">Close</button>
  </div>
</template>
"#,
    );
    assert!(
        result
            .iter()
            .all(|message| !message.contains("'open'") && !message.contains("'close'")),
        "{result:?}"
    );
}

#[test]
fn bare_open_stays_a_browser_global_when_only_this_rule_is_enabled() {
    let result = lint_sfc_with_only_this_rule(
        r#"<script setup lang="ts">
const label = "panel";
</script>
<template>
  <button type="button" @click="open(label)">Open</button>
</template>
"#,
    );
    assert!(
        result.iter().any(|message| message.contains("'open'")),
        "{result:?}"
    );
    assert!(
        result.iter().all(|message| !message.contains("'label'")),
        "{result:?}"
    );
}
