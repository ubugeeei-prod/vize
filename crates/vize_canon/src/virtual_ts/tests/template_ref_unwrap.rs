#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
    helpers::SHARED_PREAMBLE_DTS,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn generate_template_code(
    script: &str,
    template: &str,
    hoist_shared_preamble: bool,
) -> std::string::String {
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
            hoist_shared_preamble,
            ..Default::default()
        },
    )
    .code
    .into()
}

fn exact_line_count(source: &str, expected: &str) -> usize {
    source.lines().filter(|line| *line == expected).count()
}

#[test]
fn explicit_ref_literal_types_survive_template_unwrap() {
    let script = r#"import { ref } from 'vue'
const direction = ref<'rtl'>('rtl')
"#;
    let template = r#"<Child :direction="direction" />"#;
    let hoisted_code = generate_template_code(script, template, true);
    let embedded_code = generate_template_code(script, template, false);

    const WIDEN_HELPER: &str = "type __VizeWidenTemplateRef<T> = __VizeIsAny<T> extends true ? T : __VizeIsUnion<T> extends true ? T : T extends string ? string extends T ? string : T : T extends number ? number extends T ? number : T : T extends boolean ? boolean extends T ? boolean : T : T;";
    assert_eq!(
        exact_line_count(SHARED_PREAMBLE_DTS, WIDEN_HELPER),
        1,
        "the shared template-ref helper must preserve the exact widening oracle:\n{}",
        SHARED_PREAMBLE_DTS,
    );
    assert_eq!(
        exact_line_count(&embedded_code, &format!("    {WIDEN_HELPER}")),
        1,
        "the embedded template-ref helper must preserve the exact widening oracle:\n{embedded_code}",
    );
    for code in [hoisted_code, embedded_code] {
        assert_eq!(
            exact_line_count(&code, "  type __R_direction = typeof direction;"),
            1,
            "template expressions should capture the exact setup ref type:\n{code}",
        );
        assert_eq!(
            exact_line_count(
                &code,
                "    var direction: __U<__R_direction> = undefined as any;",
            ),
            1,
            "template expressions should emit the exact unwrapped ref shadow:\n{code}",
        );
    }
}

#[test]
fn authored_props_binding_does_not_collide_with_template_props_object() {
    let script = r#"defineProps<{ label?: string }>()
const { props, size } = useField()
"#;
    let code = generate_template_code(
        script,
        r#"<div>{{ props.title }} {{ size }} {{ label }}</div>"#,
        true,
    );

    assert_eq!(
        exact_line_count(&code, "  type __R_props = typeof props;"),
        1,
        "the authored maybe-ref binding must retain its exact template capture:\n{code}",
    );
    assert_eq!(
        exact_line_count(&code, "    var props: __UN<__R_props> = undefined as any;",),
        1,
        "the authored binding must remain the exact template-visible `props`:\n{code}",
    );
    assert_eq!(
        exact_line_count(
            &code,
            "  const __vize_props: __DefineProps<Props> = {} as __DefineProps<Props>;",
        ),
        1,
        "declared props must use the exact collision-free internal object:\n{code}",
    );
    assert_eq!(
        exact_line_count(&code, r#"  const label = __vize_props["label"];"#),
        1,
        "declared prop locals must resolve through the internal object:\n{code}",
    );
    assert_eq!(
        exact_line_count(
            &code,
            "  const props: __DefineProps<Props> = {} as __DefineProps<Props>;",
        ),
        0,
        "the generated object must not redeclare the authored binding:\n{code}",
    );
}
