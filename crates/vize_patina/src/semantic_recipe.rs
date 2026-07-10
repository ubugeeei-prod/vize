//! Graph-native linting over the owned Croquis semantic product.
//!
//! This path is deliberately parser-free: frontend providers construct and
//! cache [`CroquisSemanticProduct`], while Patina only consumes its stable
//! semantic snapshot.

use vize_atlas::{
    Compilation, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
    RegisterProviderError,
};
use vize_carton::{CompactString, cstr};
use vize_croquis::{
    CroquisSemanticProduct, CroquisSemanticSnapshot, SemanticReactivityLossSnapshot,
};

use crate::LintDiagnostic;

const NO_UNUSED_VARS: &str = "vue/no-unused-vars";
const NO_REACTIVITY_LOSS: &str = "type/no-reactivity-loss";

/// Owned diagnostics derived solely from shared semantic facts.
#[derive(Debug, Clone, Default)]
pub struct PatinaSemanticReport {
    /// Deterministically ordered lint diagnostics.
    pub diagnostics: Vec<LintDiagnostic>,
    /// Number of template-scope bindings inspected by this recipe.
    pub checked_template_bindings: usize,
    /// Number of reactivity-loss facts inspected by this recipe.
    pub checked_reactivity_losses: usize,
}

impl PatinaSemanticReport {
    /// Whether this report contains any warning or error.
    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}

/// Atlas identity for Patina diagnostics produced from shared semantics.
pub struct PatinaSemanticReportProduct;

impl Product for PatinaSemanticReportProduct {
    type Value = PatinaSemanticReport;

    const NAME: &'static str = "patina.semantic-diagnostics";
}

/// Provider for the parser-free Patina semantic recipe.
#[derive(Debug, Clone, Copy, Default)]
pub struct PatinaSemanticReportProvider;

impl Provider for PatinaSemanticReportProvider {
    type Product = PatinaSemanticReportProduct;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<CroquisSemanticProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<PatinaSemanticReport, ProviderError> {
        let semantics = context.get::<CroquisSemanticProduct>()?;
        Ok(lint_semantic_snapshot(&semantics))
    }
}

/// Registration handle for Patina's graph-native semantic path.
#[derive(Debug, Clone, Copy, Default)]
pub struct PatinaSemanticRecipe;

impl PatinaSemanticRecipe {
    /// Register the output provider in an Atlas compilation.
    pub fn register(self, compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
        compilation.register_provider(PatinaSemanticReportProvider)
    }

    /// Root product requested by this recipe.
    pub fn product(self) -> ProductId {
        ProductId::of::<PatinaSemanticReportProduct>()
    }
}

/// Register Patina's graph-native semantic lint recipe.
pub fn register_semantic_lint_recipe(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    PatinaSemanticRecipe.register(compilation)
}

/// Run Patina's semantic-only rules without parsing or retaining syntax nodes.
pub fn lint_semantic_snapshot(semantics: &CroquisSemanticSnapshot) -> PatinaSemanticReport {
    let mut report = PatinaSemanticReport::default();

    for scope in &semantics.scopes {
        if !matches!(scope.kind, "v-for" | "v-slot") {
            continue;
        }
        for binding in &scope.bindings {
            report.checked_template_bindings += 1;
            if binding.used || binding.name.starts_with('_') {
                continue;
            }

            let start = binding.declaration_offset;
            let end = start.saturating_add(binding.name.len() as u32);
            let message = cstr!(
                "Variable '{}' is defined by {} but never used",
                binding.name,
                scope.kind
            );
            report.diagnostics.push(
                LintDiagnostic::warn(NO_UNUSED_VARS, message, start, end).with_help(
                    "Remove the binding or prefix it with an underscore when it is intentional.",
                ),
            );
        }
    }

    report.checked_reactivity_losses = semantics.reactivity_losses.len();
    report.diagnostics.extend(
        semantics
            .reactivity_losses
            .iter()
            .map(reactivity_loss_diagnostic),
    );
    report.diagnostics.sort_by(|left, right| {
        (left.start, left.end, left.rule_name, left.message.as_str()).cmp(&(
            right.start,
            right.end,
            right.rule_name,
            right.message.as_str(),
        ))
    });
    report
}

fn reactivity_loss_diagnostic(loss: &SemanticReactivityLossSnapshot) -> LintDiagnostic {
    let message = reactivity_loss_message(loss);
    LintDiagnostic::warn(
        NO_REACTIVITY_LOSS,
        message,
        loss.range.start,
        loss.range.end.max(loss.range.start.saturating_add(1)),
    )
    .with_help("Keep the reactive source intact, or derive values with toRef, toRefs, or computed.")
}

fn reactivity_loss_message(loss: &SemanticReactivityLossSnapshot) -> CompactString {
    let source = loss.source_name.as_deref().unwrap_or("reactive value");
    match loss.kind {
        "reactiveDestructure" | "refValueDestructure" | "propsDestructure" => {
            let names = loss
                .extracted_names
                .iter()
                .map(CompactString::as_str)
                .collect::<Vec<_>>()
                .join(", ");
            cstr!(
                "Destructuring '{}' creates plain snapshots for: {}",
                source,
                names
            )
        }
        "refValueExtract" | "reactivePropertyExtract" | "getterCallExtract" | "plainValueAlias" => {
            let target = loss.target_name.as_deref().unwrap_or("a local binding");
            cstr!(
                "Assigning from '{}' to '{}' stores a plain snapshot",
                source,
                target
            )
        }
        "functionArgumentExtract" => {
            let target = loss.target_name.as_deref().unwrap_or("an argument");
            cstr!(
                "Passing '{}' as '{}' cuts its reactive graph",
                source,
                target
            )
        }
        "reactiveSpread" => cstr!("Spreading '{}' creates a non-reactive copy", source),
        "reactiveReassign" => cstr!("Reassigning '{}' breaks tracked identity", source),
        kind => cstr!("Operation '{}' loses reactivity from '{}'", kind, source),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_armature::parse;
    use vize_atlas::{ProductStatus, SourceId};
    use vize_carton::Bump;
    use vize_croquis::{Drawer, DrawerOptions};

    fn semantic_snapshot(template: &str) -> CroquisSemanticSnapshot {
        let allocator = Bump::new();
        let (root, errors) = parse(&allocator, template);
        assert!(errors.is_empty());
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_template(&root);
        drawer.finish().semantic_snapshot()
    }

    fn script_semantic_snapshot(script: &str) -> CroquisSemanticSnapshot {
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_script_setup(script);
        drawer.finish().semantic_snapshot()
    }

    #[test]
    fn semantic_recipe_reuses_unused_variable_facts_without_syntax() {
        let snapshot = semantic_snapshot(
            r#"<li v-for="(item, index) in items" :key="item.id">{{ item }}</li>"#,
        );
        let report = lint_semantic_snapshot(&snapshot);

        assert_eq!(report.checked_template_bindings, 2);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].rule_name, "vue/no-unused-vars");
        assert!(report.diagnostics[0].message.contains("index"));
    }

    #[test]
    fn semantic_recipe_reuses_reactivity_loss_facts_without_script_ast() {
        let snapshot = script_semantic_snapshot(
            "import { reactive } from 'vue';\nconst state = reactive({ count: 0 });\nconst { count } = state;",
        );
        let report = lint_semantic_snapshot(&snapshot);

        assert_eq!(report.checked_reactivity_losses, 1);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].rule_name, "type/no-reactivity-loss");
        assert!(report.diagnostics[0].message.contains("state"));
    }

    struct SnapshotProvider(CroquisSemanticSnapshot);

    impl Provider for SnapshotProvider {
        type Product = CroquisSemanticProduct;

        fn provide(
            &self,
            _context: &mut ProviderContext<'_>,
        ) -> Result<CroquisSemanticSnapshot, ProviderError> {
            Ok(self.0.clone())
        }
    }

    fn register_source(compilation: &mut Compilation) -> SourceId {
        compilation
            .add_source("invalid.vue", "<<< deliberately not parsed >>>")
            .unwrap()
    }

    #[test]
    fn atlas_provider_only_demands_the_shared_semantic_product() {
        let mut compilation = Compilation::new();
        compilation
            .register_provider(SnapshotProvider(semantic_snapshot(
                r#"<div v-for="unused in items"></div>"#,
            )))
            .unwrap();
        register_semantic_lint_recipe(&mut compilation).unwrap();
        let source = register_source(&mut compilation);

        let outcome = compilation
            .query::<PatinaSemanticReportProduct>(source)
            .unwrap();

        assert_eq!(outcome.status(), ProductStatus::Executed);
        assert_eq!(outcome.value().diagnostics.len(), 1);
        let products: Vec<_> = outcome
            .plan()
            .products()
            .iter()
            .map(|product| product.name())
            .collect();
        assert_eq!(
            products,
            ["croquis.semantics", "patina.semantic-diagnostics"]
        );
    }
}
