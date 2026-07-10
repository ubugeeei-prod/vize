//! Immutable requested dependency closures.

use vize_carton::FxHashMap;

use crate::{InputId, Product, ProductId, ProductRequest, ProviderId, SourceId, SourceRevision};

/// Topologically ordered dependency closure for one or more requested roots.
///
/// Planning performs no provider work. [`Plan::requests`] contains only the
/// roots and their transitive dependencies, with every dependency before its
/// users. The original product-only accessors remain convenient for plans
/// rooted in one source; request-aware accessors retain identity in project
/// plans spanning multiple sources.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Plan {
    pub(crate) source: SourceId,
    pub(crate) source_revision: SourceRevision,
    pub(crate) source_revisions: Vec<(SourceId, SourceRevision)>,
    pub(crate) provider_generation: u64,
    pub(crate) input_revisions: Vec<(InputId, u64)>,
    pub(crate) roots: Vec<ProductId>,
    pub(crate) products: Vec<ProductId>,
    pub(crate) root_requests: Vec<ProductRequest>,
    pub(crate) requests: Vec<ProductRequest>,
    pub(crate) product_dependencies: FxHashMap<ProductId, Vec<ProductId>>,
    pub(crate) dependencies: FxHashMap<ProductRequest, Vec<ProductRequest>>,
    pub(crate) providers: FxHashMap<ProductRequest, ProviderId>,
    pub(crate) input_dependencies: FxHashMap<ProductRequest, Vec<InputId>>,
    pub(crate) source_dependencies: FxHashMap<ProductRequest, Vec<(SourceId, SourceRevision)>>,
}

impl Plan {
    /// Source of the first root, retained for single-source compatibility.
    pub const fn source(&self) -> SourceId {
        self.source
    }

    /// Revision of the first root's source.
    pub const fn source_revision(&self) -> SourceRevision {
        self.source_revision
    }

    /// Every source revision captured while planning.
    pub fn source_revisions(&self) -> &[(SourceId, SourceRevision)] {
        &self.source_revisions
    }

    /// Captured revision for one participating source.
    pub fn source_revision_for(&self, source: SourceId) -> Option<SourceRevision> {
        self.source_revisions
            .iter()
            .find_map(|(candidate, revision)| (*candidate == source).then_some(*revision))
    }

    /// Requested product identities, in first-request order.
    ///
    /// For multi-source plans use [`Plan::root_requests`] to retain source
    /// identity.
    pub fn roots(&self) -> &[ProductId] {
        &self.roots
    }

    /// Requested roots with complete source and product identity.
    pub fn root_requests(&self) -> &[ProductRequest] {
        &self.root_requests
    }

    /// Topologically ordered products. Duplicate product identities may appear
    /// when the same product is requested for multiple sources.
    pub fn products(&self) -> &[ProductId] {
        &self.products
    }

    /// Topologically ordered request closure with complete node identity.
    pub fn requests(&self) -> &[ProductRequest] {
        &self.requests
    }

    /// Direct same-source dependencies selected for `product`.
    pub fn dependencies(&self, product: ProductId) -> Option<&[ProductId]> {
        self.product_dependencies.get(&product).map(Vec::as_slice)
    }

    /// Direct dependencies selected for one complete request identity.
    pub fn dependencies_for_request(&self, request: ProductRequest) -> Option<&[ProductRequest]> {
        self.dependencies.get(&request).map(Vec::as_slice)
    }

    /// Concrete provider selected for a product on the first root source.
    pub fn provider(&self, product: ProductId) -> Option<ProviderId> {
        self.provider_for_request(ProductRequest::new(self.source, product))
    }

    /// Concrete provider selected for one complete request.
    pub fn provider_for_request(&self, request: ProductRequest) -> Option<ProviderId> {
        self.providers.get(&request).copied()
    }

    /// Concrete provider selected for typed product `P` on the first source.
    pub fn provider_for<P: Product>(&self) -> Option<ProviderId> {
        self.provider(ProductId::of::<P>())
    }

    /// Inputs that can affect this product or any transitive dependency.
    pub fn input_dependencies(&self, product: ProductId) -> Option<&[InputId]> {
        self.input_dependencies_for_request(ProductRequest::new(self.source, product))
    }

    /// Inputs that can affect a complete request or any dependency.
    pub fn input_dependencies_for_request(&self, request: ProductRequest) -> Option<&[InputId]> {
        self.input_dependencies.get(&request).map(Vec::as_slice)
    }

    /// Sources whose revisions contribute transitively to one request.
    pub fn source_dependencies_for_request(
        &self,
        request: ProductRequest,
    ) -> Option<&[(SourceId, SourceRevision)]> {
        self.source_dependencies.get(&request).map(Vec::as_slice)
    }

    /// Inputs relevant to the complete plan and the revisions it captured.
    pub fn input_revisions(&self) -> &[(InputId, u64)] {
        &self.input_revisions
    }

    /// Whether the plan contains typed product `P` for any source.
    pub fn contains<P: Product>(&self) -> bool {
        self.products.contains(&ProductId::of::<P>())
    }

    /// Whether the plan contains typed product `P` for `source`.
    pub fn contains_for_source<P: Product>(&self, source: SourceId) -> bool {
        self.requests
            .contains(&ProductRequest::for_product::<P>(source))
    }
}
