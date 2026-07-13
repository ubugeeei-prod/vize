//! Production Patina recipe for raw JavaScript and TypeScript modules.

use vize_atlas::{
    Compilation, PlanningContext, ProductId, Provider, ProviderContext, ProviderError,
    RegisterProviderError, Shared,
};
use vize_module::ModuleSyntaxProduct;

use super::{PatinaDocumentReportProduct, PatinaLinterInput};
use crate::{LintResult, Linter};

/// Patina report provider consuming the source-faithful Module product.
pub struct PatinaModuleLintProvider {
    linter: Shared<Linter>,
}

impl PatinaModuleLintProvider {
    pub fn from_shared(linter: Shared<Linter>) -> Self {
        Self { linter }
    }
}

impl Provider for PatinaModuleLintProvider {
    type Product = PatinaDocumentReportProduct;

    fn input_dependencies(&self) -> Vec<vize_atlas::InputId> {
        vec![vize_atlas::InputId::of::<PatinaLinterInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_raw_module_name(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<ModuleSyntaxProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<LintResult, ProviderError> {
        let modules = context.get::<ModuleSyntaxProduct>()?;
        let [module] = modules.modules.as_slice() else {
            return Err(ProviderError::message(
                "raw-module lint requires exactly one Module syntax unit",
            ));
        };
        let linter = context
            .input::<PatinaLinterInput>()
            .map(Shared::clone)
            .unwrap_or_else(|| Shared::clone(&self.linter));
        Ok(linter.lint_script_with_shared_artifacts(module, context.source().name()))
    }
}

/// Register the Module-backed Patina report root.
pub fn register_shared_module_lint_recipe(
    compilation: &mut Compilation,
    linter: Shared<Linter>,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(PatinaModuleLintProvider::from_shared(linter))
}

fn is_raw_module_name(name: &str) -> bool {
    let clean = name.split(['?', '#']).next().unwrap_or(name);
    matches!(
        clean.rsplit_once('.').map(|(_, extension)| extension),
        Some("js" | "mjs" | "cjs" | "ts" | "mts" | "cts")
    )
}

#[cfg(test)]
mod tests {
    use vize_atlas::{Compilation, ProductStatus};

    use super::*;

    #[test]
    fn module_report_matches_direct_linter_and_reuses_module_product() {
        let text = "import { ref } from '@vue/reactivity'; export const count = ref(0);";
        let linter = Shared::new(Linter::with_preset(crate::LintPreset::Opinionated));
        let expected = linter.lint_script(text, "state.ts");
        let mut compilation = Compilation::new();
        vize_module::register_raw_providers(&mut compilation).unwrap();
        register_shared_module_lint_recipe(&mut compilation, Shared::clone(&linter)).unwrap();
        let source = compilation.add_source("state.ts", text).unwrap();
        let snapshot = compilation.snapshot();
        let mut session = snapshot.query_session();

        let first = session
            .query::<PatinaDocumentReportProduct>(source)
            .unwrap();
        let second = session
            .query::<PatinaDocumentReportProduct>(source)
            .unwrap();

        assert_eq!(first.status(), ProductStatus::Executed);
        assert_eq!(second.status(), ProductStatus::CacheHit);
        assert!(first.plan().contains::<ModuleSyntaxProduct>());
        assert_eq!(
            vize_carton::cstr!("{:?}", first.value()),
            vize_carton::cstr!("{expected:?}")
        );
        assert_eq!(
            session
                .counters()
                .for_product::<ModuleSyntaxProduct>()
                .executions(),
            1
        );
    }
}
