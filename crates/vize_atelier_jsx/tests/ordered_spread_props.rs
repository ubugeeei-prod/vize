//! Full output contract for ordered JSX object sources (#10458).
#![expect(
    clippy::expect_used,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "integration fixtures pin complete compiler packets"
)]

use serde_json::{Value, json};
use vize_atelier_jsx::{JsxLang, VaporCompileOptions, compile_to_vapor};
use vize_l0::Allocator;

#[test]
fn ordered_object_source_modules_and_templates_are_complete() {
    let corpus: Value =
        serde_json::from_str(include_str!("vue_jsx_vapor_compat/fixtures/corpus.json"))
            .expect("semantic corpus");
    let ids = [
        "props/spread-before-static",
        "props/spread-after-static",
        "props/ordered-spread-chain",
        "props/spread-class-style",
        "events/spread-listener-order",
        "props/ordered-spread-reads",
    ];
    let cases = corpus["cases"].as_array().expect("cases");
    let mut packets = Vec::new();
    for id in ids {
        let case = cases
            .iter()
            .find(|case| case["id"] == id)
            .expect("registered semantic case");
        let source = case["source"].as_str().expect("authored source");
        let allocator = Allocator::new();
        let output = compile_to_vapor(
            &allocator,
            source,
            JsxLang::Jsx,
            VaporCompileOptions::default(),
        );
        assert!(
            output.diagnostics.is_empty(),
            "{id}: {:?}",
            output.diagnostics
        );
        assert_eq!(output.components.len(), 1, "{id}");
        let component = output.components.first().expect("one authored component");
        packets.push(json!({"id": id, "source": source, "code": component.code.as_str(), "templates": component.templates.iter().map(|template| template.as_str()).collect::<Vec<_>>(), "componentName": component.component_name.as_deref(), "scopedStyle": component.scoped_style.is_some(), "diagnostics": []}));
    }
    assert_eq!(packets.len(), ids.len());
    insta::assert_snapshot!(
        serde_json::to_string_pretty(&packets).expect("complete compiler packets")
    );
}
