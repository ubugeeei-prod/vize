mod union;

use crate::virtual_ts::generate_virtual_ts;

const STATIC_PAYLOAD_ALIASES: [&str; 3] = [
    "type __VizeSlotOutletTarget<",
    "type __VizeSlotOutletArgs<",
    "type __VizeSlotOutletPayload<",
];
const DYNAMIC_PAYLOAD_ALIASES: [&str; 3] = [
    "type __VizeAnySlotOutletTarget<",
    "type __VizeAnySlotOutletArgs<",
    "type __VizeAnySlotOutletPayload<",
];

fn generate(script: &str, template: &str) -> vize_carton::String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts(&summary, Some(script), Some(&root), 0).code
}

fn assert_payload_alias_partition(code: &str, needs_static: bool, needs_dynamic: bool) {
    let expected_fn = if needs_static || needs_dynamic { 1 } else { 0 };
    assert_eq!(
        code.matches("type __VizeSlotOutletFn =").count(),
        expected_fn,
        "payload function alias admission differs:\n{code}"
    );
    for alias in STATIC_PAYLOAD_ALIASES {
        assert_eq!(
            code.matches(alias).count(),
            usize::from(needs_static),
            "static payload alias admission differs for {alias}:\n{code}"
        );
    }
    for alias in DYNAMIC_PAYLOAD_ALIASES {
        assert_eq!(
            code.matches(alias).count(),
            usize::from(needs_dynamic),
            "dynamic payload alias admission differs for {alias}:\n{code}"
        );
    }
}

#[test]
fn infers_the_spread_expression_type() {
    let script = r#"const attrs = { label: 'ok' }
defineSlots<{ default(props: { label: string }): any }>()
"#;
    let template = r#"<slot v-bind="attrs" />"#;

    let code = generate(script, template);

    assert_eq!(
        code.matches("function __vizeSlotOutletSpread<__Expected>()")
            .count(),
        1,
        "slot outlet spread helper must split supplied and inferred generics:\n{}",
        code
    );
    assert_eq!(
        code.matches(
            r#"...__vizeSlotOutletSpread<__VizeSlotOutletPayload<__VizeSlots, "default">>()(attrs),"#,
        )
        .count(),
        1,
        "slot outlet spread call must infer its expression argument:\n{}",
        code
    );
}

#[test]
fn no_spread_omits_payload_aliases_without_dropping_outlet_checks() {
    let script = r#"const slotName: 'default' | 'static' = 'default'
const value = 'ok'
defineSlots<{
  default(props: { value: string }): any
  static(props: { value: string }): any
}>()
"#;
    let code = generate(
        script,
        r#"<div><slot :value="value" /><slot :name="slotName" :value="value" /></div>"#,
    );

    assert_payload_alias_partition(&code, false, false);
    assert_eq!(
        code.matches("function __vizeSlotOutlet<S>").count(),
        1,
        "declared outlets still need their normalization helper:\n{code}"
    );
    assert_eq!(
        code.matches("__vizeSlotOutlet((undefined as unknown as")
            .count(),
        2,
        "both authored outlet checks must remain:\n{code}"
    );
    assert_eq!(
        code.matches("function __vizeSlotOutletSpread<__Expected>()")
            .count(),
        0,
        "a component without a spread must not emit spread helpers:\n{code}"
    );
}

#[test]
fn payload_aliases_follow_the_name_kinds_that_actually_spread() {
    let script = r#"const attrs = { value: 'ok' }
const slotName: 'default' | 'static' = 'default'
const value = 'ok'
defineSlots<{
  default(props: { value: string }): any
  static(props: { value: string }): any
}>()
"#;
    let cases = [
        (
            r#"<div><slot name="static" v-bind="attrs" /><slot :name="slotName" :value="value" /></div>"#,
            true,
            false,
        ),
        (
            r#"<div><slot name="static" :value="value" /><slot :name="slotName" v-bind="attrs" /></div>"#,
            false,
            true,
        ),
        (
            r#"<div><slot name="static" v-bind="attrs" /><slot :name="slotName" v-bind="attrs" /></div>"#,
            true,
            true,
        ),
    ];

    for (template, needs_static, needs_dynamic) in cases {
        let code = generate(script, template);
        assert_payload_alias_partition(&code, needs_static, needs_dynamic);
        assert_eq!(
            code.matches("function __vizeSlotOutletSpread<__Expected>()")
                .count(),
            1,
            "spread infrastructure must remain shared per component:\n{code}"
        );
    }
}

#[test]
fn authored_payload_alias_names_stay_untouched_without_spreads() {
    let script = r#"type __VizeSlotOutletFn = { authored: 'fn' }
type __VizeSlotOutletTarget = { authored: 'static-target' }
type __VizeSlotOutletArgs = { authored: 'static-args' }
type __VizeSlotOutletPayload = { authored: 'static-payload' }
type __VizeAnySlotOutletTarget = { authored: 'dynamic-target' }
type __VizeAnySlotOutletArgs = { authored: 'dynamic-args' }
type __VizeAnySlotOutletPayload = { authored: 'dynamic-payload' }
const value = 'ok'
defineSlots<{ default(props: { value: string }): any }>()
"#;
    let code = generate(script, r#"<slot :value="value" />"#);

    for authored in [
        "authored: 'fn'",
        "authored: 'static-target'",
        "authored: 'static-args'",
        "authored: 'static-payload'",
        "authored: 'dynamic-target'",
        "authored: 'dynamic-args'",
        "authored: 'dynamic-payload'",
    ] {
        assert_eq!(
            code.matches(authored).count(),
            1,
            "authored declaration {authored} must remain in the virtual file:\n{code}"
        );
    }
    for generated in [
        "__VizeSlotOutletFn = (...args: any[]) => any",
        "__VizeSlotOutletTarget<__S, __K extends PropertyKey> = Extract<",
        "__VizeSlotOutletArgs<__S, __K extends PropertyKey> = [",
        "__VizeSlotOutletPayload<__S, __K extends PropertyKey> = [",
        "__VizeAnySlotOutletTarget<__S> = Extract<",
        "__VizeAnySlotOutletArgs<__S> = [",
        "__VizeAnySlotOutletPayload<__S> = [",
    ] {
        assert_eq!(
            code.matches(generated).count(),
            0,
            "no-spread generation must not collide with {generated}:\n{code}"
        );
    }
    assert_eq!(
        code.matches("__vizeSlotOutlet((undefined as unknown as")
            .count(),
        1,
        "the outlet check must survive authored helper-like names:\n{code}"
    );
}
