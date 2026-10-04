use serde_json::{Value, json};
use vize_carton::i18n::translator;
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Locale, Severity};

pub const RULE: &str = "vue/component-definition-name-casing";
pub const SOURCE: &str = "<div>Content</div>";
pub const LOCALES: [Locale; 3] = [Locale::En, Locale::Ja, Locale::Zh];

pub fn configured(locale: Locale, help: HelpLevel) -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_locale(locale)
        .with_help_level(help)
        .with_vue_version(None)
        .with_vapor_mode(None)
}

pub fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| {
                    json!({
                        "message": label.message.as_str(), "start": label.start, "end": label.end,
                    })
                })
                .collect();
            json!({
                "rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
                "message": diagnostic.message.as_str(), "start": diagnostic.start,
                "end": diagnostic.end, "help": diagnostic.help.as_ref().map(|help| help.as_str()),
                "labels": labels, "fix": diagnostic.fix,
            })
        })
        .collect();
    json!({
        "filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics,
    })
}

pub fn warning(locale: Locale, help: HelpLevel, stem: &str, severity: Severity) -> Value {
    json!({
        "rule_name": RULE, "severity": severity,
        "message": translator().format(locale, "vue/component-definition-name-casing.message", &[("name", stem)]).as_str(),
        "start": 0, "end": 0,
        "help": help.process(translator().get(locale, "vue/component-definition-name-casing.help").as_ref()).map(|help| help.to_string()),
        "labels": [], "fix": null,
    })
}

pub fn expected(filename: &str, diagnostics: Vec<Value>) -> Value {
    let errors = diagnostics
        .iter()
        .filter(|d| d["severity"] == "error")
        .count();
    json!({
        "filename": filename, "error_count": errors,
        "warning_count": diagnostics.len() - errors, "diagnostics": diagnostics,
    })
}

pub fn assert_pair(linter: &Linter, source: &str, filename: &str, expected: Value) {
    let original = linter.lint_template(source, filename);
    let native = linter.lint_native_template(source, filename).unwrap();
    assert_eq!(
        complete(&original),
        expected,
        "original {source} / {filename}"
    );
    assert_eq!(complete(&native), expected, "native {source} / {filename}");
    assert_eq!(format!("{native:#?}"), format!("{original:#?}"));
    let repeat = linter.lint_native_template(source, filename).unwrap();
    assert_eq!(format!("{native:#?}"), format!("{repeat:#?}"));
}

pub fn parser(message: &str, start: u32, end: u32) -> Value {
    json!({
        "rule_name": "parser/template", "severity": "error", "message": message,
        "start": start, "end": end, "help": null, "labels": [], "fix": null,
    })
}
