//! Content Mapper inference probes must isolate repeatable `class` and `style`
//! attributes instead of emitting duplicate object keys (#10184).

use std::path::Path;

use crate::batch::generate_vue_content_mapper_transform;

#[test]
fn content_mapper_isolates_duplicate_class_and_style_attribute_probes() {
    let source = include_str!(
        "../../../../../../../tests/_fixtures/differential/typechecker/content-mapper-duplicate-attributes-10184/src/App.vue.txt"
    );
    let result =
        generate_vue_content_mapper_transform(Path::new("src/App.vue"), source).expect("transform");

    for marker in [
        "const __vize_emit_props_",
        "const __vize_resolved_events_",
        "var __vize_ref_",
    ] {
        let probe = result
            .text
            .split_once(marker)
            .and_then(|(_, tail)| tail.split_once("});\n"))
            .map(|(probe, _)| probe)
            .unwrap_or_else(|| panic!("missing {marker} probe:\n{}", result.text));
        assert_eq!(probe.matches("\"class\":").count(), 1, "{probe}");
        assert_eq!(probe.matches("\"style\":").count(), 1, "{probe}");
    }
}
