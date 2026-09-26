//! Equal expression bytes at different positions retain separate map anchors.

use super::support::{Lane, compile_on, decode};

#[test]
fn repeated_expressions_keep_authored_maps_in_each_scope() {
    for source in [
        r#"<main><p :id="item.value"></p><p :title="item.value"></p></main>"#,
        r#"<main><p :id="item.value"></p><p v-for="item in items" :title="item.value"></p><p :title="item.value"></p></main>"#,
        r#"<main><button @click="save" :title="label"></button><button @click="save" :title="label"></button></main>"#,
    ] {
        let selected = compile_on(source, true, Lane::Selected);
        let retained = compile_on(source, true, Lane::Legacy);
        let selected_map: serde_json::Value =
            serde_json::from_str(selected.map.as_deref().expect("selected map")).expect("map JSON");
        let retained_map: serde_json::Value =
            serde_json::from_str(retained.map.as_deref().expect("retained map")).expect("map JSON");
        // Native and retained variable numbering may differ. Every mapping's
        // actual generated/source coordinates and named anchors must agree.
        assert_eq!(selected_map["names"], retained_map["names"], "{source}");
        let selected_segments = decode(selected_map["mappings"].as_str().expect("mappings"));
        let retained_segments = decode(retained_map["mappings"].as_str().expect("mappings"));
        assert_eq!(selected_segments, retained_segments, "{source}");
        let expression = if source.contains("item.value") {
            "item.value"
        } else {
            "label"
        };
        for (offset, _) in source.match_indices(expression) {
            assert!(
                selected_segments
                    .iter()
                    .any(|segment| segment.2 == 0 && segment.3 == offset as i64),
                "missing authored position {offset}: {source}"
            );
        }
    }
}
