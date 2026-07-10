//! Atlas identity for the owned, frontend-neutral Croquis semantic snapshot.

use vize_atlas::Product;

use crate::CroquisSemanticSnapshot;

/// Demandable semantic facts shared by tools and backends.
///
/// Relief- and OXC-specific producers live in their respective frontend
/// crates; the cached value has no parser lifetime or syntax-node references.
pub struct CroquisSemanticProduct;

impl Product for CroquisSemanticProduct {
    type Value = CroquisSemanticSnapshot;

    const NAME: &'static str = "croquis.semantics";
}
