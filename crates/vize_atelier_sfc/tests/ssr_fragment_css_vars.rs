//! #7892: the original whole SFC and independent root-shape controls.
#![expect(
    clippy::expect_used,
    reason = "complete public fixture packets must serialize or fail"
)]

use serde_json::{Value, json};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileResult, SfcParseOptions, TemplateCompileOptions, compile_sfc,
    parse_sfc,
};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/ssr-fragment-css-vars/Reported.vue.txt"
);
const CONTROLS: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/ssr-fragment-css-vars/cases.json");

/// The SSR output without the line that marks the component as Vapor.
fn strip_vapor_marker(code: &str) -> String {
    code.replace("  __vapor: true,\n", "")
        .replace("_sfc_main.__vapor = true\n", "")
}

fn without_fragment_anchors(code: &str) -> String {
    code.replace("<!--[-->", "").replace("<!--]-->", "")
}

fn compile(source: &str, target: &str) -> SfcCompileResult {
    let parse = SfcParseOptions {
        filename: "App.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("whole authored SFC parses");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            vapor: target == "vapor-ssr",
            scope_id: Some("cssvars".into()),
            template: TemplateCompileOptions {
                ssr: target != "client",
                is_prod: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("whole authored SFC compiles");
    assert!(result.errors.is_empty(), "{target}: {:?}", result.errors);
    assert!(result.map.is_none());
    assert!(result.macro_artifacts.is_empty());
    assert!(
        result.warnings.is_empty(),
        "{target}: {:?}",
        result.warnings
    );
    result
}

#[test]
fn original_fragment_css_variables_keep_complete_public_result_packets() {
    let mut fixtures = vec![json!({"name":"reported", "source":ORIGINAL, "cssRoots":2})];
    for mut fixture in serde_json::from_str::<Vec<Value>>(CONTROLS).expect("controls") {
        let template = fixture["template"].as_str().expect("control template");
        let start = ORIGINAL.find("<template>").expect("original open") + "<template>".len();
        let end = ORIGINAL.find("</template>").expect("original close");
        let mut source = [
            ORIGINAL.get(..start).expect("prefix"),
            template,
            ORIGINAL.get(end..).expect("suffix"),
        ]
        .concat();
        if fixture["noCss"] == true {
            source = source.replace("v-bind(color)", "red");
        }
        fixture["source"] = source.into();
        fixtures.push(fixture);
    }
    for fixture in &mut fixtures {
        let source = fixture["source"].as_str().expect("whole input");
        let ssr = compile(source, "ssr");
        let client = compile(source, "client");
        // A Vapor component renders the same server markup apart from the
        // fragment anchors Vapor hydration claims (around Suspense slots), and
        // is marked to hydrate as Vapor.
        let vapor = compile(source, "vapor-ssr");
        assert_eq!(vapor.code.matches("__vapor").count(), 1, "{}", vapor.code);
        assert_eq!(
            without_fragment_anchors(&strip_vapor_marker(&vapor.code)),
            without_fragment_anchors(&ssr.code)
        );
        assert_eq!(vapor.css, ssr.css);
        assert_eq!(
            serde_json::to_value(&vapor.bindings).expect("complete Vapor SSR bindings"),
            serde_json::to_value(&ssr.bindings).expect("complete SSR bindings")
        );
        if fixture["name"] == "reported" {
            assert_eq!(ssr.code.matches("_ssrRenderAttrs(_cssVars)").count(), 2);
        }
        fixture["ssr"] = serde_json::to_value(ssr).expect("complete SSR result");
        fixture["client"] = serde_json::to_value(client).expect("complete client result");
        fixture["vaporSsr"] = serde_json::to_value(vapor).expect("complete Vapor SSR result");
    }
    if let Some(path) = std::env::var_os("VIZE_FRAGMENT_CSS_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec(&json!({"cases":fixtures})).expect("complete capture serializes"),
        )
        .expect("capture writes");
    }
}
