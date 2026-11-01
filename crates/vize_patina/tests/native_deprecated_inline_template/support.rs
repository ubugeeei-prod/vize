use vize_l0::Span;
use vize_patina::{
    HelpLevel, LintDiagnostic, LintResult, Linter, Locale, RuleRegistry, Severity,
    native::template::NativeTemplateLintRefusal, rules::vue::NoDeprecatedInlineTemplate,
};

pub const RULE: &str = "vue/no-deprecated-inline-template";
pub const FILE: &str = "App.vue";
pub const ORIGINAL: &str = include_str!(
    "../../../../tests/_fixtures/differential/linter/native-inline-template-6848/Original.html.txt"
);
pub const LOCALES: [Locale; 3] = [Locale::En, Locale::Ja, Locale::Zh];

pub fn configured(locale: Locale, help: HelpLevel, severity: Severity) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoDeprecatedInlineTemplate));
    Linter::with_registry(registry)
        .with_locale(locale)
        .with_help_level(help)
        .with_rule_severity_overrides(vec![(RULE.into(), severity)])
}

// Independent complete catalog strings, not a call through the tested reporter.
pub fn finding(locale: Locale, help: HelpLevel, severity: Severity, range: Span) -> LintDiagnostic {
    let (message, full, short) = match locale {
        Locale::En => (
            "the `inline-template` attribute was removed in Vue 3",
            "Move the markup into the child component or pass it through slots instead of `inline-template`.",
            "Move the markup into the child component or pass it through slots instead of inline-template.",
        ),
        Locale::Ja => (
            "`inline-template`属性はVue 3で削除されました",
            "マークアップを子コンポーネントへ移すか、`inline-template`の代わりにスロットで渡してください。",
            "マークアップを子コンポーネントへ移すか、inline-templateの代わりにスロットで渡してください。",
        ),
        Locale::Zh => (
            "`inline-template`属性已在Vue 3中移除",
            "请将标记移入子组件，或通过插槽传递，而不是使用`inline-template`。",
            "请将标记移入子组件，或通过插槽传递，而不是使用inline-template。",
        ),
    };
    LintDiagnostic {
        rule_name: RULE,
        severity,
        message: message.into(),
        start: range.start,
        end: range.end,
        help: match help {
            HelpLevel::None => None,
            HelpLevel::Short => Some(short.into()),
            HelpLevel::Full => Some(full.into()),
        },
        labels: vec![],
        fix: None,
    }
}

pub fn expected(diagnostics: Vec<LintDiagnostic>) -> LintResult {
    LintResult {
        filename: FILE.into(),
        error_count: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count(),
        warning_count: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count(),
        diagnostics,
    }
}

pub fn span(source: &str, authored: &str) -> Span {
    let start = u32::try_from(source.find(authored).unwrap()).unwrap();
    Span::new(start, start + u32::try_from(authored.len()).unwrap())
}

pub fn full(result: &LintResult) -> std::string::String {
    format!("{result:#?}")
}

#[derive(Debug)]
pub struct Observation {
    source: std::string::String,
    expected: LintResult,
    legacy: LintResult,
    native: Result<LintResult, NativeTemplateLintRefusal>,
    native_requery: Result<LintResult, NativeTemplateLintRefusal>,
    legacy_requery: LintResult,
}

pub fn observe(linter: &Linter, source: &str, expected: LintResult) -> Observation {
    Observation {
        source: source.into(),
        expected,
        legacy: linter.lint_template(source, FILE),
        native: linter.lint_native_template(source, FILE),
        native_requery: linter.lint_native_template(source, FILE),
        legacy_requery: linter.lint_template(source, FILE),
    }
}

impl Observation {
    pub fn assert_pair(&self) {
        let expected = full(&self.expected);
        assert_eq!(full(&self.legacy), expected, "{}", self.source);
        assert_eq!(
            full(self.native.as_ref().unwrap()),
            expected,
            "{}",
            self.source
        );
        assert_eq!(
            full(self.native_requery.as_ref().unwrap()),
            expected,
            "{}",
            self.source
        );
        assert_eq!(full(&self.legacy_requery), expected, "{}", self.source);
    }

    pub fn assert_refusal(&self, refusal: NativeTemplateLintRefusal) {
        assert_eq!(
            self.native.as_ref().unwrap_err(),
            &refusal,
            "{}",
            self.source
        );
        assert_eq!(
            self.native_requery.as_ref().unwrap_err(),
            &refusal,
            "{}",
            self.source
        );
        assert_eq!(full(&self.legacy), full(&self.expected), "{}", self.source);
        assert_eq!(
            full(&self.legacy_requery),
            full(&self.expected),
            "{}",
            self.source
        );
    }
}
