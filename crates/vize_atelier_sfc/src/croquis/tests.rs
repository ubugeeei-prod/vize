use super::{
    SfcCroquisOptions, analyze_sfc_descriptor_resolved, analyze_sfc_descriptor_with_context,
};
use crate::{SfcParseOptions, parse_sfc};

#[test]
fn split_scripts_share_one_synthetic_script_offset_space() {
    let source = r#"<script lang="ts">
import PlainCard from './PlainCard.vue'
export interface PlainProps { label: string }
</script>
<script setup lang="ts" generic="T">
import { ref } from 'vue'
const count = ref(0)
</script>
"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let analysis =
        analyze_sfc_descriptor_with_context(&descriptor, None, SfcCroquisOptions::full());
    let script = analysis.script_content_ref().unwrap();

    let plain_import = analysis
        .croquis
        .import_statements
        .iter()
        .find(|span| script[span.start as usize..span.end as usize].contains("PlainCard"));
    let setup_import = analysis
        .croquis
        .import_statements
        .iter()
        .find(|span| script[span.start as usize..span.end as usize].contains("{ ref }"));

    assert!(plain_import.is_some());
    assert!(setup_import.is_some());
    assert!(analysis.croquis.bindings.contains("PlainCard"));
    assert!(analysis.croquis.bindings.contains("count"));

    let count_span = analysis.croquis.binding_spans.get("count").unwrap();
    assert_eq!(
        &script[count_span.0 as usize..count_span.1 as usize],
        "count"
    );
    assert_eq!(
        analysis.script_source_offset(&descriptor, count_span.0),
        source.find("count").unwrap() as u32,
    );
}

#[test]
fn resolved_props_do_not_leak_runtime_option_keys_into_bindings() {
    let source = r#"<script setup lang="ts">
const props = defineProps({
  count: {
    type: Number,
    required: true,
    default: 0,
    validator: (value: number) => value >= 0,
  },
  label: String,
})
void props
</script>
"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();

    let analysis = analyze_sfc_descriptor_resolved(
        &descriptor,
        None,
        SfcCroquisOptions::full(),
        false,
        false,
        "App.vue",
    );

    assert!(analysis.croquis.bindings.contains("count"));
    assert!(analysis.croquis.bindings.contains("label"));
    for option in ["type", "required", "default", "validator"] {
        assert!(
            !analysis.croquis.bindings.contains(option),
            "runtime prop option {option:?} is not a setup binding: {:#?}",
            analysis.croquis.bindings
        );
    }
}
