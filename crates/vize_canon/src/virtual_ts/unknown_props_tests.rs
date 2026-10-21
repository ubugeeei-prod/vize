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

fn type_declaration<'a>(code: &'a str, name: &str) -> Option<&'a str> {
    code.lines().find(|line| {
        line.strip_prefix("  type ")
            .and_then(|rest| rest.split(['<', '=']).next())
            .is_some_and(|declared| declared.trim_end() == name)
    })
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
    let code = generate_parent_usage(true);
    assert_eq!(
        check_tail_line(&code),
        "  type __VizeComponentCheckTail<C> = __VizeIsGeneratedComponent<C> extends true ? __VizePublicComponentAttrs & __VizeComponentGlobalHtmlAttrs & __VizeAllowedFallthroughAttrs<C> : __VizePublicComponentAttrs & __VizeComponentGlobalHtmlAttrs;",
    );
    assert_eq!(type_declaration(&code, "__VizeAttrCamel"), None);
    assert_eq!(type_declaration(&code, "__VizeNativeAttrNames"), None);
    assert_eq!(type_declaration(&code, "__VizeGlobalHtmlAttrs"), None);
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
fn listener_inference_maps_its_own_prop_keys_values_and_spreads() {
    let script = "import Child from './Child.vue'; const attrs = { hidden: true };";
    for spread in ["", "v-bind=\"attrs\" "] {
        let template = vize_carton::cstr!(
            "<!-- 日本語 -->\r\n<Child {spread}:hidden=\"'wrong'\" @close=\"() => {{}}\" />"
        );
        let allocator = vize_carton::Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, &template);
        assert!(errors.is_empty());
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let template_offset = 91;
        let output = generate_virtual_ts_with_offsets_and_checks(
            &analyzer.finish(),
            Some(script),
            Some(&root),
            17,
            template_offset,
            &VirtualTsOptions::default(),
            VirtualTsGenerationOptions::default(),
        );
        let inference = output
            .code
            .find("const __vize_emit_props_")
            .expect("the listener must instantiate the child's public input contract");
        let generated_key = inference
            + output
                .code
                .get(inference..)
                .unwrap()
                .find("\"hidden\"")
                .unwrap();
        let generated_value = inference
            + output
                .code
                .get(inference..)
                .unwrap()
                .find("'wrong'")
                .unwrap();
        let source_key = template_offset as usize + template.find(":hidden").unwrap() + 1;
        let source_value = template_offset as usize + template.find("'wrong'").unwrap();
        let row = output
            .mapping
            .rows()
            .find(|row| row.span.gen_range.start == generated_key)
            .expect("inference must map its own literal, not borrow the primary prop checker");
        assert_eq!(row.meta.features, super::ProjectionFeatures::DIAGNOSTICS);
        let row = row.span;
        for feature in [
            super::ProjectionFeatures::HOVER,
            super::ProjectionFeatures::COMPLETION,
            super::ProjectionFeatures::DEFINITION,
        ] {
            let target = output
                .mapping
                .to_generated_for(source_key, feature)
                .expect("the primary prop owner remains navigable");
            assert!(
                !row.gen_range.contains(&target),
                "inference witnesses must not claim editor operations"
            );
        }
        assert_eq!(
            output
                .mapping
                .diagnostic_range_to_authored(generated_value, generated_value + 7),
            Some((source_value, source_value + 7))
        );
        for (generated, source) in [
            (generated_key..generated_key + 8, source_key..source_key + 6),
            (
                generated_key + 1..generated_key + 7,
                source_key..source_key + 6,
            ),
            (
                generated_value..generated_value + 7,
                source_value..source_value + 7,
            ),
        ] {
            assert!(
                row.sub_spans
                    .iter()
                    .any(|span| { span.gen_range == generated && span.src_range == source })
            );
        }
        if !spread.is_empty() {
            let literal_start =
                inference + output.code.get(inference..).unwrap().find("({\n").unwrap() + 1;
            let literal_end = literal_start
                + output
                    .code
                    .get(literal_start..)
                    .unwrap()
                    .find("});\n")
                    .unwrap()
                + 1;
            let tag_start = template_offset as usize + template.find("<Child").unwrap() + 1;
            let row = output
                .mapping
                .rows()
                .find(|row| row.span.gen_range == (literal_start..literal_end))
                .expect("the whole inference argument must own a diagnostic span");
            assert_eq!(row.meta.features, super::ProjectionFeatures::DIAGNOSTICS);
            assert_eq!(row.span.src_range, tag_start..tag_start + "Child".len());
            assert_eq!(
                output
                    .mapping
                    .diagnostic_range_to_authored(literal_start, literal_end),
                Some((tag_start, tag_start + "Child".len()))
            );
            for feature in [
                super::ProjectionFeatures::HOVER,
                super::ProjectionFeatures::COMPLETION,
                super::ProjectionFeatures::DEFINITION,
            ] {
                let target = output.mapping.to_generated_for(tag_start, feature).unwrap();
                assert!(!row.span.gen_range.contains(&target));
            }
            let generated = inference
                + output
                    .code
                    .get(inference..)
                    .unwrap()
                    .find("...attrs")
                    .unwrap()
                + 3;
            let source = template_offset as usize + template.find("\"attrs\"").unwrap() + 1;
            assert!(output.mapping.spans().iter().any(|row| {
                row.sub_spans.iter().any(|span| {
                    span.gen_range == (generated..generated + 5)
                        && span.src_range == (source..source + 5)
                })
            }));
        }
    }
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
        code.contains(
            "const { } = undefined as unknown as __VizeComponentNavigationProps<__Div_Props_0>;"
        ),
        "{code}"
    );
    assert!(
        code.contains("__VizeComponentRawProps<typeof Div>"),
        "{code}"
    );
}

/// `vue-tsc` keeps `aria-*` component attributes as HTML attributes
/// (`htmlAttributes: ['aria-*']`), so the checked props object must not
/// camelize them onto a declared `ariaLabel` prop.
#[test]
fn checked_component_props_keep_aria_attributes_as_authored() {
    let code = generate_usage(
        "import Child from './Child.vue'; const label: string | undefined = undefined;",
        r#"<Child :aria-label="label" :data-test="label" />"#,
        true,
    );
    assert!(code.contains("\"aria-label\": label"), "{code}");
    assert!(code.contains("\"dataTest\": label"), "{code}");
}
