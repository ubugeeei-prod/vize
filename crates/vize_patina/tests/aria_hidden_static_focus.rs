//! Original #7977 inputs and complete diagnostics on existing product paths.

use vize_atelier_jsx::JsxLang;
use vize_l0::Allocator;
use vize_patina::{
    HelpLevel, LintContext, LintDiagnostic, LintResult, Linter, Locale, RuleRegistry,
    ir::TemplateSyntax,
    markup::{L2Template, MarkupContext, MarkupDocument},
    native::template::NativeTemplateLintRefusal,
    rules::a11y::NoAriaHiddenOnFocusable,
};

const RULE: &str = "a11y/no-aria-hidden-on-focusable";
const FILE: &str = "MySelect.vue";
const ORIGINAL: &str = include_str!("fixtures/issue-7977/MySelect.vue.txt");
const CRLF: &str = include_str!("fixtures/issue-7977/MySelectCRLF.vue.txt");
const CONTROLS: &str = include_str!("fixtures/issue-7977/Controls.vue.txt");

fn linter(locale: Locale, help: HelpLevel) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoAriaHiddenOnFocusable));
    Linter::with_registry(registry)
        .with_locale(locale)
        .with_help_level(help)
}

fn expected(source: &str, targets: &[&str], locale: Locale, help: HelpLevel) -> LintResult {
    let (message, action, full) = match locale {
        Locale::En => (
            "aria-hidden=\"true\" must not be used on focusable elements",
            "Remove aria-hidden=\"true\".",
            "Remove aria-hidden=\"true\". tabindex=\"-1\" is still programmatically focusable",
        ),
        Locale::Ja => (
            "フォーカス可能な要素にaria-hidden=\"true\"を使用してはいけません",
            "aria-hidden=\"true\"を削除してください。",
            "aria-hidden=\"true\"を削除してください。tabindex=\"-1\"でもプログラムからはフォーカスできます",
        ),
        Locale::Zh => (
            "不能在可聚焦元素上使用aria-hidden=\"true\"",
            "删除 aria-hidden=\"true\"。",
            "删除 aria-hidden=\"true\"。tabindex=\"-1\" 仍然可以通过程序聚焦",
        ),
    };
    let diagnostics = targets
        .iter()
        .map(|target| {
            let start = source.find(target).unwrap();
            let mut diagnostic =
                LintDiagnostic::error(RULE, message, start as u32, (start + target.len()) as u32);
            if help != HelpLevel::None {
                let text = if help == HelpLevel::Full && target.contains("tabindex=\"-1\"") {
                    full
                } else {
                    action
                };
                diagnostic = diagnostic.with_help(text);
            }
            diagnostic
        })
        .collect();
    LintResult {
        filename: FILE.into(),
        diagnostics,
        error_count: targets.len(),
        warning_count: 0,
    }
}

fn whole(actual: &LintResult, expected: &LintResult) {
    assert_eq!(format!("{actual:#?}"), format!("{expected:#?}"));
}

fn body(source: &str) -> &str {
    source
        .strip_prefix("<template>")
        .unwrap()
        .strip_suffix("</template>\n")
        .or_else(|| {
            source
                .strip_prefix("<template>")
                .unwrap()
                .strip_suffix("</template>\r\n")
        })
        .unwrap()
}

#[test]
fn original_lf_crlf_all_locales_and_help_levels_preserve_the_two_genuine_focusable_findings() {
    for source in [ORIGINAL, CRLF] {
        let negative = "<input class=\"sizer\" readonly tabindex=\"-1\" aria-hidden=\"true\" />";
        let targets = [
            negative,
            "<button type=\"button\" aria-hidden=\"true\">y</button>",
        ];
        for locale in [Locale::En, Locale::Ja, Locale::Zh] {
            for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
                let configured = linter(locale, help);
                whole(
                    &configured.lint_sfc(source, FILE),
                    &expected(source, &targets, locale, help),
                );
                let bare = body(source);
                whole(
                    &configured.lint_template(bare, FILE),
                    &expected(bare, &targets, locale, help),
                );
            }
        }
    }
    assert_eq!(CRLF, ORIGINAL.replace('\n', "\r\n"));
}

#[test]
fn full_boolean_dynamic_focusable_boundary_and_sibling_controls_match_bare_and_sfc_outputs() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/issue-7977/source.json")).unwrap();
    let targets: Vec<_> = manifest["controlTargets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|target| target.as_str().unwrap())
        .collect();
    let configured = linter(Locale::En, HelpLevel::Full);
    whole(
        &configured.lint_sfc(CONTROLS, FILE),
        &expected(CONTROLS, &targets, Locale::En, HelpLevel::Full),
    );
    let bare = body(CONTROLS);
    whole(
        &configured.lint_template(bare, FILE),
        &expected(bare, &targets, Locale::En, HelpLevel::Full),
    );
}

#[test]
fn genuine_relief_and_l1_l2_facade_ancestor_views_publish_identical_whole_findings() {
    let source = body(ORIGINAL);
    let targets = [
        "<input class=\"sizer\" readonly tabindex=\"-1\" aria-hidden=\"true\" />",
        "<button type=\"button\" aria-hidden=\"true\">y</button>",
    ];
    let expected = expected(source, &targets, Locale::En, HelpLevel::Full);
    let allocator = Allocator::default();
    let (root, errors) = vize_armature::Parser::new(&allocator, source).parse();
    assert!(errors.is_empty(), "{errors:?}");
    let lowered = L2Template::lower(&allocator, source);
    assert!(lowered.surface_errors().is_empty());
    let markup = lowered.markup();
    for document in [
        MarkupDocument::new(&root, TemplateSyntax::Vue),
        MarkupDocument::from_l2(&markup, TemplateSyntax::Vue),
    ] {
        let mut lint = LintContext::new(&allocator, source, FILE);
        lint.current_rule = RULE;
        let mut context = MarkupContext::new(&mut lint, &document);
        document.visit_with(&NoAriaHiddenOnFocusable, &mut context);
        whole(
            &LintResult {
                filename: FILE.into(),
                diagnostics: lint.into_diagnostics(),
                error_count: 2,
                warning_count: 0,
            },
            &expected,
        );
    }
}

#[test]
fn jsx_literal_boolean_and_expression_values_preserve_existing_dynamic_conservatism() {
    for (source, target) in [
        (
            "const A = () => <button disabled aria-hidden=\"true\" />;",
            None,
        ),
        (
            "const A = () => <button disabled=\"false\" aria-hidden=\"true\" />;",
            None,
        ),
        (
            "const A = () => <div inert><button aria-hidden=\"true\" /></div>;",
            None,
        ),
        (
            "const A = () => <input type=\"hidden\" aria-hidden=\"true\" />;",
            None,
        ),
        (
            "const A = () => <button disabled={false} aria-hidden=\"true\" />;",
            Some("<button disabled={false} aria-hidden=\"true\" />"),
        ),
        (
            "const A = () => <button inert={false} aria-hidden=\"true\" />;",
            Some("<button inert={false} aria-hidden=\"true\" />"),
        ),
        (
            "const A = () => <div inert={false}><button aria-hidden=\"true\" /></div>;",
            Some("<button aria-hidden=\"true\" />"),
        ),
        (
            "const A = () => <input type={kind} aria-hidden=\"true\" />;",
            Some("<input type={kind} aria-hidden=\"true\" />"),
        ),
    ] {
        let targets = target.into_iter().collect::<Vec<_>>();
        whole(
            &linter(Locale::En, HelpLevel::Full).lint_jsx(source, FILE, JsxLang::Jsx),
            &expected(source, &targets, Locale::En, HelpLevel::Full),
        );
    }
}

#[test]
fn this_unprovided_native_callback_still_refuses_instead_of_claiming_clean_or_handled() {
    for source in [
        body(ORIGINAL),
        body(CONTROLS),
        "<button disabled aria-hidden=\"true\" />",
    ] {
        assert_eq!(
            linter(Locale::En, HelpLevel::Full)
                .lint_native_template(source, FILE)
                .unwrap_err(),
            NativeTemplateLintRefusal::UnprovidedRule { rule: RULE.into() },
        );
    }
}
