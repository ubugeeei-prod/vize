//! Atlas identities for owned, frontend-neutral Croquis products.

use vize_atlas::Product;

#[cfg(feature = "analysis")]
use vize_atlas::{
    Compilation, PlanningContext, ProductId, Provider, ProviderContext, ProviderError,
    RegisterProviderError,
};

use crate::CroquisSemanticSnapshot;

#[cfg(feature = "analysis")]
use crate::CroquisDocument;

/// Complete semantic document produced once by a frontend.
#[cfg(feature = "analysis")]
pub struct CroquisDocumentProduct;

#[cfg(feature = "analysis")]
impl Product for CroquisDocumentProduct {
    type Value = CroquisDocument;

    const NAME: &'static str = "croquis.document";
}

/// Demandable semantic facts shared by tools and backends.
///
/// Relief- and OXC-specific producers live in their respective frontend
/// crates; the cached value has no parser lifetime or syntax-node references.
pub struct CroquisSemanticProduct;

impl Product for CroquisSemanticProduct {
    type Value = CroquisSemanticSnapshot;

    const NAME: &'static str = "croquis.semantics";
}

/// Project a complete document into the compact snapshot contract without
/// rerunning frontend analysis.
#[cfg(feature = "analysis")]
pub struct CroquisSemanticProjectionProvider;

#[cfg(feature = "analysis")]
impl Provider for CroquisSemanticProjectionProvider {
    type Product = CroquisSemanticProduct;

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<CroquisDocumentProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<CroquisSemanticSnapshot, ProviderError> {
        let document = context.get::<CroquisDocumentProduct>()?;
        Ok(document.semantic_snapshot())
    }
}

#[cfg(feature = "analysis")]
pub fn register_semantic_projection(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(CroquisSemanticProjectionProvider)
}
