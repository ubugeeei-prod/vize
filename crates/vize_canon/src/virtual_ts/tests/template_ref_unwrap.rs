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

#[test]
fn explicit_ref_literal_types_survive_template_unwrap() {
    let script = r#"import { ref } from 'vue'
const direction = ref<'rtl'>('rtl')
"#;
    let template = r#"<Child :direction="direction" />"#;
    let hoisted_code = generate_template_code(script, template, true);
    let embedded_code = generate_template_code(script, template, false);

    assert!(
        SHARED_PREAMBLE_DTS.contains("T extends string ? string extends T ? string : T"),
        "the shared template-ref helper must preserve explicit string literals:\n{}",
        SHARED_PREAMBLE_DTS
    );
    assert!(
        embedded_code.contains("T extends string ? string extends T ? string : T"),
        "the embedded template-ref helper must preserve explicit string literals:\n{embedded_code}"
    );
    for code in [hoisted_code, embedded_code] {
        assert!(
            code.contains("type __R_direction = typeof direction;"),
            "template expressions should capture the setup ref type:\n{code}"
        );
        assert!(
            code.contains("var direction: __U<__R_direction> = undefined as any;"),
            "template expressions should shadow the ref through the unwrap helper:\n{code}"
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

    assert!(
        code.contains("type __R_props = typeof props;"),
        "the authored maybe-ref binding must retain its template capture:\n{code}"
    );
    assert!(
        code.contains("var props: __U<__R_props> = undefined as any;"),
        "the authored binding must remain the template-visible `props`:\n{code}"
    );
    assert!(
        code.contains("const __vize_props:")
            && code.contains(r#"const label = __vize_props["label"];"#),
        "declared props must use a collision-free internal object:\n{code}"
    );
    assert!(
        !code.contains("const props: __DefineProps"),
        "the generated object must not redeclare the authored binding:\n{code}"
    );
}
