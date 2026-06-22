use vize_croquis::{Analyzer, AnalyzerOptions};

use super::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets,
    generate_virtual_ts_with_offsets_and_checks,
};

fn standard_virtual_ts(script: &str, template: &str, options: &VirtualTsOptions) -> String {
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets(&analyzer.finish(), Some(script), Some(&root), 0, 0, options)
        .code
        .to_string()
}

fn legacy_virtual_ts(script: &str, template: &str, options: &VirtualTsOptions) -> String {
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets_and_checks(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        options,
        VirtualTsGenerationOptions {
            legacy_vue2: true,
            ..Default::default()
        },
    )
    .code
    .to_string()
}

#[test]
fn legacy_vue2_component_event_fallback_stays_permissive() {
    let script = r#"import VDatePicker from './VDatePicker.vue'
function updateDate(newDate: string) {
  void newDate
}
"#;
    let template = r#"<VDatePicker @input="updateDate" />"#;

    let standard = standard_virtual_ts(script, template, &VirtualTsOptions::default());
    assert!(
        standard.contains("? InputEvent :"),
        "standard component event fallback should stay DOM-typed:\n{standard}"
    );

    let legacy = legacy_virtual_ts(script, template, &VirtualTsOptions::default());
    assert!(
        legacy.contains("? any :") && !legacy.contains("? InputEvent :"),
        "legacy Vue 2 component event fallback should not force DOM payloads:\n{legacy}"
    );
}

#[test]
fn legacy_vue2_skips_external_component_prop_checks() {
    let script = "const width = 320\n";
    let template = r#"<VDatePicker :width="width" />"#;
    let options = VirtualTsOptions {
        auto_import_stubs: vec![
            "declare const VDatePicker: { new (): { $props: { mini?: boolean } } };".into(),
        ],
        external_template_bindings: vec!["VDatePicker".into()],
        ..Default::default()
    };

    let standard = standard_virtual_ts(script, template, &options);
    assert!(
        standard.contains("type __VDatePicker_Props_0"),
        "standard external component props should remain checkable:\n{standard}"
    );

    let legacy = legacy_virtual_ts(script, template, &options);
    assert!(
        !legacy.contains("type __VDatePicker_Props_0"),
        "legacy Vue 2 external component props should avoid Vuetify false positives:\n{legacy}"
    );
}

#[test]
fn legacy_vue2_unresolved_keyed_props_are_unchecked() {
    let script = r#"
type Props = { mini?: boolean } & { dense?: boolean }
defineProps<Props>()
"#;
    let template = r#"<v-select :width="width" :hide-details="hideDetails" />"#;

    let standard = standard_virtual_ts(script, template, &VirtualTsOptions::default());
    assert!(
        standard.contains("\"width\" satisfies keyof Props"),
        "standard keyed prop fallback should still catch unknown props:\n{standard}"
    );

    let legacy = legacy_virtual_ts(script, template, &VirtualTsOptions::default());
    assert!(
        legacy.contains("(props as Record<string, unknown>)[\"width\"]")
            && !legacy.contains("satisfies keyof Props"),
        "legacy Vue 2 keyed prop fallback should not reject Vuetify/mixin names:\n{legacy}"
    );
}
