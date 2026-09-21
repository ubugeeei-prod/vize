use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

/// A custom directive is checked through its original hook, which makes
/// `v-focus="'nope'"` a `TS2322` instead of silence (#3445).
#[test]
fn custom_directive_value_calls_the_declared_hook() {
    let script = r#"import type { Directive } from 'vue'
const vFocus: Directive<HTMLElement, number> = () => {}"#;
    let template = r#"<div v-focus="'nope'" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, summary) = analyze(&allocator, script, template);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output.code.contains("= __vizeDirective(vFocus);"),
        "the authored value should be checked against the directive's value type:\n{}",
        output.code
    );
}

/// TypeScript anchors incompatible binding values on the `value` key. Keep
/// that key and the expression mapped, without mapping synthetic call arguments.
#[test]
fn custom_directive_value_key_anchors_at_the_authored_value() {
    let script = r#"import type { Directive } from 'vue'
const vFocus: Directive<HTMLElement, number> = () => {}"#;
    let template = r#"<div v-focus="'nope'" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, summary) = analyze(&allocator, script, template);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    let value_start = template.find("'nope'").expect("bound expression");
    let value_range = value_start..value_start + "'nope'".len();
    let spans: Vec<_> = output
        .mappings
        .iter()
        .filter(|span| span.src_range == value_range)
        .collect();

    assert!(
        spans
            .iter()
            .any(|span| &output.code[span.gen_range.clone()] == "value"),
        "the synthetic binding key must carry the diagnostic to the authored value:\n{}",
        output.code
    );
    assert!(
        spans
            .iter()
            .any(|span| &output.code[span.gen_range.clone()] == "'nope'"),
        "the argument must retain its authored expression range:\n{}",
        output.code
    );
}

/// `v-focus` resolves to `vFocus` by Vue's own convention, and a directive that
/// has no such setup binding is registered elsewhere: through
/// `app.directive('focus', …)` / a plugin (`GlobalDirectives`), or an Options API
/// `directives` block. Naming `vFocus` as a value would report `TS2304: Cannot
/// find name` on every such directive in the ecosystem, so the name only ever
/// appears as a key into those registries, and an unregistered one is unchecked.
#[test]
fn a_directive_without_a_setup_binding_resolves_through_the_registries() {
    let script = "const unrelated = 1";
    let template = r#"<div v-focus:target.once="'nope'" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, summary) = analyze(&allocator, script, template);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    let check: Vec<&str> = output
        .code
        .lines()
        .filter(|line| line.contains("__vize_directive_check_"))
        .map(str::trim)
        .collect();
    assert_eq!(
        check,
        [
            r#"const __vize_directive_check_26 = __vizeDirective(__vizeRegisteredDirective<unknown, "vFocus">());"#,
            r#"__vize_directive_check_26(null!, { ...__vizeDirectiveBindingRest, arg: "target", modifiers: { "once": true, }, value: ('nope') }, ...__vizeDirectiveTail(__vize_directive_check_26)); // CustomDirective"#,
        ],
        "{}",
        output.code
    );
}

/// Kebab-case directive names follow the same convention:
/// `v-my-directive` -> `vMyDirective`.
#[test]
fn kebab_case_directive_resolves_to_its_camel_case_binding() {
    let script = r#"import type { Directive } from 'vue'
const vMyDirective: Directive<HTMLElement, number> = () => {}"#;
    let template = r#"<div v-my-directive="'nope'" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, summary) = analyze(&allocator, script, template);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output.code.contains("= __vizeDirective(vMyDirective);"),
        "kebab-case directive names resolve to their camel-case binding:\n{}",
        output.code
    );
}

/// Builtin directives keep their existing paths. `v-show` and `v-model` are
/// collected as their own expression kinds, and `v-if`/`v-for` are handled by
/// the control-flow and scope emitters, so none of them may pick up a directive
/// value check.
#[test]
fn builtin_directives_are_not_treated_as_custom() {
    let script = "const flag = true\nconst items = [1]\nconst text = 'x'";
    let template = r#"<div v-show="flag"><p v-for="i in items" :key="i">{{ i }}</p><input v-model="text" /></div>"#;
    let allocator = vize_carton::Allocator::new();
    let (root, summary) = analyze(&allocator, script, template);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        !output.code.contains("__vize_directive_check_"),
        "builtin directives must keep their existing emission:\n{}",
        output.code
    );
}

/// A directive argument and modifiers sit in later `Directive` type parameters,
/// so `Value` stays in the same position and `v-focus:arg.mod` checks its value
/// identically. Checking the argument and modifiers themselves is separate work.
#[test]
fn directive_argument_and_modifiers_do_not_change_the_value_check() {
    let script = r#"import type { Directive } from 'vue'
const vFocus: Directive<HTMLElement, number> = () => {}"#;
    let template = r#"<div v-focus:arg.mod="'nope'" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, summary) = analyze(&allocator, script, template);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output.code.contains("= __vizeDirective(vFocus);"),
        "an argument and modifiers must not change the value check:\n{}",
        output.code
    );
}

/// A custom directive with no value at all (`<div v-focus />`) has nothing to
/// check and must not emit a dangling declaration.
#[test]
fn valueless_custom_directive_emits_no_check() {
    let script = r#"import type { Directive } from 'vue'
const vFocus: Directive<HTMLElement, number> = () => {}"#;
    let template = r#"<div v-focus />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, summary) = analyze(&allocator, script, template);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        !output.code.contains("__vize_directive_check_"),
        "a valueless directive has no expression to check:\n{}",
        output.code
    );
}

/// Structural hook resolution preserves generic signatures and also works
/// without importing a Vue 3-only `Directive` alias into every document.
#[test]
fn directive_helpers_preserve_hooks_without_a_vue_type_dependency() {
    for text in [
        crate::virtual_ts::SHARED_PREAMBLE_DTS,
        crate::virtual_ts::helpers::VUE_TYPE_HELPERS,
    ] {
        assert!(!text.contains("__VizeDirectiveValue"));
        assert!(!text.contains("import('vue').Directive<"));
        assert!(text.contains("declare function __vizeDirective<D>"));
        assert!(text.contains("__VizeDirectiveHook<D>"));
        assert!(text.contains("declare function __vizeDirectiveTail<"));
    }
}

fn analyze<'a>(
    allocator: &'a vize_carton::Allocator,
    script: &str,
    template: &'a str,
) -> (vize_relief::RootNode<'a>, vize_croquis::Croquis) {
    let (root, _) = vize_armature::parse(allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    (root, analyzer.finish())
}
