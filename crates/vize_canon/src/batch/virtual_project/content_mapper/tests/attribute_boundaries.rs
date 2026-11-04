use std::path::Path;

use crate::batch::generate_vue_content_mapper_transform;

#[test]
fn adjacent_quoted_attributes_keep_the_complete_spaced_projection() {
    let source = "<script setup lang=\"ts\">\nconst a = 1;\n</script>\n\n<template>\n  <div :title=\"`x`\"status=\"y\">{{ a }}</div>\n</template>\n";
    let boundary = source.find("status").expect("second authored attribute");
    let mut spaced = source.to_owned();
    spaced.insert(boundary, ' ');
    let path = Path::new("Gap.vue");
    let actual = generate_vue_content_mapper_transform(path, source).expect("adjacent transform");
    let mut expected =
        generate_vue_content_mapper_transform(path, &spaced).expect("spaced transform");
    assert!(actual.diagnostics.is_empty(), "{:?}", actual.diagnostics);
    assert!(
        expected.diagnostics.is_empty(),
        "{:?}",
        expected.diagnostics
    );
    for span in &mut expected.mappings {
        let start = span.0[2];
        let end = start + span.0[3];
        let authored = |offset: usize| offset - usize::from(offset > boundary);
        span.0[2] = authored(start);
        span.0[3] = authored(end) - authored(start);
    }
    assert_eq!(
        serde_json::to_value(actual).expect("whole adjacent packet"),
        serde_json::to_value(expected).expect("whole spaced packet")
    );
}
