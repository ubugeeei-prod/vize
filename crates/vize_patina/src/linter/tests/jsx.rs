//! Tests for linting JSX/TSX by lowering it to the shared relief template AST.
//!
//! These exercise an element/attribute rule (`vue/a11y-img-alt`) over JSX,
//! confirming AST-driven template rules run unchanged on lowered JSX. Directive
//! structure rules (e.g. `vue/require-v-for-key`) are intentionally out of scope
//! because JSX loops/conditionals lower structurally, not as directives.

use crate::linter::Linter;
use crate::rule::RuleRegistry;
use crate::rules::vue::A11yImgAlt;
use vize_atelier_jsx::JsxLang;

fn img_alt_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(A11yImgAlt));
    Linter::with_registry(registry)
}

#[test]
fn jsx_img_without_alt_is_flagged() {
    let linter = img_alt_linter();
    let result = linter.lint_jsx(
        r#"const A = () => <img src="/x.jpg"/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.warning_count, 1,
        "<img> without alt should be flagged: {:?}",
        result.diagnostics
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "vue/a11y-img-alt"),
        "expected vue/a11y-img-alt diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn jsx_img_with_alt_is_not_flagged() {
    let linter = img_alt_linter();
    let result = linter.lint_jsx(
        r#"const A = () => <img src="/x.jpg" alt=""/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.warning_count, 0,
        "<img> with alt should not be flagged: {:?}",
        result.diagnostics
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/a11y-img-alt"),
        "did not expect vue/a11y-img-alt diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn tsx_img_without_alt_is_flagged() {
    let linter = img_alt_linter();
    let result = linter.lint_jsx(
        r#"const A = () => <img src="/x.jpg"/>;"#,
        "test.tsx",
        JsxLang::Tsx,
    );

    assert_eq!(
        result.warning_count, 1,
        "<img> without alt should be flagged in TSX: {:?}",
        result.diagnostics
    );
}
