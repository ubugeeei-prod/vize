//! Original #7942 full source and authored selector/declaration contracts.
use serde_json::Value;
use vize_atelier_sfc::{
    CssCompileOptions, SfcCompileOptions, SfcParseOptions, compile_css, compile_sfc, parse_sfc,
};

const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/"
);
const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/App.vue"
);
const ORIGINAL_CSS: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/App.pipeline.css"
);

#[test]
fn all_original_and_anchor_controls_keep_complete_css_results() {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/scoped-slotted-where-7942/cases.json"
    ))
    .unwrap();
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    for case in cases {
        let source = std::fs::read_to_string(
            std::path::Path::new(ROOT).join(case["source"].as_str().unwrap()),
        )
        .unwrap();
        let expected = std::fs::read_to_string(
            std::path::Path::new(ROOT).join(case["pipeline"].as_str().unwrap()),
        )
        .unwrap();
        for minify in [false, true] {
            let reference = compile_css(
                &expected,
                &CssCompileOptions {
                    minify,
                    ..Default::default()
                },
            );
            assert!(reference.errors.is_empty(), "{:?}", reference.errors);
            let actual = compile_css(
                &source,
                &CssCompileOptions {
                    scoped: true,
                    scope_id: Some("data-v-1".into()),
                    minify,
                    ..Default::default()
                },
            );
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::to_value(reference).unwrap(),
                "{} minify={minify}",
                case["id"]
            );
        }
    }
}

#[test]
fn original_whole_sfc_preserves_css_and_complete_repeat_output() {
    let descriptor = parse_sfc(ORIGINAL, SfcParseOptions::default()).unwrap();
    assert_eq!(descriptor.styles.len(), 1);
    assert_eq!(
        descriptor.styles[0].content,
        ORIGINAL
            .split("<style scoped>")
            .nth(1)
            .unwrap()
            .split("</style>")
            .next()
            .unwrap()
    );
    for trim in [false, true] {
        let options = SfcCompileOptions {
            scope_id: Some("1".into()),
            style: vize_atelier_sfc::StyleCompileOptions {
                trim,
                ..Default::default()
            },
            ..Default::default()
        };
        let actual = compile_sfc(&descriptor, options.clone()).unwrap();
        assert_eq!(
            actual.css.as_deref(),
            Some(if trim {
                ORIGINAL_CSS.trim()
            } else {
                ORIGINAL_CSS
            })
        );
        assert!(actual.errors.is_empty(), "{:?}", actual.errors);
        assert!(actual.warnings.is_empty(), "{:?}", actual.warnings);
        let repeat = compile_sfc(&descriptor, options).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(repeat).unwrap()
        );
    }
}
