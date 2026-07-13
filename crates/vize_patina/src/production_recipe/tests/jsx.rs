//! JSX and TSX production-recipe integration tests.

use vize_atelier_jsx::JsxSyntaxProduct;
use vize_atlas::{Compilation, ProductStatus};
use vize_croquis::CroquisDocumentProduct;
use vize_relief::ReliefProduct;

use super::super::{PatinaDocumentReportProduct, register_document_lint_recipe};
use crate::Linter;

fn jsx_linter() -> Linter {
    let mut rules = crate::RuleRegistry::new();
    rules.register(Box::new(crate::rules::a11y::ImgAlt));
    rules.register(Box::new(crate::rules::vue::RequireVForKey));
    Linter::with_registry(rules)
}

#[test]
fn jsx_recipe_parses_once_without_relief_and_reuses_cache() {
    let source_text = "const App = () => <img />;";
    let expected = jsx_linter().lint_jsx(source_text, "App.jsx", vize_atelier_jsx::JsxLang::Jsx);
    let mut compilation = Compilation::new();
    vize_atelier_jsx::register_atlas_providers(&mut compilation).unwrap();
    register_document_lint_recipe(&mut compilation, jsx_linter()).unwrap();
    let source = compilation.add_source("App.jsx", source_text).unwrap();
    let plan = compilation
        .plan_for::<PatinaDocumentReportProduct>(source)
        .unwrap();

    assert!(plan.contains::<JsxSyntaxProduct>());
    assert!(plan.contains::<CroquisDocumentProduct>());
    assert!(!plan.contains::<ReliefProduct>());
    let snapshot = compilation.snapshot();
    let mut session = snapshot.query_session();
    let first = session
        .query::<PatinaDocumentReportProduct>(source)
        .unwrap();
    let second = session
        .query::<PatinaDocumentReportProduct>(source)
        .unwrap();
    assert_eq!(
        first.value().warning_count,
        expected.warning_count,
        "{:?}",
        first.value().diagnostics
    );
    assert_eq!(first.value().error_count, expected.error_count);
    assert_eq!(second.status(), ProductStatus::CacheHit);
    assert_eq!(
        session
            .counters()
            .for_product::<JsxSyntaxProduct>()
            .executions(),
        1
    );
    assert_eq!(
        session
            .counters()
            .for_product::<PatinaDocumentReportProduct>()
            .cache_hits(),
        1
    );
}

#[test]
fn tsx_recipe_uses_owned_syntax_projection() {
    let mut compilation = Compilation::new();
    vize_atelier_jsx::register_atlas_providers(&mut compilation).unwrap();
    register_document_lint_recipe(&mut compilation, jsx_linter()).unwrap();
    let source = compilation
        .add_source(
            "App.tsx",
            "const App = (p: Props): JSX.Element => <img src={p.src} />;",
        )
        .unwrap();

    let outcome = compilation
        .query::<PatinaDocumentReportProduct>(source)
        .unwrap();

    assert_eq!(outcome.value().warning_count, 1);
    assert!(outcome.plan().contains::<JsxSyntaxProduct>());
    assert!(!outcome.plan().contains::<ReliefProduct>());
}

#[test]
fn jsx_structural_rule_consumes_owned_for_projection() {
    let mut rules = crate::RuleRegistry::new();
    rules.register(Box::new(crate::rules::vue::RequireVForKey));
    let mut compilation = Compilation::new();
    vize_atelier_jsx::register_atlas_providers(&mut compilation).unwrap();
    register_document_lint_recipe(&mut compilation, Linter::with_registry(rules)).unwrap();
    let source = compilation
        .add_source(
            "List.jsx",
            "const List = () => <ul>{items.map(item => <li>{item}</li>)}</ul>;",
        )
        .unwrap();

    let outcome = compilation
        .query::<PatinaDocumentReportProduct>(source)
        .unwrap();

    assert_eq!(outcome.value().error_count, 1);
    assert!(!outcome.plan().contains::<ReliefProduct>());
}

#[test]
fn jsx_recipe_preserves_fixable_trivia_diagnostics() {
    let mut rules = crate::RuleRegistry::new();
    rules.register(Box::new(crate::rules::vue::NoMultiSpaces::default()));
    let mut compilation = Compilation::new();
    vize_atelier_jsx::register_atlas_providers(&mut compilation).unwrap();
    register_document_lint_recipe(&mut compilation, Linter::with_registry(rules)).unwrap();
    let source_text = "const App = () => <div    class=\"a\">x</div>;";
    let source = compilation.add_source("App.tsx", source_text).unwrap();

    let outcome = compilation
        .query::<PatinaDocumentReportProduct>(source)
        .unwrap();

    assert_eq!(outcome.value().warning_count, 1);
    let diagnostic = &outcome.value().diagnostics[0];
    assert_eq!(diagnostic.rule_name, "vue/no-multi-spaces");
    let edit = &diagnostic.fix.as_ref().unwrap().edits[0];
    assert_eq!(&source_text[edit.start as usize..edit.end as usize], "    ");
    assert_eq!(edit.new_text.as_str(), " ");
    assert!(outcome.plan().contains::<JsxSyntaxProduct>());
    assert!(!outcome.plan().contains::<ReliefProduct>());
}
