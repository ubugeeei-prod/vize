//! Whole SFC outputs feed an independent Chromium cascade and pixel oracle.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "complete source-built CSS observations and immutable fixture contracts"
)]
use vize_atelier_sfc::{
    CssCompileOptions, SfcCompileOptions, SfcParseOptions, compile_css, compile_sfc, parse_sfc,
};

#[test]
fn selector_list_nesting_keeps_full_scoped_modules_and_cascade() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/scoped_nesting_specificity.json"))
            .expect("pinned full source and reference CSS");
    let cases = pack["cases"].as_array().expect("all cases");
    assert_eq!(cases.len(), 32);
    let mut captures = Vec::new();
    for case in cases {
        let source = case["source"].as_str().expect("whole SFC");
        let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("valid SFC");
        let pipeline = vize_atelier_sfc::vite_plugin::scope_css_for_pipeline(
            case["css"].as_str().expect("authored CSS"),
            "data-v-nesting",
        );
        if case["id"].as_str() == Some("nested-list-sibling-bare-deep") {
            let reference_css = case["referenceCss"].as_str().expect("semantic CSS");
            assert_eq!(pipeline.as_str(), reference_css, "{}", case["id"]);
            for minify in [false, true] {
                let reference = compile_css(
                    reference_css,
                    &CssCompileOptions {
                        minify,
                        ..Default::default()
                    },
                );
                assert!(reference.errors.is_empty(), "{:?}", reference.errors);
                let actual = compile_css(
                    case["css"].as_str().expect("authored CSS"),
                    &CssCompileOptions {
                        scope_id: Some("data-v-nesting".into()),
                        scoped: true,
                        minify,
                        ..Default::default()
                    },
                );
                assert_eq!(
                    serde_json::to_value(actual).expect("complete public CSS result"),
                    serde_json::to_value(reference).expect("complete reference CSS result"),
                    "{} / public compileCss minify={minify}",
                    case["id"]
                );
            }
        }
        for backend in ["dom", "ssr", "vapor"] {
            let mut options = SfcCompileOptions {
                scope_id: Some("nesting".into()),
                vapor: backend == "vapor",
                ..Default::default()
            };
            options.template.ssr = backend == "ssr";
            let output = compile_sfc(&descriptor, options).expect("complete SFC compile");
            assert!(
                output.errors.is_empty(),
                "{}: {:?}",
                case["id"],
                output.errors
            );
            assert!(
                output.warnings.is_empty(),
                "{}: {:?}",
                case["id"],
                output.warnings
            );
            assert_eq!(
                output.css.as_deref(),
                Some(pipeline.as_str()),
                "{} / {backend}",
                case["id"]
            );
            captures.push(serde_json::json!({
                "id": case["id"], "backend": backend, "source": source,
                "code": output.code.as_str(), "css": output.css.as_deref(),
                "errors": [], "warnings": [],
            }));
        }
    }
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/scoped-list-nesting-10606/Layout.vue.txt"
    );
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("original whole SFC");
    let mut options = SfcCompileOptions {
        parse,
        ..Default::default()
    };
    options.template.ssr = true;
    let output = compile_sfc(&descriptor, options).expect("original complete SSR module");
    assert!(output.errors.is_empty());
    assert!(output.warnings.is_empty());
    assert_eq!(
        output.code.as_str(),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/scoped-list-nesting-10606/module.expected.txt"
        )
    );
    assert_eq!(
        output.css.as_deref(),
        Some(include_str!(
            "../../../tests/_fixtures/differential/compiler/scoped-list-nesting-10606/style.expected.css"
        ))
    );
    if let Some(path) = std::env::var_os("VIZE_SCOPED_NESTING_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&captures).expect("all complete DTOs"),
        )
        .expect("write current source capture");
    }
}
