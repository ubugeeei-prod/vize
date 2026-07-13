//! Atlas root for editor virtual documents.

use vize_atelier_sfc::SfcDescriptorProduct;
use vize_atlas::{
    Compilation, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
    RegisterProviderError,
};
use vize_croquis::CroquisDocumentProduct;
use vize_relief::ReliefProduct;

use super::{VirtualCodeGenerator, VirtualDocuments};

/// Complete standard-SFC virtual document set.
pub(crate) struct VirtualDocumentsProduct;

impl Product for VirtualDocumentsProduct {
    type Value = VirtualDocuments;

    const NAME: &'static str = "maestro.virtual-documents";
}

struct VirtualDocumentsProvider;

impl Provider for VirtualDocumentsProvider {
    type Product = VirtualDocumentsProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        let name = context.source().name();
        name.ends_with(".vue") && !name.ends_with(".art.vue")
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<SfcDescriptorProduct>(),
            ProductId::of::<ReliefProduct>(),
            ProductId::of::<CroquisDocumentProduct>(),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<VirtualDocuments, ProviderError> {
        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let syntax = context.get::<ReliefProduct>()?;
        let croquis = context.get::<CroquisDocumentProduct>()?;
        let Some(descriptor) = descriptor.descriptor() else {
            return Ok(VirtualDocuments::new());
        };
        match (descriptor.template.as_ref(), syntax.as_ref()) {
            (Some(_), None) | (None, Some(_)) => {
                return Err(ProviderError::message(
                    "SFC descriptor and Relief syntax disagree about template presence",
                ));
            }
            _ => {}
        }
        let mut generator = VirtualCodeGenerator::new();
        Ok(generator.generate_from_snapshot(
            descriptor,
            context.source().name(),
            syntax.as_ref().as_ref().map(|syntax| syntax.snapshot()),
            croquis.analysis(),
        ))
    }
}

pub(crate) fn register_virtual_documents_provider(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(VirtualDocumentsProvider)
}
