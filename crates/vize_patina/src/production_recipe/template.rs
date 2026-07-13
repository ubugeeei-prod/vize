//! Production Patina recipe for independently supplied template documents.

use vize_atlas::{
    Compilation, PlanningContext, ProductId, Provider, ProviderContext, ProviderError,
    RegisterProviderError, Shared, SourceId, SourceInput, SourceInputId,
};
use vize_carton::CompactString;
use vize_croquis::CroquisDocumentProduct;
use vize_relief::ReliefProduct;

use super::{PatinaDocumentReportProduct, PatinaLinterInput};
use crate::{LintResult, Linter};

/// Interpretation of an independently supplied template document.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PatinaTemplateDocumentKind {
    /// Vue template content, with parser diagnostics included.
    VueTemplate,
    /// A complete HTML document, including inline script linting.
    StandaloneHtml,
}

/// Per-source identity and behavior for a template lint root.
#[derive(Debug, Clone)]
pub struct PatinaTemplateLintRequest {
    filename: CompactString,
    kind: PatinaTemplateDocumentKind,
}

impl PatinaTemplateLintRequest {
    pub fn vue_template(filename: impl Into<CompactString>) -> Self {
        Self {
            filename: filename.into(),
            kind: PatinaTemplateDocumentKind::VueTemplate,
        }
    }

    pub fn standalone_html(filename: impl Into<CompactString>) -> Self {
        Self {
            filename: filename.into(),
            kind: PatinaTemplateDocumentKind::StandaloneHtml,
        }
    }
}

/// Typed source-scoped request making raw-template lint applicability explicit.
pub struct PatinaTemplateLintInput;

impl SourceInput for PatinaTemplateLintInput {
    type Value = PatinaTemplateLintRequest;

    const NAME: &'static str = "patina.template-lint-request";
}

/// Patina report provider consuming raw-template Relief and Croquis products.
pub struct PatinaTemplateLintProvider {
    linter: Shared<Linter>,
}

impl PatinaTemplateLintProvider {
    pub fn from_shared(linter: Shared<Linter>) -> Self {
        Self { linter }
    }
}

impl Provider for PatinaTemplateLintProvider {
    type Product = PatinaDocumentReportProduct;

    fn input_dependencies(&self) -> Vec<vize_atlas::InputId> {
        vec![vize_atlas::InputId::of::<PatinaLinterInput>()]
    }

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<PatinaTemplateLintInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        context.source_input::<PatinaTemplateLintInput>().is_some()
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<ReliefProduct>(),
            ProductId::of::<CroquisDocumentProduct>(),
        ]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<LintResult, ProviderError> {
        let request = context
            .source_input::<PatinaTemplateLintInput>()
            .cloned()
            .ok_or_else(|| ProviderError::message("raw-template lint request is absent"))?;
        let syntax = context.get::<ReliefProduct>()?;
        let syntax = syntax
            .as_ref()
            .as_ref()
            .ok_or_else(|| ProviderError::message("raw-template Relief is absent"))?;
        let semantics = context.get::<CroquisDocumentProduct>()?;
        let linter = context
            .input::<PatinaLinterInput>()
            .map(Shared::clone)
            .unwrap_or_else(|| Shared::clone(&self.linter));
        let result = match request.kind {
            PatinaTemplateDocumentKind::VueTemplate => linter.lint_template_with_shared_products(
                context.source().text(),
                request.filename.as_str(),
                syntax.snapshot(),
                syntax.parse_diagnostics(),
                Some(semantics.analysis()),
            ),
            PatinaTemplateDocumentKind::StandaloneHtml => linter
                .lint_standalone_html_with_shared_products(
                    context.source().text(),
                    request.filename.as_str(),
                    syntax.snapshot(),
                    syntax.parse_diagnostics(),
                    Some(semantics.analysis()),
                ),
        };
        Ok(result)
    }
}

/// Install the explicit lint interpretation for one raw-template source.
pub fn install_template_lint_request(
    compilation: &mut Compilation,
    source: SourceId,
    request: PatinaTemplateLintRequest,
) -> Result<(), vize_atlas::CompilationInputError> {
    compilation
        .set_source_input::<PatinaTemplateLintInput>(source, request)
        .map(|_| ())
}

/// Register the raw-template Patina report root.
pub fn register_shared_template_lint_recipe(
    compilation: &mut Compilation,
    linter: Shared<Linter>,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(PatinaTemplateLintProvider::from_shared(linter))
}

#[cfg(test)]
mod tests {
    use vize_atlas::{Compilation, ProductStatus};
    use vize_carton::cstr;
    use vize_relief::ReliefProduct;

    use super::*;

    fn query(kind: PatinaTemplateDocumentKind) -> (LintResult, vize_atlas::Plan) {
        let text = r#"<button v-for="item in items">{{ item }}</button>"#;
        let filename = if kind == PatinaTemplateDocumentKind::StandaloneHtml {
            "index.html"
        } else {
            "button.html"
        };
        let linter = Shared::new(Linter::new());
        let mut compilation = Compilation::new();
        compilation
            .register_provider(vize_atelier_template::RawTemplateReliefProvider)
            .unwrap();
        compilation
            .register_provider(vize_atelier_template::RawTemplateCroquisProvider)
            .unwrap();
        register_shared_template_lint_recipe(&mut compilation, Shared::clone(&linter)).unwrap();
        let source = compilation
            .add_source("fixture.vue-template", text)
            .unwrap();
        compilation
            .set_source_input::<vize_atelier_template::TemplateCompileSettingsInput>(
                source,
                vize_atelier_template::TemplateCompileRequest::default(),
            )
            .unwrap();
        let request = match kind {
            PatinaTemplateDocumentKind::VueTemplate => {
                PatinaTemplateLintRequest::vue_template(filename)
            }
            PatinaTemplateDocumentKind::StandaloneHtml => {
                PatinaTemplateLintRequest::standalone_html(filename)
            }
        };
        install_template_lint_request(&mut compilation, source, request).unwrap();
        let expected = match kind {
            PatinaTemplateDocumentKind::VueTemplate => linter.lint_template(text, filename),
            PatinaTemplateDocumentKind::StandaloneHtml => {
                linter.lint_standalone_html(text, filename)
            }
        };
        let outcome = compilation
            .snapshot()
            .query_session()
            .query::<PatinaDocumentReportProduct>(source)
            .unwrap();
        assert_eq!(outcome.status(), ProductStatus::Executed);
        assert_eq!(cstr!("{:?}", outcome.value()), cstr!("{expected:?}"));
        (outcome.value().clone(), outcome.plan().clone())
    }

    #[test]
    fn raw_template_report_matches_direct_linter_and_depends_on_relief() {
        let (_, plan) = query(PatinaTemplateDocumentKind::VueTemplate);
        assert!(plan.contains::<ReliefProduct>());
        assert!(plan.contains::<CroquisDocumentProduct>());
    }

    #[test]
    fn standalone_html_report_matches_direct_linter_and_depends_on_relief() {
        let (_, plan) = query(PatinaTemplateDocumentKind::StandaloneHtml);
        assert!(plan.contains::<ReliefProduct>());
        assert!(plan.contains::<CroquisDocumentProduct>());
    }
}
