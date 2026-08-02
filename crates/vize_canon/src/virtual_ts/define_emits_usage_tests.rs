use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
};
use vize_carton::{String, config::VueVersion};
use vize_croquis::{Analyzer, AnalyzerOptions};

const EMIT_ANCHOR_COMMENT: &str = "// Reference defineEmits result used through template $emit";
const EMIT_ANCHOR: &str =
    "\n  // Reference defineEmits result used through template $emit\n  void emit;\n";

#[test]
fn template_dollar_emit_references_define_emits_result_for_vue2_and_vue3() {
    let script = r#"const emit = defineEmits<{
  (event: 'click'): void
}>()
"#;

    for dialect in [VueVersion::V2_7, VueVersion::V3] {
        let code = generate_setup(script, r#"<button @click="$emit('click')" />"#, dialect);
        assert_eq!(
            code.matches(EMIT_ANCHOR).count(),
            1,
            "expected one exact defineEmits anchor for {dialect:?}:\n{code}"
        );
    }
}

#[test]
fn template_dollar_emit_references_renamed_define_emits_result() {
    let code = generate_setup(
        r#"const dispatch = defineEmits<{
  (event: 'save'): void
}>()
"#,
        r#"<button @click="$emit('save')" />"#,
        VueVersion::V3,
    );

    assert!(code.contains(
        "\n  // Reference defineEmits result used through template $emit\n  void dispatch;\n"
    ));
    assert!(!code.contains("void emit;"));
}

#[test]
fn direct_emit_usage_does_not_need_the_dollar_emit_anchor() {
    let template_usage = generate_setup(
        "const emit = defineEmits<{ click: [] }>()",
        r#"<button @click="emit('click')" />"#,
        VueVersion::V3,
    );
    assert!(!template_usage.contains(EMIT_ANCHOR_COMMENT));
    assert!(template_usage.contains("void emit;"));

    let script_usage = generate_setup(
        r#"const emit = defineEmits<{ click: [] }>()
emit('click')
"#,
        "<button />",
        VueVersion::V3,
    );
    assert!(!script_usage.contains(EMIT_ANCHOR_COMMENT));
    assert!(!script_usage.contains("void emit;"));
}

#[test]
fn unused_define_emits_result_stays_unanchored() {
    let code = generate_setup(
        "const emit = defineEmits<{ click: [] }>()",
        "<button />",
        VueVersion::V3,
    );

    assert!(!code.contains(EMIT_ANCHOR_COMMENT));
    assert!(!code.contains("void emit;"));
}

#[test]
fn options_api_dollar_emit_does_not_consume_an_unrelated_binding() {
    let script = r#"const emit = 1
export default {
  emits: ['click'],
}
"#;
    let template = r#"<button @click="$emit('click')" />"#;
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let code = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            preserve_unused_diagnostics: true,
            options_api: true,
            ..Default::default()
        },
    )
    .code;

    assert!(!code.contains(EMIT_ANCHOR_COMMENT));
    assert!(!code.contains("void emit;"));
}

fn generate_setup(script: &str, template: &str, dialect: VueVersion) -> String {
    let allocator = vize_carton::Bump::new();
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
            dialect,
            preserve_unused_diagnostics: true,
            ..Default::default()
        },
    )
    .code
}
