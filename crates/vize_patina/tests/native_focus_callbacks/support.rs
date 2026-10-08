use vize_l0::Span;
use vize_patina::{
    HelpLevel, LintDiagnostic, LintResult, Linter, Locale, RuleRegistry, Severity,
    rules::a11y::{NoAccessKey, NoAutofocus},
};

pub const FILE: &str = "/元/test.vue";
pub const AUTO: &str = "a11y/no-autofocus";
pub const KEY: &str = "a11y/no-access-key";
pub const LOCALES: [Locale; 3] = [Locale::En, Locale::Ja, Locale::Zh];

pub fn linter(reverse: bool, locale: Locale, help: HelpLevel) -> Linter {
    let mut registry = RuleRegistry::new();
    if reverse {
        registry.register(Box::new(NoAccessKey));
        registry.register(Box::new(NoAutofocus));
    } else {
        registry.register(Box::new(NoAutofocus));
        registry.register(Box::new(NoAccessKey));
    }
    Linter::with_registry(registry)
        .with_locale(locale)
        .with_help_level(help)
        .with_vue_version(None)
        .with_vapor_mode(None)
}

// Independently authored catalog text, not native translator or decomposition.
pub fn text(rule: &str, locale: Locale) -> (&'static str, &'static str, &'static str) {
    match (rule, locale) {
        (AUTO, Locale::En) => (
            "The autofocus attribute should not be used. It can disrupt navigation for screen reader users",
            "Remove the autofocus attribute. Manage focus programmatically when needed using ref and focus()",
            "Remove the autofocus attribute.",
        ),
        (AUTO, Locale::Ja) => (
            "autofocus属性は使用すべきではありません。スクリーンリーダーユーザーのナビゲーションを妨げる可能性があります",
            "autofocus属性を削除してください。必要な場合はrefとfocus()を使用してプログラムでフォーカスを管理してください",
            "autofocus属性を削除してください。",
        ),
        (AUTO, Locale::Zh) => (
            "不应使用autofocus属性。它可能会干扰屏幕阅读器用户的导航",
            "删除autofocus属性。需要时使用ref和focus()以编程方式管理焦点",
            "删除autofocus属性。",
        ),
        (KEY, Locale::En) => (
            "The accesskey attribute should not be used. Access keys create keyboard shortcut conflicts",
            "Remove the accesskey attribute. Access keys create inconsistent keyboard shortcuts across platforms and conflict with assistive technology shortcuts",
            "Remove the accesskey attribute.",
        ),
        (KEY, Locale::Ja) => (
            "accesskey属性は使用すべきではありません。キーボードショートカットの競合を引き起こします",
            "accesskey属性を削除してください。アクセスキーはプラットフォーム間で不一致なキーボードショートカットを作成し、支援技術のショートカットと競合します",
            "accesskey属性を削除してください。",
        ),
        (KEY, Locale::Zh) => (
            "不应使用accesskey属性。访问键会导致键盘快捷键冲突",
            "删除accesskey属性。访问键在不同平台上创建不一致的键盘快捷键，并与辅助技术快捷键冲突",
            "删除accesskey属性。",
        ),
        _ => panic!("only independently authored focus catalogs"),
    }
}

pub fn span(source: &str, authored: &str) -> Span {
    let start = source.find(authored).unwrap() as u32;
    Span::new(start, start + authored.len() as u32)
}

pub fn finding(
    rule: &'static str,
    locale: Locale,
    help: HelpLevel,
    range: Span,
    severity: Severity,
) -> LintDiagnostic {
    let (message, full, short) = text(rule, locale);
    LintDiagnostic {
        rule_name: rule,
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

pub fn full(result: &LintResult) -> std::string::String {
    format!("{result:#?}")
}

pub fn pair(linter: &Linter, source: &str, expected: LintResult) {
    let expected = full(&expected);
    assert_eq!(full(&linter.lint_template(source, FILE)), expected);
    assert_eq!(
        full(&linter.lint_native_template(source, FILE).unwrap()),
        expected
    );
    assert_eq!(
        full(&linter.lint_native_template(source, FILE).unwrap()),
        expected
    );
    assert_eq!(full(&linter.lint_template(source, FILE)), expected);
}

pub fn original(linter: &Linter, source: &str, expected: LintResult) {
    assert_eq!(full(&linter.lint_template(source, FILE)), full(&expected));
}

pub fn parser(message: &str, range: Span, severity: Severity) -> LintDiagnostic {
    LintDiagnostic {
        rule_name: "parser/template",
        severity,
        message: message.into(),
        start: range.start,
        end: range.end,
        help: None,
        labels: vec![],
        fix: None,
    }
}
