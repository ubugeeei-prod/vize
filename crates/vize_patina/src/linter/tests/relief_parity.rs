//! Each migrated rule reports identically through its Relief hooks and its
//! markup facade, on bare templates, SFCs and JSX/TSX.

use crate::context::LintContext;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleMeta, RuleRegistry};
use vize_atelier_jsx::JsxLang;
use vize_relief::{DirectiveNode, ElementNode, ForNode, IfNode, InterpolationNode, RootNode};

mod a11y;
mod all_hooks;

/// A rule with its markup entry hidden, so every lane drives its Relief hooks.
struct ReliefOnly(Box<dyn Rule>);

impl Rule for ReliefOnly {
    fn meta(&self) -> &'static RuleMeta {
        self.0.meta()
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        self.0.run_on_sfc(ctx);
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        self.0.run_on_template(ctx, root);
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        self.0.enter_element(ctx, element);
    }

    fn exit_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        self.0.exit_element(ctx, element);
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        self.0.check_directive(ctx, element, directive);
    }

    fn check_for<'a>(&self, ctx: &mut LintContext<'a>, for_node: &ForNode<'a>) {
        self.0.check_for(ctx, for_node);
    }

    fn check_if<'a>(&self, ctx: &mut LintContext<'a>, if_node: &IfNode<'a>) {
        self.0.check_if(ctx, if_node);
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        self.0.check_interpolation(ctx, interpolation);
    }
}

fn linter(rule: Box<dyn Rule>) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry)
}

/// The rule's complete diagnostics, fixes and help included.
fn rendered(result: &LintResult, rule: &str) -> String {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == rule)
        .collect();
    format!("{diagnostics:?}")
}

/// Compare every rule `family` builds on every template and JSX source.
fn assert_parity(family: fn() -> Vec<Box<dyn Rule>>, templates: &[&str], jsx: &[&str]) {
    let corpus = jsx_corpus();
    let mut mismatches = Vec::new();
    let mut checked = 0;
    for ((rule, baseline), all_hooks) in family().into_iter().zip(family()).zip(family()) {
        let name = rule.meta().name;
        assert!(
            rule.as_markup_rule().is_some(),
            "{name} has no markup entry"
        );
        let facade = linter(rule);
        let relief = linter(Box::new(ReliefOnly(baseline)));
        let all_hooks = linter(Box::new(all_hooks::AllHooks(all_hooks)));
        let mut compare = |lane: &str, source: &str, actual: LintResult, expected: LintResult| {
            let (actual, expected) = (rendered(&actual, name), rendered(&expected, name));
            if actual != expected {
                mismatches.push(format!(
                    "{name} {lane}: {source}\n  facade: {actual}\n  expected: {expected}"
                ));
            }
            checked += 1;
        };
        for source in templates {
            compare(
                "template",
                source,
                facade.lint_template(source, "test.vue"),
                relief.lint_template(source, "test.vue"),
            );
            compare(
                "template/all-hooks",
                source,
                facade.lint_template(source, "test.vue"),
                all_hooks.lint_template(source, "test.vue"),
            );
            let sfc = format!(
                "<script setup>const note = '日本語😀'</script>\r\n<template>{source}</template>"
            );
            compare(
                "sfc",
                source,
                facade.lint_sfc(&sfc, "test.vue"),
                relief.lint_sfc(&sfc, "test.vue"),
            );
            compare(
                "sfc/all-hooks",
                source,
                facade.lint_sfc(&sfc, "test.vue"),
                all_hooks.lint_sfc(&sfc, "test.vue"),
            );
        }
        let snippets = jsx.iter().flat_map(|source| {
            [(JsxLang::Jsx, "test.jsx"), (JsxLang::Tsx, "test.tsx")]
                .map(|(lang, filename)| (*source, lang, filename))
        });
        let files = corpus
            .iter()
            .map(|(source, lang, filename)| (source.as_str(), *lang, *filename));
        for (source, lang, filename) in snippets.chain(files) {
            compare(
                filename,
                source,
                facade.lint_jsx(source, filename, lang),
                relief.lint_jsx(source, filename, lang),
            );
            compare(
                "jsx/all-hooks",
                source,
                facade.lint_jsx(source, filename, lang),
                all_hooks.lint_jsx(source, filename, lang),
            );
        }
    }
    assert!(checked > 0);
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// Every `.jsx` / `.tsx` file in the JSX differential corpus.
fn jsx_corpus() -> Vec<(String, JsxLang, &'static str)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/jsx");
    let mut paths: Vec<_> = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == "jsx" || ext == "tsx")
        })
        .collect();
    paths.sort();
    assert!(paths.len() >= 15, "{paths:?}");
    paths
        .into_iter()
        .map(|path| {
            let tsx = path.extension().is_some_and(|ext| ext == "tsx");
            let source = std::fs::read_to_string(&path).unwrap();
            if tsx {
                (source, JsxLang::Tsx, "test.tsx")
            } else {
                (source, JsxLang::Jsx, "test.jsx")
            }
        })
        .collect()
}
