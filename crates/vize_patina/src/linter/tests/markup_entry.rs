//! `Linter::lint_markup`: one entry for every L2 markup artifact.

use crate::linter::{LintResult, Linter};
use crate::markup::L2Markup;
use crate::rule::RuleRegistry;
use crate::rules::{a11y::ImgAlt, vue::NoMultiSpaces};
use vize_atelier_jsx::JsxLang;
use vize_l0::Allocator;

fn linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ImgAlt));
    registry.register(Box::new(NoMultiSpaces::default()));
    Linter::with_registry(registry)
}

fn rows(result: &LintResult) -> Vec<(&str, u32, u32)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.start, diagnostic.end))
        .collect()
}

#[test]
fn a_jsx_root_artifact_lints_like_lint_jsx() {
    let linter = linter();
    let source = "const A = () => <div  id=\"x\"><img src=\"a.png\" />{a  +  b}</div>;";
    let allocator = Allocator::new();
    let lowered =
        vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);
    let [root] = lowered.roots.as_slice() else {
        panic!("one render root");
    };
    let projected = root.l2.as_ref().expect("the root projects to L2");
    let markup = linter.lint_markup(source, "a.jsx", &L2Markup::from_projected_root(projected));
    let jsx = linter.lint_jsx(source, "a.jsx", JsxLang::Jsx);

    assert_eq!(markup.diagnostics.len(), 4, "{:?}", markup.diagnostics);
    let mut expected = rows(&jsx);
    expected.sort();
    let mut actual = rows(&markup);
    actual.sort();
    assert_eq!(actual, expected);
}

#[test]
fn a_sealed_artifact_lints_as_embedded_markup() {
    let linter = linter();
    let source = "<img src=\"a.png\">";
    let allocator = Allocator::new();
    let (tree, _, errors) = vize_l1::markup::parse_with_authored_diagnostics(&allocator, source);
    let lowered = vize_l1_to_l2::lower_preserving_comments(&allocator, &tree, &errors);
    let artifact =
        vize_l2::artifact::Artifact::try_new(lowered.artifact).expect("a valid sealed artifact");
    let markup = L2Markup::from_artifact(&artifact);
    assert!(!markup.is_template());

    let result = linter.lint_markup(source, "a.tsrx", &markup);
    assert_eq!(rows(&result), vec![("a11y/img-alt", 0, 17)]);
}
