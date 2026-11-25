//! #13666: the pinned formatter's object layout policy reaches every script entry.
use serde::Deserialize;
use vize_glyph::{FormatOptions, ObjectWrap, format_script, format_sfc};
use vize_l0::{String, config::ConfigDocument, cstr};

#[derive(Deserialize)]
struct Fixtures {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    print_width: u32,
    source: String,
    preserve: String,
    collapse: String,
}

#[test]
fn object_wrap_matches_the_recorded_oxfmt_outputs_and_fixed_points() {
    let fixtures: Fixtures = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/formatter-object-wrap-13666/cases.json"
    ))
    .unwrap();
    for case in fixtures.cases {
        for (object_wrap, expected) in [
            (ObjectWrap::Preserve, &case.preserve),
            (ObjectWrap::Collapse, &case.collapse),
        ] {
            let options = FormatOptions {
                object_wrap,
                print_width: case.print_width,
                ..FormatOptions::default()
            };
            let actual = format_script(&case.source, &options).unwrap();
            assert_eq!(actual, *expected, "{} {object_wrap:?}", case.id);
            assert_eq!(format_script(&actual, &options).unwrap(), *expected);
            let source = cstr!("<script setup lang=\"ts\">\n{}</script>\n", case.source);
            let expected_sfc = cstr!("<script setup lang=\"ts\">\n{expected}</script>\n");
            let actual = format_sfc(&source, &options).unwrap();
            assert_eq!(actual.code, expected_sfc, "{} {object_wrap:?}", case.id);
            let second = format_sfc(&actual.code, &options).unwrap();
            assert_eq!(second.code, expected_sfc);
            assert!(!second.changed);
        }
        let defaults = FormatOptions {
            print_width: case.print_width,
            ..FormatOptions::default()
        };
        assert_eq!(
            format_script(&case.source, &defaults).unwrap(),
            case.preserve
        );
    }
}

#[test]
fn configuration_validates_and_retains_the_object_wrap_policy() {
    for (value, expected) in [
        ("preserve", "const options = {\n  a: 1,\n};\n"),
        ("collapse", "const options = { a: 1 };\n"),
    ] {
        let json = cstr!("{{\"formatter\":{{\"objectWrap\":\"{value}\"}}}}");
        let document: ConfigDocument = serde_json::from_str(&json).unwrap();
        let options = FormatOptions::from_profile(document.formatter_profile()).unwrap();
        assert_eq!(
            format_script("const options = {\n a: 1,\n};\n", &options).unwrap(),
            expected,
        );
    }
    assert!(
        serde_json::from_str::<ConfigDocument>(r#"{"formatter":{"objectWrap":"auto"}}"#).is_err()
    );
}
