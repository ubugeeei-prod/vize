use crate::rules::vue::{NoDeprecatedVOnNativeModifier, NoDeprecatedVOnNumberModifiers, ValidVOn};
use crate::{HelpLevel, LintResult, Linter, Locale, RuleRegistry, Severity};
use vize_l0::String;

pub const RULE: &str = "vue/valid-v-on";
pub const NATIVE: &str = "vue/no-deprecated-v-on-native-modifier";
pub const NUMERIC: &str = "vue/no-deprecated-v-on-number-modifiers";
pub const SOURCE: &str = "476d72209d7dac6bcc4a2ad3a18431ef561b7052";
pub const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/native-event-modifier-6848/Original.html.txt"
));
pub const UNSUPPORTED_MODIFIER_ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/native-valid-v-on-6848/Original.html.txt"
));

#[derive(Clone, Copy, Debug)]
pub enum Order {
    ValidOnly,
    NativeOnly,
    ValidNativeNumeric,
    ValidNumericNative,
    NativeValidNumeric,
    NativeNumericValid,
    NumericValidNative,
    NumericNativeValid,
}
pub const ORDERS: [Order; 6] = [
    Order::ValidNativeNumeric,
    Order::ValidNumericNative,
    Order::NativeValidNumeric,
    Order::NativeNumericValid,
    Order::NumericValidNative,
    Order::NumericNativeValid,
];
impl Order {
    pub fn names(self) -> &'static [&'static str] {
        match self {
            Self::ValidOnly => &[RULE],
            Self::NativeOnly => &[NATIVE],
            Self::ValidNativeNumeric => &[RULE, NATIVE, NUMERIC],
            Self::ValidNumericNative => &[RULE, NUMERIC, NATIVE],
            Self::NativeValidNumeric => &[NATIVE, RULE, NUMERIC],
            Self::NativeNumericValid => &[NATIVE, NUMERIC, RULE],
            Self::NumericValidNative => &[NUMERIC, RULE, NATIVE],
            Self::NumericNativeValid => &[NUMERIC, NATIVE, RULE],
        }
    }
}

pub fn configured(input: &Case<'_>) -> Linter {
    let mut registry = RuleRegistry::new();
    for name in input.registration.names() {
        match *name {
            RULE => registry.register(Box::new(ValidVOn)),
            NATIVE => registry.register(Box::new(NoDeprecatedVOnNativeModifier)),
            NUMERIC => registry.register(Box::new(NoDeprecatedVOnNumberModifiers)),
            _ => unreachable!(),
        }
    }
    let mut linter = Linter::with_registry(registry)
        .with_locale(input.locale)
        .with_help_level(input.help)
        .with_disabled_rules(input.disabled_rules.clone());
    if let Some(custom) = &input.custom_modifiers {
        linter = linter.with_valid_v_on_modifiers(custom.clone());
    }
    if let Some(value) = input.severity {
        linter = linter.with_rule_severity_overrides(vec![
            (RULE.into(), value),
            (NATIVE.into(), value),
            (NUMERIC.into(), value),
        ]);
    }
    linter
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "the complete Case remains in emitted Debug custody"
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
    pub custom_modifiers: Option<Vec<String>>,
    pub disabled_rules: Vec<String>,
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
        authority: "complete old valid-v-on unit input or explicitly new control; no historical full golden",
        history: SOURCE,
        source,
        filename: "Original.vue",
        entry: "template",
        rule: RULE,
        registration: order,
        custom_modifiers: None,
        disabled_rules: Vec::new(),
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
fn query(linter: &Linter, input: &Case<'_>, source: &str, native: bool) -> LintResult {
    if native {
        let result = linter.lint_native_template(source, input.filename);
        println!("query-native-before-verdict:{input:#?}\nsource:{source:?}\nresult:{result:#?}");
        result.unwrap_or_else(|error| panic!("{input:#?}\nsource:{source:?}\n{error:#?}"))
    } else {
        let result = linter.lint_template(source, input.filename);
        println!("query-legacy-before-verdict:{input:#?}\nsource:{source:?}\nresult:{result:#?}");
        result
    }
}
pub fn observe(linter: &Linter, input: &Case<'_>, native: bool) -> Observation {
    let initial = query(linter, input, input.source, native);
    let applications = initial
        .diagnostics
        .iter()
        .enumerate()
        .filter_map(|(diagnostic_index, diagnostic)| {
            diagnostic.fix.as_ref().map(|fix| {
                let source = fix.apply(input.source);
                let result = query(linter, input, &source, native);
                Application {
                    diagnostic_index,
                    source,
                    result,
                }
            })
        })
        .collect::<Vec<_>>();
    let unchanged_requery = applications
        .is_empty()
        .then(|| query(linter, input, input.source, native));
    Observation {
        initial,
        applications,
        unchanged_requery,
    }
}
pub fn pair_with(linter: &Linter, input: &Case<'_>) -> Observation {
    let legacy = observe(linter, input, false);
    println!("completed legacy before native attempt:\n{input:#?}\n{legacy:#?}");
    let native = observe(linter, input, true);
    let legacy_bytes = format!("{input:#?}\n{legacy:#?}\n");
    let native_bytes = format!("{input:#?}\n{native:#?}\n");
    println!("legacy:\n{legacy_bytes}native:\n{native_bytes}");
    assert_eq!(
        legacy.initial.diagnostics.len(),
        input.diagnostics.unwrap(),
        "{legacy_bytes}"
    );
    assert_eq!(
        legacy.applications.len(),
        input.fixes.unwrap(),
        "{legacy_bytes}"
    );
    assert_eq!(native_bytes, legacy_bytes);
    assert_eq!(
        format!("{input:#?}\n{:#?}\n", observe(linter, input, true)),
        legacy_bytes
    );
    legacy
}
pub fn pair(input: &Case<'_>) -> Observation {
    pair_with(&configured(input), input)
}
pub fn span(source: &str, needle: &str) -> vize_l0::Span {
    let start = source.find(needle).unwrap() as u32;
    vize_l0::Span::new(start, start + needle.len() as u32)
}
