use super::{ProductCaptureSource, product_capture_value};
use davinci_test_support::schema;
use serde_json::Value;
use vize_l0::dump::capture::{CaptureOutcome, CaptureSink, StageCapture, StageCaptureUnavailable};
use vize_l0::{Span, String, level::Level};
use vize_l2::dump::NativeDumpError;

fn source() -> ProductCaptureSource<'static> {
    ProductCaptureSource {
        path: None,
        container: "raw-template",
        authored_syntax: "vue-template",
        compiled_syntax: "vue-template",
        template_span: None,
    }
}

fn schema() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../docs/davinci/plan/product-stage-feed.schema.json"
    ))
    .unwrap()
}

#[test]
fn successful_product_feed_keeps_exact_existing_bytes_and_omits_failures() {
    let mut capture = StageCapture::new("dom");
    capture.page(Level::L1, "parse", || String::from("<p>λ</p>"));
    capture.finish(|| CaptureOutcome::Accepted);
    let value = product_capture_value("vize-dump", source(), &capture);
    assert_eq!(schema::validate(&schema(), &value, "$"), Ok(()));
    assert!(value.get("unavailable").is_none());
    assert_eq!(
        serde_json::to_string(&value).unwrap(),
        "{\"command\":\"vize-dump\",\"observed\":{\"remarks\":false,\"timings\":false},\"options\":[],\"outcome\":{\"kind\":\"accepted\",\"reason\":null},\"pages\":[{\"level\":\"l1\",\"step\":\"parse\",\"text\":\"<p>λ</p>\"}],\"remarks\":[],\"schema_version\":2,\"source\":{\"authored_syntax\":\"vue-template\",\"compiled_syntax\":\"vue-template\",\"container\":\"raw-template\",\"path\":null,\"template_span\":null},\"target\":\"dom\",\"timings\":[]}"
    );
}

#[test]
fn accepted_product_preserves_exact_typed_refusal_beside_later_valid_pages() {
    let mut capture = StageCapture::new("dom");
    capture.try_page(Level::L2, "lower", || {
        Err::<String, _>(NativeDumpError::JsBindingUnsupported {
            span: Span::new(4, 9),
        })
    });
    capture.page(Level::L4, "emit", || String::from("module"));
    capture.finish(|| CaptureOutcome::Accepted);
    let value = product_capture_value("vize-dump", source(), &capture);
    assert_eq!(schema::validate(&schema(), &value, "$"), Ok(()));
    assert_eq!(value["pages"].as_array().unwrap().len(), 1);
    assert_eq!(value["pages"][0]["level"], "l4");
    assert_eq!(
        value["unavailable"],
        serde_json::json!([{
            "level": "l2", "step": "lower",
            "reason": "native binding dump unsupported at bytes 4..9",
        }])
    );
}

#[test]
fn nonaccepted_host_strips_failed_inspections_even_if_a_producer_forgot_finish() {
    for outcome in [
        CaptureOutcome::Pending,
        CaptureOutcome::Legacy(String::from("compatibility selected")),
        CaptureOutcome::Unavailable(String::from("no template")),
        CaptureOutcome::Rejected(String::from("invalid artifact")),
    ] {
        let mut capture = StageCapture::new("dom");
        capture.unavailable.push(StageCaptureUnavailable {
            level: Level::L2,
            step: "lower",
            reason: String::from("not representable"),
        });
        capture.outcome = outcome;
        let value = product_capture_value("vize-dump", source(), &capture);
        assert!(value.get("unavailable").is_none());
        assert_eq!(schema::validate(&schema(), &value, "$"), Ok(()));
    }
}

#[test]
fn optional_failed_inspection_schema_refuses_page_text_and_missing_reason() {
    let mut capture = StageCapture::new("dom");
    capture.finish(|| CaptureOutcome::Accepted);
    let mut value = product_capture_value("vize-dump", source(), &capture);
    value["unavailable"] = serde_json::json!([{
        "level": "l2", "step": "lower", "text": "fake page",
    }]);
    assert!(schema::validate(&schema(), &value, "$").is_err());
}
