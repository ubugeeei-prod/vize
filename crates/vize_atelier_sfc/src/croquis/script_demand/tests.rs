use crate::croquis::{
    SfcCroquisOptions, analyze_sfc_descriptor_with_context, analyze_sfc_descriptor_with_occurrences,
};
use crate::{SfcParseOptions, parse_sfc};

#[test]
fn specialized_script_results_preserve_whole_outputs_and_empty_disabled_boundaries() {
    for source in [
        "<style>.a{color:red}</style>",
        "<script>export const plain=1</script>",
        "<script setup>const value=1; const read=value</script>",
        "<script>export const plain=1</script><script setup>const value=plain; const read=value</script>",
    ] {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let mut disabled = SfcCroquisOptions::full();
        disabled.analyzer_options.analyze_script = false;
        for options in [
            SfcCroquisOptions::full(),
            SfcCroquisOptions::for_compile(),
            SfcCroquisOptions::lint_demand(),
            SfcCroquisOptions::full().without_script_merge(),
            disabled,
        ] {
            let ordinary = analyze_sfc_descriptor_with_context(&descriptor, None, options);
            let (captured, packet) =
                analyze_sfc_descriptor_with_occurrences(&descriptor, None, options);
            assert_eq!(ordinary.croquis.to_vir(), captured.croquis.to_vir());
            assert_eq!(
                serde_json::to_value(ordinary.croquis.semantic_snapshot()).unwrap(),
                serde_json::to_value(captured.croquis.semantic_snapshot()).unwrap(),
            );
            assert_eq!(ordinary.script_content, captured.script_content);
            assert_eq!(ordinary.script_offset, captured.script_offset);
            if !options.analyzer_options.analyze_script {
                assert!(packet.is_none());
            } else if descriptor.script.is_none() && descriptor.script_setup.is_none() {
                let packet = packet.expect("no script retains the complete empty packet");
                assert_eq!(packet.bindings().count(), 0);
                assert!(packet.occurrences().is_empty());
            }
        }
    }
}
