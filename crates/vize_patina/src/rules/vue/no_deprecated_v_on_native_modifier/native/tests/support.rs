use crate::rules::vue::{NoDeprecatedVOnNativeModifier, NoDeprecatedVOnNumberModifiers};
use crate::{HelpLevel, LintResult, Linter, Locale, RuleRegistry, Severity};
use vize_l0::String;

pub const RULE: &str = "vue/no-deprecated-v-on-native-modifier";
pub const NUMERIC: &str = "vue/no-deprecated-v-on-number-modifiers";
pub const SOURCE: &str = "588289eae50d3724e61246d5ce44c0a7044ed807";
pub const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/native-event-modifier-6848/Original.html.txt"
));

#[derive(Clone, Copy, Debug)]
pub enum Order {
    NativeOnly,
    NativeNumeric,
    NumericNative,
}

pub fn configured(
    order: Order,
    locale: Locale,
    help: HelpLevel,
    severity: Option<Severity>,
) -> Linter {
    let mut registry = RuleRegistry::new();
    match order {
        Order::NativeOnly => registry.register(Box::new(NoDeprecatedVOnNativeModifier)),
        Order::NativeNumeric => {
            registry.register(Box::new(NoDeprecatedVOnNativeModifier));
            registry.register(Box::new(NoDeprecatedVOnNumberModifiers));
        }
        Order::NumericNative => {
            registry.register(Box::new(NoDeprecatedVOnNumberModifiers));
            registry.register(Box::new(NoDeprecatedVOnNativeModifier));
        }
    }
    let linter = Linter::with_registry(registry)
        .with_locale(locale)
        .with_help_level(help);
    match severity {
        Some(value) => {
            linter.with_rule_severity_overrides(vec![(RULE.into(), value), (NUMERIC.into(), value)])
        }
        None => linter,
    }
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "complete Case fields remain in emitted Debug custody"
)]
pub struct Case<'a> {
    pub id: &'a str,
    pub authority: &'static str,
    pub history: &'static str,
    pub source: &'a str,
    pub filename: &'static str,
    pub entry: &'static str,
    pub rule: &'static str,
    pub registration: Order,
    pub locale: Locale,
    pub help: HelpLevel,
    pub severity: Option<Severity>,
    pub diagnostics: Option<usize>,
    pub fixes: Option<usize>,
}

pub fn case<'a>(
    id: &'a str,
    source: &'a str,
    order: Order,
    diagnostics: usize,
    fixes: usize,
) -> Case<'a> {
    Case {
        id,
        authority: "whole cross-rule original or new focused control; not an old native bare golden",
        history: SOURCE,
        source,
        filename: "App.vue",
        entry: "template",
        rule: RULE,
        registration: order,
        locale: Locale::En,
        help: HelpLevel::Full,
        severity: None,
        diagnostics: Some(diagnostics),
        fixes: Some(fixes),
    }
}

#[derive(Debug)]
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

fn query(linter: &Linter, case: &Case<'_>, source: &str, native: bool) -> LintResult {
    if native {
        linter
            .lint_native_template(source, case.filename)
            .unwrap_or_else(|error| panic!("{case:#?}\nsource:{source:?}\nnative:{error:#?}"))
    } else {
        linter.lint_template(source, case.filename)
    }
}

pub fn observe(linter: &Linter, case: &Case<'_>, native: bool) -> Observation {
    let initial = query(linter, case, case.source, native);
    let applications: Vec<_> = initial
        .diagnostics
        .iter()
        .enumerate()
        .filter_map(|(diagnostic_index, diagnostic)| {
            diagnostic.fix.as_ref().map(|fix| {
                let source = fix.apply(case.source);
                let result = query(linter, case, &source, native);
                Application {
                    diagnostic_index,
                    source,
                    result,
                }
            })
        })
        .collect();
    let unchanged_requery = applications
        .is_empty()
        .then(|| query(linter, case, case.source, native));
    Observation {
        initial,
        applications,
        unchanged_requery,
    }
}

pub fn pair(case: &Case<'_>) -> Observation {
    let linter = configured(case.registration, case.locale, case.help, case.severity);
    let legacy = observe(&linter, case, false);
    println!("completed legacy before native attempt:\n{case:#?}\n{legacy:#?}");
    let native = observe(&linter, case, true);
    let legacy_bytes = format!("{case:#?}\n{legacy:#?}\n");
    let native_bytes = format!("{case:#?}\n{native:#?}\n");
    println!("legacy:\n{legacy_bytes}native:\n{native_bytes}");
    assert_eq!(
        legacy.initial.diagnostics.len(),
        case.diagnostics.unwrap(),
        "{legacy_bytes}"
    );
    assert_eq!(
        legacy.applications.len(),
        case.fixes.unwrap(),
        "{legacy_bytes}"
    );
    assert_eq!(native_bytes, legacy_bytes);
    assert_eq!(
        format!("{case:#?}\n{:#?}\n", observe(&linter, case, true)),
        legacy_bytes
    );
    legacy
}

pub fn span(source: &str, needle: &str) -> vize_l0::Span {
    let start = source.find(needle).unwrap() as u32;
    vize_l0::Span::new(start, start + needle.len() as u32)
}
