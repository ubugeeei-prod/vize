use vize_l0::config::VueVersion;
use vize_l0::{Allocator, ErrorCode, TemplateSyntaxMode};
use vize_l1::dialect::LegacyCaps;
use vize_l1::markup::{SurfaceChild, SurfaceParseOptions, parse_with_options_and_caps};

#[test]
fn legacy_caps_retain_triple_mustache_as_raw_html() {
    let allocator = Allocator::new();
    let (tree, errors) = parse_with_options_and_caps(
        &allocator,
        "{{{ html }}}",
        SurfaceParseOptions::default(),
        LegacyCaps::for_version(VueVersion::V1),
    );
    assert!(errors.is_empty(), "{errors:?}");
    let [SurfaceChild::Interpolation(interpolation)] = tree.children.as_slice() else {
        panic!("expected one interpolation: {:#?}", tree.children);
    };
    assert!(interpolation.is_raw_html());
    assert_eq!(interpolation.open.text, "{{{");
    assert_eq!(interpolation.content.text, " html ");
    assert_eq!(interpolation.close.text, "}}}");
}

#[test]
fn quirks_accepts_adjacent_attributes_without_changing_standard_diagnostics() {
    let allocator = Allocator::new();
    let source = r#"<div id="a"class="b"></div>"#;
    let (_, standard) = parse_with_options_and_caps(
        &allocator,
        source,
        SurfaceParseOptions::default(),
        LegacyCaps::VUE3,
    );
    assert!(
        standard
            .iter()
            .any(|error| { error.code == ErrorCode::MissingWhitespaceBetweenAttributes })
    );

    let (_, quirks) = parse_with_options_and_caps(
        &allocator,
        source,
        SurfaceParseOptions {
            syntax: TemplateSyntaxMode::Quirks,
            ..SurfaceParseOptions::default()
        },
        LegacyCaps::VUE3,
    );
    assert!(quirks.is_empty(), "{quirks:?}");
}
