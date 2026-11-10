//! Component hover ownership and editor-only public prop surfaces (#12461).
use std::path::Path;

use super::super::super::span_features::{
    CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_HOVER, retain_component_prop_hover_owner,
};
use super::super::{
    CONTENT_MAPPER_SPAN_FEATURES_ALL, CONTENT_MAPPER_SPAN_FEATURES_ATOM, ContentMapperSpanKind,
    generate_vue_content_mapper_transform,
};
use crate::batch::ContentMapperSpan;

#[test]
fn component_prop_navigation_is_the_only_hover_owner() {
    let generated =
        "  void __vize_props_nav_0.title;\n  (undefined as Check)({ \"title\": value });";
    let navigation = generated.find("title").unwrap();
    let check = generated.rfind("title").unwrap();
    let authored = 40;
    let mut spans = [
        ContentMapperSpan([
            navigation,
            "title".len(),
            authored,
            "title".len(),
            ContentMapperSpanKind::Verbatim as usize,
            CONTENT_MAPPER_SPAN_FEATURES_ALL,
        ]),
        ContentMapperSpan([
            check,
            "title".len(),
            authored,
            "title".len(),
            ContentMapperSpanKind::Atom as usize,
            CONTENT_MAPPER_SPAN_FEATURES_ATOM,
        ]),
    ];

    retain_component_prop_hover_owner(generated, &mut spans);

    assert_eq!(spans[0].0[5], CONTENT_MAPPER_SPAN_FEATURES_ALL);
    assert_eq!(
        spans[1].0[5],
        CONTENT_MAPPER_SPAN_FEATURES_ATOM & !CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_HOVER
    );
}

#[test]
fn editor_only_prop_surfaces_preserve_docs_without_changing_value_types() {
    let authored = r#"<script setup lang="ts">
interface Props {
  /** Visible label. */
  label?: string
}
const props = withDefaults(defineProps<Props>(), { label: 'ok' })
</script>"#;
    let result = generate_vue_content_mapper_transform(Path::new("Props.vue"), authored)
        .expect("content mapper transform");

    assert_eq!(
        result
            .text
            .matches(
                "): __WithDefaultsResult<_T, _D, _BKeys> & { readonly [K in keyof _T]: unknown };"
            )
            .count(),
        1,
        "{}",
        result.text,
    );
    // A documented identifier-safe runtime prop is spelled as an identifier so
    // hover shows `title`; undocumented keys keep their exact quoted bytes.
    let runtime = generate_vue_content_mapper_transform(
        Path::new("Runtime.vue"),
        "<script setup lang=\"ts\">const props = defineProps({\n  /** Visible title. */\n  'title': String,\n  'count': Number,\n})</script>",
    )
    .expect("runtime prop transform");
    for (member, count) in [
        ("  title?: string;", 1),
        ("  \"title\"?: string;", 0),
        ("  \"count\"?: number;", 1),
    ] {
        assert_eq!(
            runtime.text.matches(member).count(),
            count,
            "{member}\n{}",
            runtime.text
        );
    }
}
