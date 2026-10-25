//! Observation support only; original-owner and canonical timings stay separate.
use serde_json::{Value, json};
use vize_carton::i18n::{Locale, translator};
use vize_l0::diag::{PartKind, Stage};
use vize_patina::native::NativeLintFinding;

pub fn body() -> String {
    let mut body = String::from("<main title='作者😀'>");
    for index in 0..64 {
        body.push_str(if index % 4 == 0 {
            "<img src=\"photo\"/>"
        } else {
            "<img alt=''/>"
        });
    }
    body.push_str("</main>");
    body
}

pub fn packet(finding: &NativeLintFinding) -> Value {
    let d = finding.diagnostic();
    assert_eq!(d.stage, Stage::Surface);
    assert_eq!(d.severity(), vize_l0::diag::Severity::Warning);
    assert!(d.witness().is_none());
    assert_eq!(d.parts.len(), 1);
    let help = d.parts.first().unwrap();
    assert_eq!(help.kind, PartKind::Help);
    assert_eq!(help.span, d.span);
    json!({"rule_name":finding.rule_name(),"severity":d.severity().as_str(),
        "message":d.message.as_str(),"start":d.span.start,"end":d.span.end,
        "help":help.message.as_str(),"labels":[],"fix":null})
}

pub fn measure(
    phase: &str,
    representation: &str,
    source: &str,
    mut query: impl FnMut() -> Vec<NativeLintFinding>,
) -> Value {
    let output = query();
    assert_eq!(output.len(), 16);
    let packets: Vec<_> = output.iter().map(packet).collect();
    for _ in 0..100 {
        let output = core::hint::black_box(query());
        assert_eq!(output.len(), 16);
    }
    let profiler = vize_l0::profiler::global_profiler();
    let mut samples = Vec::new();
    for _ in 0..9 {
        profiler.clear();
        profiler.enable();
        for _ in 0..500 {
            let output = core::hint::black_box(query());
            assert_eq!(output.len(), 16);
            // The source phase ended inside query; destruction stays outside it.
            drop(output);
        }
        profiler.disable();
        let metric = profiler.get(phase).unwrap();
        assert_eq!(metric.count, 500);
        assert_eq!(profiler.all().len(), 1);
        assert!(profiler.counter_snapshot().is_empty());
        samples.push(
            json!({"calls":metric.count,"average_ns":metric.average().as_nanos(),
            "histogram_p95_ns":metric.percentile(0.95).as_nanos(),
            "min_ns":metric.min_duration.as_nanos(),"max_ns":metric.max_duration.as_nanos()}),
        );
    }
    json!({"representation":representation,"phase":phase,"source":source,
        "packets":packets,"samples":samples,"scope":"source query only; preparation and result destruction excluded; no whole-product or cross-representation speed ratio"})
}

pub fn messages() -> impl vize_l0::diag::MessageLookup {
    translator().for_locale(Locale::En)
}
