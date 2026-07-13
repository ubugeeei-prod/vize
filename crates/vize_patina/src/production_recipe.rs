//! Production Patina recipes over shared SFC and JSX/TSX artifacts.

use vize_atelier_jsx::JsxSyntaxProduct;
use vize_atelier_sfc::SfcDescriptorProduct;
use vize_atlas::{
    Compilation, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
    RegisterProviderError, Shared,
};
use vize_croquis::CroquisDocumentProduct;
use vize_relief::ReliefProduct;

use crate::{LintResult, Linter};

/// Complete configured Patina result for one document.
pub struct PatinaDocumentReportProduct;

impl Product for PatinaDocumentReportProduct {
    type Value = LintResult;

    const NAME: &'static str = "patina.document-report";
}

/// Configured production linter consuming frontend-owned syntax and complete
/// semantic products instead of parsing its own document.
pub struct PatinaDocumentProvider {
    linter: Shared<Linter>,
}

impl PatinaDocumentProvider {
    pub fn new(linter: Linter) -> Self {
        Self::from_shared(Shared::new(linter))
    }

    /// Reuse one configured linter across parallel Atlas query sessions.
    pub fn from_shared(linter: Shared<Linter>) -> Self {
        Self { linter }
    }
}

impl Provider for PatinaDocumentProvider {
    type Product = PatinaDocumentReportProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        let name = context.source().name();
        name.ends_with(".vue") || name.ends_with(".jsx") || name.ends_with(".tsx")
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        if context.source().name().ends_with(".vue") {
            vec![
                ProductId::of::<SfcDescriptorProduct>(),
                ProductId::of::<ReliefProduct>(),
                ProductId::of::<CroquisDocumentProduct>(),
            ]
        } else {
            vec![
                ProductId::of::<JsxSyntaxProduct>(),
                ProductId::of::<CroquisDocumentProduct>(),
            ]
        }
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<LintResult, ProviderError> {
        if !context.source().name().ends_with(".vue") {
            let syntax = context.get::<JsxSyntaxProduct>()?;
            let semantics = context.get::<CroquisDocumentProduct>()?;
            return Ok(self.linter.lint_jsx_with_shared_artifacts(
                context.source().text(),
                context.source().name(),
                syntax.as_ref(),
                Some(semantics.analysis()),
            ));
        }
        let descriptor_artifact = context.get::<SfcDescriptorProduct>()?;
        let syntax = context.get::<ReliefProduct>()?;
        let semantics = context.get::<CroquisDocumentProduct>()?;
        let descriptor = match descriptor_artifact.as_result() {
            Ok(descriptor) => descriptor,
            Err(error) => {
                return Ok(self.linter.lint_sfc_with_shared_parse_error(
                    context.source().text(),
                    context.source().name(),
                    error,
                ));
            }
        };
        let template_syntax = match (descriptor.template.as_ref(), syntax.as_ref()) {
            (Some(_), Some(syntax)) => Some((syntax.snapshot(), syntax.parse_diagnostics())),
            (None, None) => None,
            _ => {
                return Err(ProviderError::message(
                    "SFC descriptor and Relief syntax disagree about template presence",
                ));
            }
        };
        Ok(self.linter.lint_sfc_with_shared_artifacts(
            context.source().text(),
            context.source().name(),
            descriptor,
            template_syntax,
            Some(semantics.analysis()),
        ))
    }
}

/// Register one configured production Patina root.
pub fn register_document_lint_recipe(
    compilation: &mut Compilation,
    linter: Linter,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(PatinaDocumentProvider::new(linter))
}

/// Register a production Patina root backed by one shared configured linter.
pub fn register_shared_document_lint_recipe(
    compilation: &mut Compilation,
    linter: Shared<Linter>,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(PatinaDocumentProvider::from_shared(linter))
}

#[cfg(test)]
mod tests {
    use vize_atelier_jsx::JsxSyntaxProduct;
    use vize_atlas::{ProductStatus, SourceId};
    use vize_croquis::{CroquisDocumentProduct, CroquisSemanticProduct};
    use vize_relief::{ReliefProduct, TransformedReliefProduct, VueDialectInput};

    use super::*;

    const SOURCE: &str = r#"<script setup>const items = []</script>
<template><p v-for="(item, index) in items">{{ item }}</p></template>"#;

    fn source(compilation: &mut Compilation) -> SourceId {
        compilation.add_source("Parity.vue", SOURCE).unwrap()
    }

    #[test]
    fn production_recipe_matches_direct_linter_without_render_or_transform() {
        let expected_linter = Linter::new();
        let expected = expected_linter.lint_sfc(SOURCE, "Parity.vue");
        let mut compilation = Compilation::new();
        compilation
            .set_input::<VueDialectInput>(vize_carton::config::VueVersion::V3)
            .unwrap();
        vize_atelier_sfc::register_atlas_providers(&mut compilation).unwrap();
        register_document_lint_recipe(&mut compilation, Linter::new()).unwrap();
        let source = source(&mut compilation);

        let outcome = compilation
            .query::<PatinaDocumentReportProduct>(source)
            .unwrap();

        assert_eq!(outcome.status(), ProductStatus::Executed);
        assert_eq!(outcome.value().error_count, expected.error_count);
        assert_eq!(outcome.value().warning_count, expected.warning_count);
        assert_eq!(
            vize_carton::cstr!("{:?}", outcome.value().diagnostics),
            vize_carton::cstr!("{:?}", expected.diagnostics)
        );
        assert!(outcome.plan().contains::<ReliefProduct>());
        assert!(outcome.plan().contains::<CroquisDocumentProduct>());
        assert!(!outcome.plan().contains::<TransformedReliefProduct>());
        assert!(
            outcome
                .plan()
                .products()
                .iter()
                .all(|product| product.name() != "rendu.hir")
        );
        assert!(!outcome.plan().contains::<CroquisSemanticProduct>());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn type_aware_recipe_reuses_relief_and_croquis_with_exact_diagnostics() {
        fn linter() -> Linter {
            Linter::with_preset(crate::LintPreset::Incremental)
                .with_rule(Box::new(crate::rules::type_aware::RequireTypedProps::new()))
        }

        let source_text = r#"<script setup lang="ts">defineProps(['msg'])</script>
<template><p>{{ msg }}</p></template>"#;
        let expected = linter().lint_sfc(source_text, "Typed.vue");
        crate::linter::reset_fallback_builds();
        let mut compilation = Compilation::new();
        compilation
            .set_input::<VueDialectInput>(vize_carton::config::VueVersion::V3)
            .unwrap();
        vize_atelier_sfc::register_atlas_providers(&mut compilation).unwrap();
        register_document_lint_recipe(&mut compilation, linter()).unwrap();
        let source = compilation.add_source("Typed.vue", source_text).unwrap();

        let outcome = compilation
            .query::<PatinaDocumentReportProduct>(source)
            .unwrap();

        assert!(outcome.plan().contains::<ReliefProduct>());
        assert!(outcome.plan().contains::<CroquisDocumentProduct>());
        assert!(!outcome.plan().contains::<TransformedReliefProduct>());
        assert_eq!(
            crate::linter::fallback_builds(),
            (0, 0),
            "Atlas type-aware lint must materialize Relief and reuse Croquis",
        );
        assert_eq!(outcome.value().error_count, expected.error_count);
        assert_eq!(outcome.value().warning_count, expected.warning_count);
        assert_eq!(
            vize_carton::cstr!("{:?}", outcome.value().diagnostics),
            vize_carton::cstr!("{:?}", expected.diagnostics)
        );
    }

    fn jsx_linter() -> Linter {
        let mut rules = crate::RuleRegistry::new();
        rules.register(Box::new(crate::rules::a11y::ImgAlt));
        rules.register(Box::new(crate::rules::vue::RequireVForKey));
        Linter::with_registry(rules)
    }

    #[test]
    fn jsx_recipe_parses_once_without_relief_and_reuses_cache() {
        let source_text = "const App = () => <img />;";
        let expected =
            jsx_linter().lint_jsx(source_text, "App.jsx", vize_atelier_jsx::JsxLang::Jsx);
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
}
