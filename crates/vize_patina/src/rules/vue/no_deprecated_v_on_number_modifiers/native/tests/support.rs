use crate::{HelpLevel, LintResult, Linter, Locale, RuleRegistry, Severity};
use vize_l0::String;

pub const RULE: &str = "vue/no-deprecated-v-on-number-modifiers";
pub const SOURCE: &str = "bc4d517ca5d835ddeba49dc9cc2f4c9699dd0cab";
pub const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/native-numeric-key-modifiers-6848/Original.html.txt"
));

pub fn configured(locale: Locale, help: HelpLevel, severity: Option<Severity>) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(
        super::super::super::NoDeprecatedVOnNumberModifiers,
    ));
    let linter = Linter::with_registry(registry)
        .with_locale(locale)
        .with_help_level(help);
    match severity {
        Some(severity) => linter.with_rule_severity_overrides(vec![(RULE.into(), severity)]),
        None => linter,
    }
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "all original-style Case fields are retained in complete Debug custody"
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
    reason = "every independent fix application is retained in complete Debug custody"
)]
pub struct Application {
    pub diagnostic_index: usize,
    pub source: String,
    pub result: LintResult,
}

#[derive(Debug)]
pub struct Observation {
    pub initial: LintResult,
    pub applications: Vec<Application>,
    pub unchanged_requery: Option<LintResult>,
}

pub fn query(linter: &Linter, source: &str, filename: &str, native: bool) -> LintResult {
    if native {
        linter.lint_native_template(source, filename).unwrap()
    } else {
        linter.lint_template(source, filename)
    }
}

pub fn observe(linter: &Linter, case: &Case<'_>, native: bool) -> Observation {
    let initial = query(linter, case.source, case.filename, native);
    assert_eq!(
        initial.diagnostics.len(),
        case.diagnostics,
        "{case:#?}\n{initial:#?}"
    );
    let applications: Vec<_> = initial
        .diagnostics
        .iter()
        .enumerate()
        .filter_map(|(diagnostic_index, diagnostic)| {
            diagnostic.fix.as_ref().map(|fix| {
                let source = fix.apply(case.source);
                let result = query(linter, &source, case.filename, native);
                Application {
                    diagnostic_index,
                    source,
                    result,
                }
            })
        })
        .collect();
    println!(
        "completed prefix before fix-count assertion:\n{case:#?}\ninitial:{initial:#?}\napplications:{applications:#?}\nunchanged_requery:UNREACHED"
    );
    assert_eq!(
        applications.len(),
        case.fixes,
        "{case:#?}\n{initial:#?}\n{applications:#?}"
    );
    let unchanged_requery = applications
        .is_empty()
        .then(|| query(linter, case.source, case.filename, native));
    Observation {
        initial,
        applications,
        unchanged_requery,
    }
}

pub fn pair(
    id: &str,
    source: &str,
    diagnostics: usize,
    fixes: usize,
    locale: Locale,
    help: HelpLevel,
    severity: Option<Severity>,
) -> Observation {
    let case = Case {
        id,
        history: SOURCE,
        source,
        filename: "App.vue",
        entry: "template",
        rule: RULE,
        vue_version: None,
        vapor: None,
        diagnostics,
        fixes,
    };
    let linter = configured(locale, help, severity);
    let legacy = observe(&linter, &case, false);
    let native = observe(&linter, &case, true);
    let legacy_bytes = format!("{case:#?}\n{legacy:#?}\n");
    let native_bytes = format!("{case:#?}\n{native:#?}\n");
    println!(
        "configuration:{locale:?}/{help:?}/{severity:?}\nlegacy:\n{legacy_bytes}native:\n{native_bytes}"
    );
    assert_eq!(native_bytes, legacy_bytes);
    assert_eq!(
        format!("{case:#?}\n{:#?}\n", observe(&linter, &case, true)),
        legacy_bytes
    );
    legacy
}
