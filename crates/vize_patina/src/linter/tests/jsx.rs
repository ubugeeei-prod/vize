//! End-to-end tests for linting `.jsx`/`.tsx` through [`Linter::lint_jsx`].
//!
//! [`Linter::lint_jsx`] runs two layers:
//!
//! 1. the **zero-cost markup-IR pass** — rules with a
//!    [`MarkupRule`](crate::markup::MarkupRule) projection run straight over the
//!    OXC AST via [`MarkupDocument::from_jsx`](crate::markup::MarkupDocument::from_jsx),
//!    with no synthetic template AST; and
//! 2. a **lowering fallback** — rules without a markup entry point run over the
//!    relief AST produced by `vize_atelier_jsx::lower_source`.
//!
//! The first group of tests targets the IR pass (the `#1499` common path); the
//! `fallback_*` tests keep an unmigrated rule (`vue/a11y-img-alt`, which only has
//! a legacy `Rule` impl) firing over the lowering path; and `no_jsx_equivalent_*`
//! documents that a directive with no JSX analogue is silently skipped.

use crate::linter::Linter;
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::{
    ImgAlt, NoAccessKey, NoAutofocus, NoDistractingElements, TabindexNoPositive,
};
use crate::rules::html::DeprecatedElement;
use crate::rules::vue::{A11yImgAlt, NoVHtml, RequireVForKey};
use vize_atelier_jsx::JsxLang;

fn linter_with(rule: Box<dyn Rule>) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry)
}

// ===========================================================================
// Zero-cost IR pass: migrated rules fire on JSX/TSX with no template AST.
// ===========================================================================

#[test]
fn ir_img_alt_jsx_without_alt_is_flagged() {
    // `a11y/img-alt` (the markup-migrated rule) runs over the OXC AST directly.
    let linter = linter_with(Box::new(ImgAlt));
    let result = linter.lint_jsx(
        r#"const A = () => <img src="/x.jpg"/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.warning_count, 1,
        "<img> without alt must flag through the IR pass: {:?}",
        result.diagnostics
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "a11y/img-alt"),
        "expected a11y/img-alt diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn ir_img_alt_jsx_with_static_alt_is_clean() {
    let linter = linter_with(Box::new(ImgAlt));
    let result = linter.lint_jsx(
        r#"const A = () => <img src="/x.jpg" alt="hi"/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.warning_count, 0,
        "static alt must be clean: {:?}",
        result.diagnostics
    );
}

#[test]
fn ir_img_alt_jsx_with_dynamic_alt_is_clean() {
    let linter = linter_with(Box::new(ImgAlt));
    let result = linter.lint_jsx(
        r#"const A = () => <img src={photo} alt={caption}/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.warning_count, 0,
        "dynamic alt={{…}} must be clean: {:?}",
        result.diagnostics
    );
}

#[test]
fn ir_img_alt_tsx_is_flagged() {
    let linter = linter_with(Box::new(ImgAlt));
    let result = linter.lint_jsx(
        "const A = (p: Props): JSX.Element => <img src=\"/x.jpg\"/>;",
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        result.warning_count, 1,
        "TSX <img> without alt must flag: {:?}",
        result.diagnostics
    );
}

#[test]
fn ir_diagnostic_range_points_at_jsx_source() {
    // The reported range must address the original JSX bytes, so editor fixes
    // land on the written `<img>` and not some reconstructed offset.
    let source = r#"const A = () => <div><img src="/x.jpg"/></div>;"#;
    let linter = linter_with(Box::new(ImgAlt));
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(result.diagnostics.len(), 1);
    let diag = &result.diagnostics[0];
    let img_start = source.find("<img").unwrap() as u32;
    assert_eq!(diag.start, img_start, "range must start at the <img> tag");
    assert_eq!(&source[diag.start as usize..diag.end as usize][..4], "<img");
}

#[test]
fn ir_no_distracting_elements_fires_on_jsx() {
    let linter = linter_with(Box::new(NoDistractingElements));
    let result = linter.lint_jsx(
        "const A = () => <marquee>hi</marquee>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.warning_count, 1,
        "<marquee> must flag: {:?}",
        result.diagnostics
    );

    let clean = linter.lint_jsx("const A = () => <div>hi</div>;", "test.jsx", JsxLang::Jsx);
    assert_eq!(
        clean.warning_count, 0,
        "<div> must be clean: {:?}",
        clean.diagnostics
    );
}

#[test]
fn ir_no_distracting_elements_skips_components() {
    // A PascalCase JSX tag is a component, never an intrinsic <marquee>.
    let linter = linter_with(Box::new(NoDistractingElements));
    let result = linter.lint_jsx(
        "const A = () => <Marquee>hi</Marquee>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.warning_count, 0,
        "component <Marquee> must be clean: {:?}",
        result.diagnostics
    );
}

#[test]
fn ir_deprecated_element_fires_on_jsx() {
    let linter = linter_with(Box::new(DeprecatedElement));
    let result = linter.lint_jsx(
        "const A = () => <center>hi</center>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.warning_count, 1,
        "<center> must flag: {:?}",
        result.diagnostics
    );
}

#[test]
fn ir_no_autofocus_fires_on_jsx() {
    let linter = linter_with(Box::new(NoAutofocus));
    // Boolean-shorthand prop.
    let shorthand = linter.lint_jsx(
        "const A = () => <input autofocus/>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        shorthand.warning_count, 1,
        "autofocus must flag: {:?}",
        shorthand.diagnostics
    );

    // JSX camelCase + expression value — still the same `autofocus` arg.
    let expr = linter.lint_jsx(
        "const A = () => <input autoFocus={true}/>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        expr.warning_count, 1,
        "autoFocus={{true}} must flag: {:?}",
        expr.diagnostics
    );

    let clean = linter.lint_jsx(
        "const A = () => <input type=\"text\"/>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        clean.warning_count, 0,
        "no autofocus must be clean: {:?}",
        clean.diagnostics
    );
}

#[test]
fn ir_no_access_key_fires_on_jsx() {
    let linter = linter_with(Box::new(NoAccessKey));
    let result = linter.lint_jsx(
        "const A = () => <div accessKey=\"h\">x</div>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.warning_count, 1,
        "accessKey must flag: {:?}",
        result.diagnostics
    );
}

#[test]
fn ir_tabindex_no_positive_fires_on_jsx() {
    let linter = linter_with(Box::new(TabindexNoPositive));
    let positive = linter.lint_jsx(
        "const A = () => <div tabIndex=\"1\">x</div>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        positive.warning_count, 1,
        "tabIndex=\"1\" must flag: {:?}",
        positive.diagnostics
    );

    let zero = linter.lint_jsx(
        "const A = () => <div tabIndex=\"0\">x</div>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        zero.warning_count, 0,
        "tabIndex=\"0\" must be clean: {:?}",
        zero.diagnostics
    );

    let negative = linter.lint_jsx(
        "const A = () => <div tabIndex=\"-1\">x</div>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        negative.warning_count, 0,
        "tabIndex=\"-1\" must be clean: {:?}",
        negative.diagnostics
    );
}

#[test]
fn ir_require_v_for_key_fires_on_jsx_map() {
    // `.map()` repeats without a `key` are the JSX shape of a keyless `v-for`.
    // The markup facade surfaces the repeated <li> and asks for a key binding.
    let linter = linter_with(Box::new(RequireVForKey));
    let missing = linter.lint_jsx(
        "const L = () => <ul>{items.map((item) => <li>{item}</li>)}</ul>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        missing.error_count, 1,
        ".map() without key must flag: {:?}",
        missing.diagnostics
    );

    let keyed = linter.lint_jsx(
        "const L = () => <ul>{items.map((item) => <li key={item.id}>{item}</li>)}</ul>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        keyed.error_count, 0,
        ".map() with key must be clean: {:?}",
        keyed.diagnostics
    );
}

// ===========================================================================
// Each migrated rule fires exactly once (IR pass, no double-report via fallback).
// ===========================================================================

#[test]
fn migrated_rule_reports_once_not_per_backend() {
    // `a11y/img-alt` is markup-capable, so it must run on the IR pass only and
    // never additionally via the lowering fallback — exactly one diagnostic.
    let linter = linter_with(Box::new(ImgAlt));
    let result = linter.lint_jsx(
        r#"const A = () => <img src="/x.jpg"/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated rule must report once, not once per backend: {:?}",
        result.diagnostics
    );
}

// ===========================================================================
// Fallback pass: an unmigrated rule still runs over the lowered relief AST.
// ===========================================================================

#[test]
fn fallback_img_without_alt_is_flagged() {
    // `vue/a11y-img-alt` has only a legacy `Rule` impl (no markup projection),
    // so it is served by the lowering fallback and must still fire.
    let linter = linter_with(Box::new(A11yImgAlt));
    let result = linter.lint_jsx(
        r#"const A = () => <img src="/x.jpg"/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.warning_count, 1,
        "<img> without alt should be flagged via fallback: {:?}",
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
fn fallback_img_with_alt_is_not_flagged() {
    let linter = linter_with(Box::new(A11yImgAlt));
    let result = linter.lint_jsx(
        r#"const A = () => <img src="/x.jpg" alt=""/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.warning_count, 0,
        "<img> with alt should be clean: {:?}",
        result.diagnostics
    );
}

#[test]
fn fallback_tsx_img_without_alt_is_flagged() {
    let linter = linter_with(Box::new(A11yImgAlt));
    let result = linter.lint_jsx(
        r#"const A = () => <img src="/x.jpg"/>;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        result.warning_count, 1,
        "TSX fallback must flag: {:?}",
        result.diagnostics
    );
}

// ===========================================================================
// No JSX equivalent: documented + skipped.
// ===========================================================================

#[test]
fn no_jsx_equivalent_v_html_is_skipped() {
    // `vue/no-v-html` matches the `v-html` directive, which has no JSX analogue
    // (`dangerouslySetInnerHTML` is a different prop entirely). It is not
    // migrated to the markup IR, and the lowering fallback finds no `v-html`
    // directive in JSX, so linting JSX produces no diagnostic. This is the
    // documented "rule with no JSX equivalent is skipped" behavior.
    let linter = linter_with(Box::new(NoVHtml));
    let result = linter.lint_jsx(
        r#"const A = () => <div dangerouslySetInnerHTML={{ __html: x }}/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.error_count + result.warning_count,
        0,
        "no-v-html has no JSX equivalent and must stay silent on JSX: {:?}",
        result.diagnostics
    );
}

// ===========================================================================
// Combined: a markup rule + an unmigrated rule coexist on one lint pass.
// ===========================================================================

#[test]
fn ir_and_fallback_rules_coexist() {
    // ImgAlt (markup IR) and A11yImgAlt (fallback) both target <img> without
    // alt. A single `lint_jsx` call must fire both — one through each path —
    // proving the two passes compose without dropping or duplicating either.
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ImgAlt));
    registry.register(Box::new(A11yImgAlt));
    let linter = Linter::with_registry(registry);

    let result = linter.lint_jsx(
        r#"const A = () => <img src="/x.jpg"/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.warning_count, 2,
        "both the IR rule and the fallback rule must fire once each: {:?}",
        result.diagnostics
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "a11y/img-alt")
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "vue/a11y-img-alt")
    );
}
