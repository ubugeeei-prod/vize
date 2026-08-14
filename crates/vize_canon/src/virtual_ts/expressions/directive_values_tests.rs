use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

/// A custom directive's value is assigned to the `Value` parameter of the
/// directive's declared `Directive<El, Value>`, which is what makes
/// `v-focus="'nope'"` a `TS2322` instead of silence (#3445).
#[test]
fn custom_directive_value_is_assigned_to_the_declared_value_type() {
    let script = r#"import type { Directive } from 'vue'
const vFocus: Directive<HTMLElement, number> = () => {}"#;
    let template = r#"<div v-focus="'nope'" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, summary) = analyze(&allocator, script, template);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output
            .code
            .contains("__VizeDirectiveValue<typeof vFocus> = ('nope');"),
        "the authored value should be checked against the directive's value type:\n{}",
        output.code
    );
}

/// TypeScript anchors `TS2322` on an annotated `const` at the *declaration
/// name*, so the check identifier's sub-span has to map to the authored value
/// rather than to the directive name. The oracle in #3445 reports at the start
/// of `'nope'`; anchoring on `v-focus` like the native prop check does would
/// put the diagnostic in the wrong place.
#[test]
fn custom_directive_check_identifier_anchors_at_the_authored_value() {
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
        .flat_map(|mapping| &mapping.sub_spans)
        .filter(|span| span.src_range == value_range)
        .collect();

    assert!(
        spans
            .iter()
            .any(|span| output.code[span.gen_range.clone()].starts_with("__vize_directive_check_")),
        "the synthetic check identifier must carry the diagnostic to the authored value:\n{}",
        output.code
    );
    assert!(
        spans
            .iter()
            .any(|span| &output.code[span.gen_range.clone()] == "'nope'"),
        "the initializer must retain its authored expression range:\n{}",
        output.code
    );
}

/// `v-focus` resolves to `vFocus` by Vue's own convention, and a directive that
/// has no such setup binding is registered globally — through
/// `app.directive('focus', …)`, a plugin, or an Options API `directives` block.
/// Naming `vFocus` anyway would report `TS2304: Cannot find name` on every
/// globally registered directive in the ecosystem, so an unbound directive stays
/// unchecked.
#[test]
fn globally_registered_directive_emits_no_check() {
    let script = "const unrelated = 1";
    let template = r#"<div v-focus="'nope'" />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, summary) = analyze(&allocator, script, template);
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        !output.code.contains("__vize_directive_check_"),
        "a directive with no setup binding must not be named:\n{}",
        output.code
    );
    assert!(
        !output.code.contains("typeof vFocus"),
        "an unbound directive name must never reach the generated program:\n{}",
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
        output
            .code
            .contains("__VizeDirectiveValue<typeof vMyDirective> = ('nope');"),
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
        output
            .code
            .contains("__VizeDirectiveValue<typeof vFocus> = ('nope');"),
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

/// The `Directive` alias is declared once, guarded against a `vue` that does not
/// export it, and kept referenced so it cannot be reported unused.
///
/// Both halves are load-bearing, and neither substitutes for the other — see the
/// `NativeElements` precedent in [`super::native_props`]. `@ts-ignore` covers the
/// `TS2694`/`TS2307` resolution error that a Vue 2.7 or shimmed `vue` produces;
/// it does *not* cover `TS6196` "declared but never used", which arrives on the
/// suggestion channel and reached `check-server` clients as an unmapped hint on
/// clean SFCs. The ambient `declare function` covers that one.
#[test]
fn directive_value_alias_is_guarded_and_kept_referenced() {
    const GUARD: &str = "// @ts-ignore TS2694/TS2307: a `vue` without `Directive` must degrade custom directive value checks to unchecked, never error. See virtual_ts/expressions/directive_values.rs.";
    const ALIAS: &str = "type __VizeDirectiveValue<D> = D extends import('vue').Directive<any, infer V> ? V : unknown;";
    const REFERENCE: &str =
        "declare function __vizeDirectiveValue<__D>(value: __VizeDirectiveValue<__D>): void;";

    for (name, text) in [
        (
            "SHARED_PREAMBLE_DTS",
            crate::virtual_ts::SHARED_PREAMBLE_DTS,
        ),
        (
            "VUE_TYPE_HELPERS",
            crate::virtual_ts::helpers::VUE_TYPE_HELPERS,
        ),
    ] {
        assert!(text.contains(GUARD), "{name} must carry the degrade guard");
        assert!(text.contains(ALIAS), "{name} must declare the alias");
        assert_eq!(
            text.matches("import('vue').Directive<").count(),
            1,
            "{name} must name `Directive` exactly once, so a `vue` without it \
             degrades one alias rather than erroring per binding"
        );
        assert!(
            text.contains(REFERENCE),
            "{name} must keep the alias referenced against TS6196"
        );
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
