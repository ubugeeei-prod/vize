use std::path::Path;

use vize_l0::cstr;

use crate::batch::generate_vue_content_mapper_transform;

#[test]
fn adjacent_quoted_attributes_keep_the_complete_spaced_projection() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/recovered-attribute-boundary-11731/Gap.vue.txt"
    ));
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
    let expression = source.find("{{ a }}").expect("authored interpolation") + 3;
    let original_marker = cstr!("// @vize-map: expr -> {}:{}", expression, expression + 1);
    let spaced_marker = cstr!(
        "// @vize-map: expr -> {}:{}",
        expression + 1,
        expression + 2
    );
    assert_eq!(expected.text.matches(spaced_marker.as_str()).count(), 1);
    expected.text = expected
        .text
        .replace(spaced_marker.as_str(), original_marker.as_str())
        .into();
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
