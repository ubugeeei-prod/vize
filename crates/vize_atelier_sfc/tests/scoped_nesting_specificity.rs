//! Whole SFC outputs feed an independent Chromium cascade and pixel oracle.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "complete source-built CSS observations and immutable fixture contracts"
)]
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[test]
fn selector_list_nesting_keeps_full_scoped_modules_and_cascade() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/scoped_nesting_specificity.json"))
            .expect("pinned full source and reference CSS");
    let cases = pack["cases"].as_array().expect("all cases");
    assert_eq!(cases.len(), 31);
    let mut captures = Vec::new();
    for case in cases {
        let source = case["source"].as_str().expect("whole SFC");
        let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("valid SFC");
        let pipeline = vize_atelier_sfc::vite_plugin::scope_css_for_pipeline(
            case["css"].as_str().expect("authored CSS"),
            "data-v-nesting",
        );
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
