//! Versioned host serialization of stages observed during product compilation.
//! The same constructor serves the CLI and WASM boundaries; level crates do
//! not serialize a stage handoff.

use serde::Serialize;
use serde_json::Value;
use vize_l0::{
    Span,
    dump::capture::{CaptureArgValue, CaptureOutcome, StageCapture},
};

pub const PRODUCT_STAGE_FEED_VERSION: u32 = 2;

/// Authored input and the syntax actually handed to the compiler.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ProductCaptureSource<'a> {
    pub path: Option<&'a str>,
    /// File container, for example a Vue SFC or a raw template.
    pub container: &'a str,
    /// Authored template grammar before a preprocessor runs.
    pub authored_syntax: &'a str,
    pub compiled_syntax: &'a str,
    /// Authored byte span of a Vue SFC template; absent for raw templates.
    pub template_span: Option<Span>,
}

#[derive(Serialize)]
struct Outcome<'a> {
    kind: &'static str,
    reason: Option<&'a str>,
}

#[derive(Serialize)]
struct Observed {
    timings: bool,
    remarks: bool,
}

#[derive(Serialize)]
struct OptionValue<'a> {
    name: &'a str,
    value: &'a str,
}

#[derive(Serialize)]
struct Page<'a> {
    level: &'static str,
    step: &'a str,
    text: &'a str,
}

#[derive(Serialize)]
struct Unavailable<'a> {
    level: &'static str,
    step: &'a str,
    reason: &'a str,
}

#[derive(Serialize)]
struct Timing<'a> {
    level: &'static str,
    step: &'a str,
    nanos: u64,
}

#[derive(Serialize)]
struct RemarkArg<'a> {
    name: &'a str,
    value: Value,
}

#[derive(Serialize)]
struct Remark<'a> {
    level: &'static str,
    pass: &'a str,
    kind: &'a str,
    name: &'a str,
    span: Span,
    args: Vec<RemarkArg<'a>>,
}

#[derive(Serialize)]
struct Feed<'a> {
    schema_version: u32,
    command: &'a str,
    source: ProductCaptureSource<'a>,
    target: &'a str,
    outcome: Outcome<'a>,
    observed: Observed,
    options: Vec<OptionValue<'a>>,
    pages: Vec<Page<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    unavailable: Vec<Unavailable<'a>>,
    timings: Vec<Timing<'a>>,
    remarks: Vec<Remark<'a>>,
}

/// Serialize one exact product run after its native/legacy selection finished.
/// A non-accepted outcome always strips provisional pages, even if a faulty
/// producer forgot to clear its sidecar.
#[must_use]
pub fn product_capture_value(
    command: &str,
    source: ProductCaptureSource<'_>,
    capture: &StageCapture,
) -> Value {
    let (kind, reason) = match &capture.outcome {
        CaptureOutcome::Pending => ("pending", None),
        CaptureOutcome::Accepted => ("accepted", None),
        CaptureOutcome::Legacy(reason) => ("legacy", Some(reason.as_str())),
        CaptureOutcome::Unavailable(reason) => ("unavailable", Some(reason.as_str())),
        CaptureOutcome::Rejected(reason) => ("rejected", Some(reason.as_str())),
    };
    let accepted = kind == "accepted";
    let feed = Feed {
        schema_version: PRODUCT_STAGE_FEED_VERSION,
        command,
        source,
        target: capture.target.as_str(),
        outcome: Outcome { kind, reason },
        observed: Observed {
            timings: accepted && capture.timings_observed,
            remarks: accepted && capture.remarks_observed,
        },
        options: capture
            .options
            .iter()
            .map(|option| OptionValue {
                name: option.name,
                value: option.value.as_str(),
            })
            .collect(),
        pages: if accepted {
            capture
                .pages
                .iter()
                .map(|page| Page {
                    level: page.level.id(),
                    step: page.step,
                    text: page.text.as_str(),
                })
                .collect()
        } else {
            Vec::new()
        },
        unavailable: if accepted {
            capture
                .unavailable
                .iter()
                .map(|failure| Unavailable {
                    level: failure.level.id(),
                    step: failure.step,
                    reason: failure.reason.as_str(),
                })
                .collect()
        } else {
            Vec::new()
        },
        timings: if accepted && capture.timings_observed {
            capture
                .timings
                .iter()
                .map(|timing| Timing {
                    level: timing.level.id(),
                    step: timing.step,
                    nanos: timing.nanos,
                })
                .collect()
        } else {
            Vec::new()
        },
        remarks: if accepted && capture.remarks_observed {
            capture
                .remarks
                .iter()
                .map(|remark| Remark {
                    level: remark.level.id(),
                    pass: remark.pass.as_str(),
                    kind: remark.kind.as_str(),
                    name: remark.name.as_str(),
                    span: remark.span,
                    args: remark
                        .args
                        .iter()
                        .map(|arg| RemarkArg {
                            name: arg.name.as_str(),
                            value: match &arg.value {
                                CaptureArgValue::Str(value) => Value::String(value.as_str().into()),
                                CaptureArgValue::Int(value) => Value::from(*value),
                                CaptureArgValue::Bool(value) => Value::Bool(*value),
                            },
                        })
                        .collect(),
                })
                .collect()
        } else {
            Vec::new()
        },
    };
    serde_json::to_value(feed).unwrap_or_default()
}

#[cfg(test)]
#[path = "product_capture/fallible_tests.rs"]
mod fallible_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use davinci_test_support::schema;
    use vize_l0::{String, dump::capture::CaptureSink, level::Level};

    fn source() -> ProductCaptureSource<'static> {
        ProductCaptureSource {
            path: Some("src/App.vue"),
            container: "vue-sfc",
            authored_syntax: "vue-template",
            compiled_syntax: "vue-template",
            template_span: Some(Span::new(10, 22)),
        }
    }

    #[test]
    fn accepted_capture_preserves_executed_levels_and_channel_availability() {
        let mut capture = StageCapture::new("dom");
        capture.page(Level::L1, "parse", || String::from("<p>hi</p>"));
        capture.page(Level::L2, "lower", || String::from("op"));
        capture.page(Level::L4, "emit", || String::from("code"));
        capture.finish(|| CaptureOutcome::Accepted);
        let value = product_capture_value("vize-dump", source(), &capture);
        assert_eq!(value["schema_version"], 2);
        assert_eq!(value["target"], "dom");
        assert_eq!(value["outcome"]["kind"], "accepted");
        assert_eq!(value["pages"][2]["level"], "l4");
        assert!(
            value["pages"]
                .as_array()
                .unwrap()
                .iter()
                .all(|page| page["level"] != "l3")
        );
        assert_eq!(value["observed"]["timings"], false);
        assert_eq!(value["observed"]["remarks"], false);
        assert!(value["timings"].as_array().unwrap().is_empty());
        assert!(value["remarks"].as_array().unwrap().is_empty());
    }

    #[test]
    fn fallback_never_serializes_provisional_pages() {
        let mut capture = StageCapture::new("dom");
        capture
            .pages
            .push(vize_l0::dump::capture::StageCapturePage {
                level: Level::L1,
                step: "parse",
                text: String::from("attempted"),
            });
        capture.outcome = CaptureOutcome::Legacy(String::from("unsupported"));
        let value = product_capture_value("compile-sfc", source(), &capture);
        assert_eq!(value["outcome"]["kind"], "legacy");
        assert_eq!(value["outcome"]["reason"], "unsupported");
        assert!(value["pages"].as_array().unwrap().is_empty());
    }

    #[test]
    fn schema_matches_produced_feed() {
        let mut capture = StageCapture::new("dom");
        capture.finish(|| CaptureOutcome::Accepted);
        let value = product_capture_value("vize-dump", source(), &capture);
        let schema_value: Value = serde_json::from_str(include_str!(
            "../../../../docs/davinci/plan/product-stage-feed.schema.json"
        ))
        .unwrap();
        assert_eq!(schema::validate(&schema_value, &value, "$"), Ok(()));
    }
}
