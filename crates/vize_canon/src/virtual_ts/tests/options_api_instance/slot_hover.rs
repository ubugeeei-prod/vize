use crate::virtual_ts::{VirtualTsGenerationOptions, generate_virtual_ts_with_offsets_and_checks};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn projection_is_editor_only() {
    let script = r#"import { defineComponent, type SlotsType } from 'vue'
export default defineComponent({
  slots: Object as SlotsType<{
    /** Row slot. */
    item: { row: number }
  }>,
})
"#;
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_plain(script);
    let summary = analyzer.finish();
    let batch = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
        VirtualTsGenerationOptions {
            options_api: true,
            preserve_authored_component: true,
            ..Default::default()
        },
    );
    let document_editor = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
        VirtualTsGenerationOptions {
            options_api: true,
            ..Default::default()
        },
    );
    let content_mapper = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
        VirtualTsGenerationOptions {
            options_api: true,
            component_name: Some("OptionsChild"),
            ..Default::default()
        },
    );

    assert_eq!(batch.code.matches("type $Slots = {};").count(), 1);
    let documented_slot =
        "type $Slots = {\n  /** Row slot. */\n  item?: (props: { row: number }) => any;\n};";
    for editor in [document_editor, content_mapper] {
        assert_eq!(editor.code.matches(documented_slot).count(), 1);
    }
}
