use super::family;
use crate::diagnostic::LintDiagnostic;
use crate::linter::{LintResult, Linter};
use crate::markup::L2Markup;
use crate::rule::RuleRegistry;
use vize_atelier_jsx::{
    JsxLang, lower_source_with_lint_syntax, lower_source_with_lint_syntax_for_vdom,
};
use vize_l0::Allocator;

fn combined(original: bool) -> Linter {
    let mut registry = RuleRegistry::new();
    for rule in family(original) {
        registry.register(rule);
    }
    Linter::with_registry(registry)
}

fn refusal(filename: &str) -> LintResult {
    LintResult {
        filename: filename.into(),
        diagnostics: vec![LintDiagnostic::error(
            "parser/jsx",
            "Original element syntax is unavailable for selected lint rules; use the source lint entry point.",
            0,
            0,
        )],
        error_count: 1,
        warning_count: 0,
    }
}

#[test]
fn a_plain_equal_source_projection_cannot_impersonate_the_qualified_factory_view() {
    let source = "const A = () => <component is={tag}/>;";
    let arena = Allocator::new();
    let syntax = lower_source_with_lint_syntax(&arena, arena.as_oxc(), source, JsxLang::Tsx);
    let root = syntax.roots().next().unwrap();
    let raw = L2Markup::from_projected_root(root.lowered().l2.as_ref().unwrap());
    let actual = combined(false).lint_markup(source, "Elements.tsx", &raw);
    assert_eq!(
        format!("{actual:?}"),
        format!("{:?}", refusal("Elements.tsx"))
    );
    let qualified = L2Markup::from_syntax_root(root).unwrap();
    assert_eq!(
        format!(
            "{:?}",
            combined(false).lint_markup(source, "Elements.tsx", &qualified)
        ),
        format!(
            "{:?}",
            combined(true).lint_jsx(source, "Elements.tsx", JsxLang::Tsx)
        ),
    );
    for foreign in [source.to_owned(), source.replace("tag", "alt")] {
        assert!(!core::ptr::eq(source, foreign.as_str()));
        assert_eq!(
            format!(
                "{:?}",
                combined(false).lint_markup(&foreign, "Foreign.tsx", &qualified)
            ),
            format!("{:?}", refusal("Foreign.tsx")),
        );
    }
    let disabled = combined(false)
        .with_enabled_rules(Some(vec![]))
        .lint_markup(source, "Elements.tsx", &raw);
    assert!(disabled.diagnostics.is_empty());
    assert_eq!((disabled.error_count, disabled.warning_count), (0, 0));
}

#[test]
fn native_identity_facts_and_retained_tree_depth_keep_original_packets() {
    for source in [
        "const A = () => <component is={tag}/>;",
        "const A = () => <Comp v-slots={{ head: () => <div><span/></div> }}/>;",
    ] {
        let arena = Allocator::new();
        let output =
            lower_source_with_lint_syntax_for_vdom(&arena, arena.as_oxc(), source, JsxLang::Tsx);
        let root = output.roots().next().unwrap();
        assert!(root.lowered().l2.as_ref().unwrap().native, "{source}");
        let qualified = L2Markup::from_syntax_root(root).unwrap();
        assert_eq!(
            format!(
                "{:?}",
                combined(false).lint_markup(source, "Elements.tsx", &qualified)
            ),
            format!(
                "{:?}",
                combined(true).lint_jsx(source, "Elements.tsx", JsxLang::Tsx)
            ),
            "{source}",
        );
    }
}

#[test]
fn conditional_fragments_keep_the_explicit_original_tree_fallback() {
    let source = "const A = () => <div>{ok ? <><span/><div/></> : <span/>}</div>;";
    let arena = Allocator::new();
    for output in [
        lower_source_with_lint_syntax(&arena, arena.as_oxc(), source, JsxLang::Tsx),
        lower_source_with_lint_syntax_for_vdom(&arena, arena.as_oxc(), source, JsxLang::Tsx),
    ] {
        let root = output.roots().next().unwrap();
        assert!(root.lowered().l2.is_err());
        assert!(L2Markup::from_syntax_root(root).is_none());
    }
    assert_eq!(
        format!(
            "{:?}",
            combined(false).lint_jsx(source, "Elements.tsx", JsxLang::Tsx)
        ),
        format!(
            "{:?}",
            combined(true).lint_jsx(source, "Elements.tsx", JsxLang::Tsx)
        ),
    );
}

#[test]
fn original_and_prepared_tsrx_regions_keep_fixed_complete_packets_and_raw_views_refuse() {
    use vize_l1_to_l2::lower::tsrx::{RegionLowering, with_prepared};
    let source = "export function Page() @{\n  <div><span/></div>\n}\n";
    let div = source.find("<div>").unwrap() as u32;
    let end = source.find("</div>").unwrap() as u32 + "</div>".len() as u32;
    let span = source.find("<span/>").unwrap() as u32;
    let expected = LintResult {
        filename: "Elements.tsrx".into(),
        diagnostics: vec![
            LintDiagnostic::warn(
                "vue/no-restricted-html-elements",
                "Unexpected use of forbidden element div.",
                div,
                end,
            ),
            LintDiagnostic::warn(
                "vue/max-template-depth",
                "Element is nested too deeply (depth of 2, maximum allowed is 1).",
                span,
                span + 7,
            ),
            LintDiagnostic::warn(
                "vue/no-restricted-html-elements",
                "Use a supported element.",
                span,
                span + 7,
            ),
        ],
        error_count: 0,
        warning_count: 3,
    };
    let configured = combined(false);
    assert_eq!(
        format!("{:?}", configured.lint_tsrx(source, "Elements.tsrx")),
        format!("{expected:?}")
    );
    with_prepared(source, |prepared| {
        assert_eq!(
            format!(
                "{:?}",
                configured.lint_prepared_tsrx(&prepared, "Elements.tsrx")
            ),
            format!("{expected:?}")
        );
        let RegionLowering::Ready(artifacts) = prepared.regions() else {
            panic!("expected supported TSRX");
        };
        let region = artifacts.regions().next().unwrap();
        let raw = L2Markup::from_artifact(region.artifact());
        assert_eq!(
            format!(
                "{:?}",
                configured.lint_markup(source, "Elements.tsrx", &raw)
            ),
            format!("{:?}", refusal("Elements.tsrx"))
        );
    });
}
