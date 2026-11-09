//! petite-vue dialect lint rules.
//!
//! These rules only fire on documents detected as petite-vue (via
//! `ctx.is_petite_vue()`); they are opt-in and belong to no preset, so they
//! never affect normal Vue SFC linting.

mod no_unsupported_directive;
mod valid_v_effect;
mod valid_v_scope;

use crate::rule::{Rule, RuleRegistry};

pub use no_unsupported_directive::NoUnsupportedDirective;
pub use valid_v_effect::ValidVEffect;
pub use valid_v_scope::ValidVScope;

/// Register petite-vue rules as explicit opt-in rules.
pub(crate) fn register_opt_in(registry: &mut RuleRegistry) {
    register_if_missing(registry, Box::new(NoUnsupportedDirective));
    register_if_missing(registry, Box::new(ValidVEffect));
    register_if_missing(registry, Box::new(ValidVScope));
}

fn register_if_missing(registry: &mut RuleRegistry, rule: Box<dyn Rule>) {
    if !registry.has_rule(rule.meta().name) {
        registry.register(rule);
    }
}

#[cfg(test)]
mod level_parity_tests {
    use super::{ValidVEffect, ValidVScope};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVEffect));
        registry.register(Box::new(ValidVScope));
        Linter::with_registry(registry)
    }

    fn document(markup: &str) -> String {
        format!(
            concat!(
                "<!DOCTYPE html><html><body>{markup}",
                "<script src=\"https://unpkg.com/petite-vue\" init></script>",
                "</body></html>"
            ),
            markup = markup
        )
    }

    #[test]
    fn admitted_level_diagnostics_match_the_shipped_relief_route() {
        let linter = linter();
        let mut observed = 0;
        for markup in [
            r#"<div v-scope="count" v-effect></div>"#,
            r#"<div v-scope="{ label: '&amp;' }" v-effect="label = '&amp;'"></div>"#,
            r#"<div v-scope="{ value: }" v-effect="   "></div>"#,
            r#"<main v-scope><button v-effect=""></button></main>"#,
            r#"<div v-scope:state.deep="items" v-effect.once="items++"></div>"#,
            r#"<section V-SCOPE="ignored" V-EFFECT></section>"#,
        ] {
            let source = document(markup);
            let levels = linter.lint_standalone_html(&source, "index.html");
            let relief = linter.lint_standalone_html_relief_for_test(&source, "index.html");
            observed += levels.diagnostics.len();
            assert_eq!(
                format!("{levels:#?}"),
                format!("{relief:#?}"),
                "route mismatch for {markup}"
            );
        }
        assert!(
            observed > 0,
            "the differential corpus must exercise reports"
        );
    }

    #[test]
    fn refused_level_document_preserves_the_relief_fallback() {
        let source = concat!(
            "<!DOCTYPE html><html><body>",
            "<script src=\"https://unpkg.com/petite-vue\" init></script>",
            "<div v-scope=\"count"
        );
        let allocator = vize_l0::Allocator::default();
        let owner = vize_l1::markup::document::NativeDocument::lex_in(
            &allocator,
            vize_l0::SourceRoot::new(source).unwrap(),
        );
        assert!(
            vize_l2::dialect::petite::PetiteDocument::lower_in(&allocator, &owner).is_err(),
            "the fallback witness must be refused by the whole native document"
        );

        let linter = linter();
        let levels = linter.lint_standalone_html(source, "index.html");
        let relief = linter.lint_standalone_html_relief_for_test(source, "index.html");
        assert_eq!(format!("{levels:#?}"), format!("{relief:#?}"));
    }
}
