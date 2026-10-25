//! Empty generic presence selects a functional public contract without a plan.

use std::path::Path;

use vize_canon::generate_vue_content_mapper_transform;

#[test]
fn empty_generic_markers_keep_identity_separate_from_type_parameters() {
    for (attribute, functional) in [
        ("", false),
        (" generic", true),
        (" generic=\"\"", true),
        (" generic=''", true),
        (" generic=\" \t \"", true),
        (" generic=\" \r\n \"", true),
        (" generic=\"\u{a0}\"", true),
        (" generic=\"T\"", true),
    ] {
        let source = format!(
            "<!-- 日本 🧭 --><script setup lang=\"ts\"{attribute}></script><template><a/><img/></template>"
        );
        let result = generate_vue_content_mapper_transform(Path::new("Fragment.vue"), &source)
            .expect("complete component projection");
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(
            result.text.contains("type __VizeGenericComponent ="),
            functional,
            "attribute presence must preserve the public callable identity: {attribute:?}"
        );
        assert!(!result.text.contains("function __setup<>()"));
        assert!(!result.text.contains("Props<>"), "{attribute:?}");
        assert!(!result.text.contains("__vizeCheck: <>"), "{attribute:?}");
    }
}
