//! Observation support only; original-owner and canonical timings stay separate.
use serde_json::{Value, json};
use vize_carton::i18n::{Locale, translator};
use vize_l0::String;
use vize_l0::diag::{PartKind, Stage};
use vize_patina::native::NativeLintFinding;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn failure(message: &str) -> Box<dyn std::error::Error> {
    message.into()
}

pub fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(failure(message))
    }
}

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

pub fn packet(finding: &NativeLintFinding) -> Result<Value> {
    let d = finding.diagnostic();
    ensure(d.stage == Stage::Surface, "surface diagnostic stage")?;
    ensure(
        d.severity() == vize_l0::diag::Severity::Warning,
        "warning severity",
    )?;
    ensure(d.witness().is_none(), "no diagnostic witness")?;
    ensure(d.parts.len() == 1, "one complete help part")?;
    let help = d
        .parts
        .first()
        .ok_or_else(|| failure("missing complete help part"))?;
    ensure(help.kind == PartKind::Help, "help part kind")?;
    ensure(
        help.span == d.span,
        "help part matches authored primary range",
    )?;
    Ok(
        json!({"rule_name":finding.rule_name(),"severity":d.severity().as_str(),
        "message":d.message.as_str(),"start":d.span.start,"end":d.span.end,
        "help":help.message.as_str(),"labels":[],"fix":null}),
    )
}

pub fn measure(
    phase: &str,
    representation: &str,
    source: &str,
    mut query: impl FnMut() -> Result<Vec<NativeLintFinding>>,
) -> Result<Value> {
    let output = query()?;
    ensure(output.len() == 16, "complete initial finding count")?;
    let packets = output.iter().map(packet).collect::<Result<Vec<_>>>()?;
    for _ in 0..100 {
        let output = core::hint::black_box(query()?);
        ensure(output.len() == 16, "complete warmup finding count")?;
    }
    let profiler = vize_l0::profiler::global_profiler();
    let mut samples = Vec::new();
    for _ in 0..9 {
        profiler.clear();
        profiler.enable();
        for _ in 0..500 {
            let output = core::hint::black_box(query()?);
            ensure(output.len() == 16, "complete measured finding count")?;
            // The source phase ended inside query; destruction stays outside it.
            drop(output);
        }
        profiler.disable();
        let metric = profiler
            .get(phase)
            .ok_or_else(|| failure("missing exact measured phase"))?;
        ensure(metric.count == 500, "all 500 phase calls retained")?;
        ensure(profiler.all().len() == 1, "only the selected source phase")?;
        ensure(
            profiler.counter_snapshot().is_empty(),
            "no counter truncation",
        )?;
        samples.push(
            json!({"calls":metric.count,"average_ns":metric.average().as_nanos(),
            "histogram_p95_ns":metric.percentile(0.95).as_nanos(),
            "min_ns":metric.min_duration.as_nanos(),"max_ns":metric.max_duration.as_nanos()}),
        );
    }
    Ok(
        json!({"representation":representation,"phase":phase,"source":source,
        "packets":packets,"samples":samples,"scope":"source query only; preparation and result destruction excluded; no whole-product or cross-representation speed ratio"}),
    )
}

pub fn messages() -> impl vize_l0::diag::MessageLookup {
    translator().for_locale(Locale::En)
}
