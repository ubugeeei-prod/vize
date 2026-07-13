//! Production Patina recipes over shared SFC and JSX/TSX artifacts.

mod mode;
mod registration;
mod script;
mod template;

pub use mode::{PatinaDocumentMode, PatinaDocumentModeInput, install_document_mode};
pub use registration::{
    install_document_linter, register_document_lint_recipe, register_shared_document_lint_recipe,
};
pub use script::{PatinaModuleLintProvider, register_shared_module_lint_recipe};
pub use template::{
    PatinaTemplateDocumentKind, PatinaTemplateLintInput, PatinaTemplateLintProvider,
    PatinaTemplateLintRequest, install_template_lint_request, register_shared_template_lint_recipe,
};

use vize_atelier_jsx::JsxSyntaxProduct;
use vize_atelier_sfc::{SfcDescriptorProduct, sfc_source_structure};
use vize_atlas::{
    CompilationInput, InputId, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, Shared, SourceInputId,
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

/// Active production linter configuration for an Atlas compilation.
///
/// Hosts may replace this input when workspace configuration changes. Atlas
/// then invalidates only Patina report products, while retaining the shared
/// frontend and semantic artifacts they consume.
pub struct PatinaLinterInput;

impl CompilationInput for PatinaLinterInput {
    type Value = Shared<Linter>;

    const NAME: &'static str = "patina.linter";
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

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<PatinaLinterInput>()]
    }

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<PatinaDocumentModeInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        let name = context.source().name();
        name.ends_with(".vue") || name.ends_with(".jsx") || name.ends_with(".tsx")
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        if mode::is_disabled_in_plan(context) {
            return Vec::new();
        }
        if context.source().name().ends_with(".vue") {
            let mut dependencies = vec![
                ProductId::of::<SfcDescriptorProduct>(),
                ProductId::of::<CroquisDocumentProduct>(),
            ];
            if sfc_source_structure(context.source().text()).has_template {
                dependencies.push(ProductId::of::<ReliefProduct>());
            }
            dependencies
        } else {
            vec![
                ProductId::of::<JsxSyntaxProduct>(),
                ProductId::of::<CroquisDocumentProduct>(),
            ]
        }
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<LintResult, ProviderError> {
        let linter = context
            .input::<PatinaLinterInput>()
            .map(Shared::clone)
            .unwrap_or_else(|| Shared::clone(&self.linter));
        if mode::is_disabled_in_provider(context) {
            return Ok(LintResult {
                filename: context.source().name().into(),
                diagnostics: Vec::new(),
                error_count: 0,
                warning_count: 0,
            });
        }
        if !context.source().name().ends_with(".vue") {
            let syntax = context.get::<JsxSyntaxProduct>()?;
            let semantics = context.get::<CroquisDocumentProduct>()?;
            return Ok(linter.lint_jsx_with_shared_artifacts(
                context.source().text(),
                context.source().name(),
                syntax.as_ref(),
                Some(semantics.analysis()),
            ));
        }
        let descriptor_artifact = context.get::<SfcDescriptorProduct>()?;
        let semantics = context.get::<CroquisDocumentProduct>()?;
        let descriptor = match descriptor_artifact.as_result() {
            Ok(descriptor) => descriptor,
            Err(error) => {
                return Ok(linter.lint_sfc_with_shared_parse_error(
                    context.source().text(),
                    context.source().name(),
                    error,
                ));
            }
        };
        let syntax = if descriptor.template.is_some() {
            Some(context.get::<ReliefProduct>()?)
        } else {
            None
        };
        let template_syntax = match (descriptor.template.as_ref(), syntax.as_ref()) {
            (Some(_), Some(syntax)) => {
                let Some(syntax) = syntax.as_ref() else {
                    return Err(ProviderError::message(
                        "SFC descriptor and Relief syntax disagree about template presence",
                    ));
                };
                Some((syntax.snapshot(), syntax.parse_diagnostics()))
            }
            (None, None) => None,
            _ => {
                return Err(ProviderError::message(
                    "SFC descriptor and Relief syntax disagree about template presence",
                ));
            }
        };
        Ok(linter.lint_sfc_with_shared_artifacts(
            context.source().text(),
            context.source().name(),
            descriptor,
            template_syntax,
            Some(semantics.analysis()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use vize_atlas::Compilation;
    use vize_atlas::{ProductStatus, SourceId};
    use vize_croquis::{CroquisDocumentProduct, CroquisSemanticProduct};
    use vize_relief::{ReliefProduct, TransformedReliefProduct, VueDialectInput};

    use super::*;

    mod jsx;

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
}
