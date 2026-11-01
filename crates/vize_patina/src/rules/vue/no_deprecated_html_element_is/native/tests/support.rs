use crate::{HelpLevel, LintResult, Linter, Locale, RuleRegistry, Severity};
use serde::Deserialize;
use serde_json::{Value, json};
use vize_l0::String;

pub const RULE: &str = "vue/no-deprecated-html-element-is";
pub const PRODUCER: &str = "a0617450b2c233be169ab586effe88a4b2cf5264";
pub const ORIGINAL: &str = "<div is=\"MyComponent\" />";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub id: String,
    pub source: String,
    pub spans: Vec<[u32; 2]>,
}

pub fn fixtures() -> Vec<Fixture> {
    let value: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/linter/native-html-element-is-6848/cases.json"
    )))
    .unwrap();
    assert_eq!(value["producer"], PRODUCER);
    assert_eq!(value["entry"], "template");
    let cases: Vec<Fixture> = serde_json::from_value(value["cases"].clone()).unwrap();
    assert_eq!(cases.len(), 12);
    cases
}

pub fn configured(locale: Locale, help: HelpLevel, severity: Option<Severity>) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(super::super::super::NoDeprecatedHtmlElementIs));
    let linter = Linter::with_registry(registry)
        .with_locale(locale)
        .with_help_level(help);
    if let Some(severity) = severity {
        linter.with_rule_severity_overrides(vec![(RULE.into(), severity)])
    } else {
        linter
    }
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "every original-style Case field is retained in complete Debug bytes"
)]
pub struct Case<'a> {
    pub id: &'a str,
    pub history: &'static str,
    pub source: &'a str,
    pub filename: &'a str,
    pub entry: &'static str,
    pub rule: &'static str,
    pub vue_version: Option<&'static str>,
    pub vapor: Option<bool>,
    pub diagnostics: usize,
    pub fixes: usize,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "the complete no-fix Application shape stays in the observation contract"
)]
pub struct Application {
    diagnostic_index: usize,
    source: String,
    result: LintResult,
}

#[derive(Debug)]
pub struct Observation {
    pub initial: LintResult,
    pub applications: Vec<Application>,
    pub unchanged_requery: Option<LintResult>,
}

pub fn observe(linter: &Linter, case: &Case<'_>, native: bool) -> Observation {
    let query = || {
        if native {
            assert_eq!(case.entry, "template");
            linter
                .lint_native_template(case.source, case.filename)
                .unwrap()
        } else if case.entry == "sfc" {
            linter.lint_sfc(case.source, case.filename)
        } else {
            linter.lint_template(case.source, case.filename)
        }
    };
    Observation {
        initial: query(),
        applications: Vec::new(),
        unchanged_requery: Some(query()),
    }
}

pub fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result.diagnostics.iter().map(|d| {
        let labels: Vec<_> = d.labels.iter().map(|l| json!({"message":l.message.as_str(),"start":l.start,"end":l.end})).collect();
        json!({"rule_name":d.rule_name,"severity":d.severity,"message":d.message.as_str(),
            "start":d.start,"end":d.end,"help":d.help.as_ref().map(|h|h.as_str()),"labels":labels,"fix":d.fix})
    }).collect();
    json!({"filename":result.filename.as_str(),"diagnostics":diagnostics,
        "error_count":result.error_count,"warning_count":result.warning_count})
}

pub fn expected(
    filename: &str,
    spans: &[[u32; 2]],
    severity: Severity,
    locale: Locale,
    help: HelpLevel,
) -> Value {
    let (message, full_help) = match locale {
        Locale::En => (
            "the `is` attribute on native HTML elements (component substitution) was removed in Vue 3",
            "Use `<component :is=\"...\">` for dynamic components, or prefix the value with `vue:` (e.g. `is=\"vue:MyComponent\"`) for a customized built-in element.",
        ),
        Locale::Ja => (
            "ネイティブHTML要素の`is`属性（コンポーネント置換）はVue 3で削除されました",
            "動的コンポーネントには`<component :is=\"...\">`を使うか、カスタマイズされた組み込み要素には値に`vue:`を付けてください（例: `is=\"vue:MyComponent\"`）。",
        ),
        Locale::Zh => (
            "原生HTML元素上的`is`属性（组件替换）已在Vue 3中移除",
            "对于动态组件请使用`<component :is=\"...\">`，对于定制内置元素请在值前加`vue:`前缀（例如 `is=\"vue:MyComponent\"`）。",
        ),
        _ => panic!("unprovided test locale"),
    };
    let diagnostics: Vec<_> = spans
        .iter()
        .map(|[start, end]| {
            json!({
                "rule_name":RULE,"severity":severity,"message":message,"start":start,"end":end,
                "help":help.process(full_help).map(|h|h.to_string()),"labels":[],"fix":null,
            })
        })
        .collect();
    json!({"filename":filename,"diagnostics":diagnostics,
        "error_count":if severity==Severity::Error {spans.len()} else {0},
        "warning_count":if severity==Severity::Warning {spans.len()} else {0}})
}

pub fn assert_pair(
    fixture: &Fixture,
    locale: Locale,
    help: HelpLevel,
    severity: Option<Severity>,
) -> Observation {
    let case = Case {
        id: &fixture.id,
        history: PRODUCER,
        source: &fixture.source,
        filename: "App.vue",
        entry: "template",
        rule: RULE,
        vue_version: None,
        vapor: None,
        diagnostics: fixture.spans.len(),
        fixes: 0,
    };
    let linter = configured(locale, help, severity);
    let legacy = observe(&linter, &case, false);
    let native = observe(&linter, &case, true);
    let expected = expected(
        case.filename,
        &fixture.spans,
        severity.unwrap_or(Severity::Error),
        locale,
        help,
    );
    for observed in [&legacy, &native] {
        assert_eq!(complete(&observed.initial), expected);
        assert_eq!(observed.applications.len(), 0);
        assert_eq!(
            complete(observed.unchanged_requery.as_ref().unwrap()),
            expected
        );
    }
    let legacy_bytes = format!("{case:#?}\n{legacy:#?}\n");
    let native_bytes = format!("{case:#?}\n{native:#?}\n");
    println!(
        "configuration: {locale:?}/{help:?}/{severity:?}\nlegacy:\n{legacy_bytes}native:\n{native_bytes}"
    );
    assert_eq!(native_bytes, legacy_bytes);
    assert_eq!(
        format!("{case:#?}\n{:#?}\n", observe(&linter, &case, true)),
        legacy_bytes
    );
    legacy
}

pub fn assert_refused(
    source: &str,
    filename: &str,
    spans: &[[u32; 2]],
    refusal: crate::native::template::NativeTemplateLintRefusal,
) {
    let case = Case {
        id: "new-explicit-refusal-control",
        history: PRODUCER,
        source,
        filename,
        entry: "template",
        rule: RULE,
        vue_version: None,
        vapor: None,
        diagnostics: spans.len(),
        fixes: 0,
    };
    let linter = configured(Locale::En, HelpLevel::Full, None);
    let legacy = observe(&linter, &case, false);
    let expected = expected(
        filename,
        spans,
        Severity::Error,
        Locale::En,
        HelpLevel::Full,
    );
    assert_eq!(complete(&legacy.initial), expected);
    assert_eq!(
        complete(legacy.unchanged_requery.as_ref().unwrap()),
        expected
    );
    assert_eq!(legacy.applications.len(), 0);
    for _ in 0..2 {
        assert_eq!(
            linter.lint_native_template(source, filename).unwrap_err(),
            refusal
        );
    }
    println!("{case:#?}\n{legacy:#?}\nnative:{refusal:?}");
}
