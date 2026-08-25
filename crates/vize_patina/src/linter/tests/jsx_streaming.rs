//! Streaming-root witnesses for the direct-IR JSX markup pass.
//!
//! Root discovery streams each outermost JSX element/fragment straight into
//! the markup walker (no per-rule root vector — the allocation side is pinned
//! by the `patina_jsx_markup_one_root` bench budget). These tests pin the
//! behavioral side: multi-root programs report every root in source order,
//! byte-for-byte deterministically, and malformed JSX stays total.

use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::ImgAlt;
use vize_atelier_jsx::JsxLang;
use vize_carton::CompactString;

fn linter_with(rule: Box<dyn Rule>) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry)
}

fn diagnostic_fingerprint(result: &LintResult) -> Vec<(&'static str, u32, u32, CompactString)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.clone(),
            )
        })
        .collect()
}

/// Three roots: a plain element, an `expr && <x/>` guard, and a `.map()`
/// callback. Streaming discovery must yield each of them exactly once, in
/// source order.
const MULTI_ROOT: &str = r#"const A = () => <img src="/a.jpg"/>;
const B = ({ on }) => on && <img src="/b.jpg"/>;
const C = ({ xs }) => xs.map((x) => <img key={x} src="/c.jpg"/>);
"#;

#[test]
fn multi_root_program_reports_every_root_in_source_order() {
    let linter = linter_with(Box::new(ImgAlt));
    let result = linter.lint_jsx(MULTI_ROOT, "multi.jsx", JsxLang::Jsx);

    assert_eq!(
        result.warning_count, 3,
        "each streamed root must be visited exactly once: {:?}",
        result.diagnostics
    );
    let starts: Vec<u32> = result.diagnostics.iter().map(|d| d.start).collect();
    let expected: Vec<u32> = ["/a.jpg", "/b.jpg", "/c.jpg"]
        .iter()
        .map(|needle| {
            let src_pos = MULTI_ROOT.find(needle).expect("fixture names the image");
            MULTI_ROOT[..src_pos]
                .rfind("<img")
                .expect("img precedes src") as u32
        })
        .collect();
    assert_eq!(starts, expected, "roots must report in source order");
}

#[test]
fn multi_root_diagnostics_are_deterministic() {
    let linter = linter_with(Box::new(ImgAlt));
    let first = diagnostic_fingerprint(&linter.lint_jsx(MULTI_ROOT, "multi.jsx", JsxLang::Jsx));
    let second = diagnostic_fingerprint(&linter.lint_jsx(MULTI_ROOT, "multi.jsx", JsxLang::Jsx));
    assert!(!first.is_empty());
    assert_eq!(first, second, "repeat runs must be byte-for-byte identical");
}

#[test]
fn malformed_jsx_stays_total_and_deterministic() {
    // An unclosed element still parses to a program with diagnostics; the
    // streaming pass must neither panic nor drift between runs.
    let source = r#"const A = () => <div><img src="/x.jpg">;"#;
    let linter = linter_with(Box::new(ImgAlt));
    let first = linter.lint_jsx(source, "broken.jsx", JsxLang::Jsx);
    let second = linter.lint_jsx(source, "broken.jsx", JsxLang::Jsx);
    assert_eq!(
        diagnostic_fingerprint(&first),
        diagnostic_fingerprint(&second),
        "malformed input must lint deterministically"
    );
}

#[test]
fn full_registry_multi_root_run_is_deterministic() {
    // Diagnostic parity over the production rule set: whatever the default
    // registry reports on a multi-root module, it reports identically on a
    // second pass — the streaming driver holds rule order and output stable.
    let linter = Linter::with_registry(RuleRegistry::default());
    let first = diagnostic_fingerprint(&linter.lint_jsx(MULTI_ROOT, "multi.jsx", JsxLang::Jsx));
    let second = diagnostic_fingerprint(&linter.lint_jsx(MULTI_ROOT, "multi.jsx", JsxLang::Jsx));
    assert_eq!(first, second);
}
