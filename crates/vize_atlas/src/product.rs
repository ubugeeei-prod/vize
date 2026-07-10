//! Open, typed product identities.

use std::{any::TypeId, fmt};

use crate::SourceId;

/// Whether a completed product survives beyond its current execution outcome.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum CachePolicy {
    /// Retain the product by source/revision for reuse by later queries.
    #[default]
    Memoized,
    /// Share the product only inside the current multi-root execution.
    Transient,
}

/// A typed artifact that can be requested from a [`Compilation`](crate::Compilation).
///
/// Product marker types are intentionally open: syntax trees, semantic facts,
/// control-flow graphs, diagnostics, generated code, and future artifacts all
/// implement this trait in their owning crate. Atlas contains no domain enum
/// that must be extended when a product is added.
pub trait Product: Send + Sync + 'static {
    /// The owned, `'static` storage retained by the artifact cache.
    ///
    /// This need not be the representation consumers use directly. Products
    /// with borrowed, arena-backed, interned, or streamed interfaces can also
    /// implement [`ProductView`] without changing this storage contract.
    type Value: Send + Sync + 'static;

    /// A stable diagnostic name for plans, counters, and traces.
    const NAME: &'static str;

    /// Persistence policy for the product's owned storage.
    ///
    /// Transient values still participate in typed dependency sharing inside
    /// one plan, but create no persistent artifact-cache entry.
    const CACHE_POLICY: CachePolicy = CachePolicy::Memoized;
}

/// Optional consumer projection over a product's cached storage.
///
/// [`Product::Value`] remains owned and `'static` so Atlas can memoize and
/// share it across queries. This separate opt-in contract lets a product expose
/// a view whose lifetime is tied to that storage, including references,
/// arena/intern-table facades, or iterator-like streams. Existing products do
/// not need to implement this trait.
pub trait ProductView: Product {
    /// Consumer-facing projection borrowing cached storage for `'storage`.
    type View<'storage>
    where
        Self: 'storage;

    /// Project cached storage into its consumer-facing view.
    fn view<'storage>(storage: &'storage Self::Value) -> Self::View<'storage>;
}

/// Runtime identity for an open [`Product`] marker.
///
/// The private [`TypeId`] supplies collision-free process-local identity; the
/// stable name is retained for diagnostics and serialized observations.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ProductId {
    type_id: TypeId,
    name: &'static str,
}

impl ProductId {
    /// Return the identity of product marker `P`.
    pub fn of<P: Product>() -> Self {
        Self {
            type_id: TypeId::of::<P>(),
            name: P::NAME,
        }
    }

    /// Return the stable diagnostic name of this product.
    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl fmt::Debug for ProductId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ProductId")
            .field(&self.name)
            .finish()
    }
}

impl fmt::Display for ProductId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name)
    }
}

/// Identity of one product requested for one source.
///
/// Unlike [`ProductId`], this is a complete artifact-graph node identity. It
/// lets a provider for one source depend on products owned by other sources
/// without introducing a project- or language-specific abstraction in Atlas.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct ProductRequest {
    source: SourceId,
    product: ProductId,
}

impl ProductRequest {
    /// Construct a request from its source and runtime product identity.
    pub const fn new(source: SourceId, product: ProductId) -> Self {
        Self { source, product }
    }

    /// Construct a request for typed product `P`.
    pub fn for_product<P: Product>(source: SourceId) -> Self {
        Self::new(source, ProductId::of::<P>())
    }

    /// Source for which the product is requested.
    pub const fn source(self) -> SourceId {
        self.source
    }

    /// Runtime identity of the requested product.
    pub const fn product(self) -> ProductId {
        self.product
    }
}

impl fmt::Display for ProductRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.source, self.product)
    }
}
