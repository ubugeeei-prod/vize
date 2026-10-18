use crate::virtual_ts::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions,
    generate_virtual_ts_with_offsets_and_checks,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn check_tail_line(code: &str) -> &str {
    code.lines()
        .find(|line| {
            line.as_bytes()
                .starts_with(b"  type __VizeComponentCheckTail")
        })
        .expect("component prop helpers should declare a check tail")
}

fn generate_parent_usage(check_unknown_props: bool) -> vize_carton::String {
    let script = r#"import Child from './Child.vue'
"#;
    let template = r#"<Child title="ok" :gradient="false" disable />"#;
    generate_usage(script, template, check_unknown_props)
}

fn generate_usage(script: &str, template: &str, check_unknown_props: bool) -> vize_carton::String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            check_options: VirtualTsCheckOptions {
                check_unknown_props,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .code
}

#[test]
fn unknown_props_off_opens_generated_check_tail() {
    assert_eq!(
        check_tail_line(&generate_parent_usage(false)),
        "  type __VizeComponentCheckTail<C> = __VizeIsGeneratedComponent<C> extends true ? __VizePublicComponentAttrs & Record<string, unknown> : Record<string, unknown>;",
    );
}

#[test]
fn unknown_props_on_keeps_native_fallthrough_tail() {
    assert_eq!(
        check_tail_line(&generate_parent_usage(true)),
        "  type __VizeComponentCheckTail<C> = __VizeIsGeneratedComponent<C> extends true ? __VizePublicComponentAttrs & __VizeGlobalHtmlAttrs & __VizeAllowedFallthroughAttrs<C> : __VizePublicComponentAttrs & __VizeGlobalHtmlAttrs;",
    );
}

#[test]
fn component_listener_inference_accepts_camelized_data_and_aria_attrs() {
    let code = generate_usage(
        "import Child from './Child.vue'",
        r#"<Child data-test="x" :aria-hidden="true" @close="() => {}" />"#,
        true,
    );
    assert!(code.contains("\"dataTest\": \"x\""), "{code}");
    assert!(code.contains("\"ariaHidden\": true"), "{code}");
    // Listener inference instantiates the child's public constructor, not just
    // the usage's separate checker. Both spellings must inhabit that contract.
    assert!(
        code.contains("[K in `data-${string}` | `data${Capitalize<string>}`]"),
        "{code}"
    );
    assert!(
        code.contains("[K in `aria-${string}` | `aria${Capitalize<string>}`]"),
        "{code}"
    );
}

#[test]
fn abstract_component_constructor_extracts_its_actual_instance_props() {
    let code = generate_usage(
        "defineProps<{ is: abstract new (...args: any) => any }>()",
        r#"<component :is="is" foo="x" />"#,
        true,
    );
    assert!(
        code.contains("type __VizePublicProps<C> = C extends abstract new (...args: any[]) => infer __I ? __VizeIsAny<__I> extends true ? Record<string, unknown> : __I extends { $props: infer __P } ? __P : {}"),
        "{code}"
    );
    assert!(code.contains("\"foo\": \"x\""), "{code}");
    assert!(
        code.contains("__VizeComponentRawProps<typeof is>"),
        "{code}"
    );
}

#[test]
fn empty_options_component_does_not_infer_unowned_optional_raw_metadata() {
    let code = generate_usage("const Div = {}", "<Div />", false);
    // The navigation completion pattern must receive `{}`, not `unknown`,
    // even when this usage has no authored props to check.
    assert!(
        code.contains("__VizeHasStaticRawProps<C> extends true ? __VizeStaticRawProps<C> : __VizePublicProps<C>"),
        "{code}"
    );
    assert!(
        code.contains("const { } = undefined as unknown as __Div_Props_0;"),
        "{code}"
    );
    assert!(
        code.contains("__VizeComponentRawProps<typeof Div>"),
        "{code}"
    );
}
