//! Cached SFC container and template-source products.

use vize_atlas::{
    ObservationKind, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
    SourceId, SourceInputId, SourceRange, SourceRevision,
};
use vize_carton::{String, cstr};

use crate::{SfcDescriptor, SfcError, parse_sfc};

use super::{SfcParseSettingsInput, is_sfc_context};

/// Cached SFC container parse, including a structured fatal diagnostic.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SfcDescriptorArtifact {
    result: Result<SfcDescriptor<'static>, SfcError>,
}

impl SfcDescriptorArtifact {
    fn new(result: Result<SfcDescriptor<'static>, SfcError>) -> Self {
        Self { result }
    }

    /// Return the parsed descriptor when the SFC container is valid.
    pub fn descriptor(&self) -> Option<&SfcDescriptor<'static>> {
        self.result.as_ref().ok()
    }

    /// Return the cached fatal SFC container diagnostic, if any.
    pub fn diagnostic(&self) -> Option<&SfcError> {
        self.result.as_ref().err()
    }

    /// Borrow the complete parse result without discarding either state.
    pub fn as_result(&self) -> Result<&SfcDescriptor<'static>, &SfcError> {
        self.result.as_ref()
    }

    /// Consume the artifact and recover the complete parse result.
    pub fn into_result(self) -> Result<SfcDescriptor<'static>, SfcError> {
        self.result
    }
}

/// Parsed, owned SFC container descriptor or its fatal parse diagnostic.
pub struct SfcDescriptorProduct;

impl Product for SfcDescriptorProduct {
    type Value = SfcDescriptorArtifact;

    const NAME: &'static str = "sfc.descriptor";
}

/// Owned template block plus exact parent-source provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfcTemplateSource {
    pub parent: SourceId,
    pub parent_revision: SourceRevision,
    pub range: SourceRange,
    pub name: String,
    pub text: String,
}

/// SFC template block selected from a container descriptor.
pub struct SfcTemplateProduct;

impl Product for SfcTemplateProduct {
    type Value = Option<SfcTemplateSource>;

    const NAME: &'static str = "sfc.template-source";
}

/// Parse an applicable `.vue` source without constructing downstream products.
pub struct SfcDescriptorProvider;

impl Provider for SfcDescriptorProvider {
    type Product = SfcDescriptorProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![
            SourceInputId::of::<SfcParseSettingsInput>(),
            SourceInputId::of::<vize_atlas::SourceKindInput>(),
        ]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_context(context)
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<SfcDescriptorArtifact, ProviderError> {
        let options = context
            .source_input::<SfcParseSettingsInput>()
            .cloned()
            .unwrap_or_default();
        let result = parse_sfc(context.source().text(), options).map(SfcDescriptor::into_owned);
        if let Err(error) = &result {
            context.observe(
                ObservationKind::Diagnostic,
                "sfc.parse.error",
                error.message.as_str(),
                error
                    .loc
                    .as_ref()
                    .map(|loc| SourceRange::new(loc.start, loc.end)),
            );
        }
        Ok(SfcDescriptorArtifact::new(result))
    }
}

/// Decompose the template block while retaining its parent identity/range.
pub struct SfcTemplateProvider;

impl Provider for SfcTemplateProvider {
    type Product = SfcTemplateProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<vize_atlas::SourceKindInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_context(context)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<SfcDescriptorProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<Option<SfcTemplateSource>, ProviderError> {
        let artifact = context.get::<SfcDescriptorProduct>()?;
        let Some(descriptor) = artifact.descriptor() else {
            return Ok(None);
        };
        let Some(template) = descriptor.template.as_ref() else {
            return Ok(None);
        };
        let source = context.source();
        Ok(Some(SfcTemplateSource {
            parent: source.id(),
            parent_revision: source.revision(),
            range: SourceRange::new(template.loc.start, template.loc.end),
            name: cstr!("{}#template", source.name()),
            text: template.content.as_ref().into(),
        }))
    }
}
