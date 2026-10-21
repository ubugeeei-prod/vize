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
        "const __vize_events_resolved_",
        "var __vize_ref_",
    ] {
        let mut count = 0;
        for (start, _) in result.text.as_str().match_indices(marker) {
            let tail = &result.text.as_str()[start + marker.len()..];
            let probe = tail
                .split_once("});\n")
                .map(|(probe, _)| probe)
                .unwrap_or_else(|| panic!("unterminated {marker} probe:\n{}", result.text));
            assert!(probe.matches("\"class\":").count() <= 1, "{probe}");
            assert!(probe.matches("\"style\":").count() <= 1, "{probe}");
            count += 1;
        }
        assert!(count > 0, "missing {marker} probe:\n{}", result.text);
    }
}
