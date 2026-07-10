//! Revision-keyed memoized product storage.

use vize_carton::{FxHashMap, FxHashSet};

use crate::{
    InputId, InvalidatedProduct, Product, ProductId, ProductRequest, ProviderId,
    ProviderObservation, Shared, SourceId, SourceRevision, provider::ErasedValue,
};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
struct CacheKey {
    source: SourceId,
    product: ProductId,
}

#[derive(Clone)]
struct CacheEntry {
    revision: SourceRevision,
    provider: ProviderId,
    inputs: Vec<InputId>,
    source_dependencies: Vec<(SourceId, SourceRevision)>,
    value: ErasedValue,
    observation_closure: Vec<ProviderObservation>,
}

pub(crate) struct CachedArtifact {
    pub(crate) value: ErasedValue,
    pub(crate) observation_closure: Vec<ProviderObservation>,
}

/// Observable identity of one cached typed artifact.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CachedProduct {
    pub source: SourceId,
    pub revision: SourceRevision,
    pub product: ProductId,
    /// Concrete provider whose value is cached.
    pub provider: ProviderId,
}

impl CachedProduct {
    /// Complete request identity for this cached artifact.
    pub const fn request(self) -> ProductRequest {
        ProductRequest::new(self.source, self.product)
    }
}

/// Per-compilation memoization store keyed by source and open product identity.
#[derive(Clone, Default)]
pub struct ArtifactCache {
    entries: FxHashMap<CacheKey, CacheEntry>,
}

impl ArtifactCache {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether any revision of typed product `P` is cached for `source`.
    pub fn contains<P: Product>(&self, source: SourceId) -> bool {
        self.entries.contains_key(&CacheKey {
            source,
            product: ProductId::of::<P>(),
        })
    }

    /// Inspect cached identities without exposing erased values.
    pub fn products(&self) -> impl Iterator<Item = CachedProduct> + '_ {
        self.entries.iter().map(|(key, entry)| CachedProduct {
            source: key.source,
            revision: entry.revision,
            product: key.product,
            provider: entry.provider,
        })
    }

    pub(crate) fn get(
        &self,
        request: ProductRequest,
        revision: SourceRevision,
        provider: ProviderId,
        source_dependencies: &[(SourceId, SourceRevision)],
    ) -> Option<CachedArtifact> {
        self.entries
            .get(&CacheKey {
                source: request.source(),
                product: request.product(),
            })
            .filter(|entry| {
                entry.revision == revision
                    && entry.provider == provider
                    && entry.source_dependencies == source_dependencies
            })
            .map(|entry| CachedArtifact {
                value: Shared::clone(&entry.value),
                observation_closure: entry.observation_closure.clone(),
            })
    }

    pub(crate) fn insert(
        &mut self,
        request: ProductRequest,
        revision: SourceRevision,
        provider: ProviderId,
        inputs: &[InputId],
        source_dependencies: &[(SourceId, SourceRevision)],
        artifact: CachedArtifact,
    ) {
        self.entries.insert(
            CacheKey {
                source: request.source(),
                product: request.product(),
            },
            CacheEntry {
                revision,
                provider,
                inputs: inputs.to_vec(),
                source_dependencies: source_dependencies.to_vec(),
                value: artifact.value,
                observation_closure: artifact.observation_closure,
            },
        );
    }

    pub(crate) fn evict_sources(
        &mut self,
        affected: &FxHashSet<SourceId>,
    ) -> Vec<InvalidatedProduct> {
        let mut keys: Vec<_> = self
            .entries
            .iter()
            .filter_map(|(key, entry)| {
                (affected.contains(&key.source)
                    || entry
                        .source_dependencies
                        .iter()
                        .any(|(source, _)| affected.contains(source)))
                .then_some(*key)
            })
            .collect();
        sort_keys(&mut keys);
        keys.into_iter()
            .filter_map(|key| {
                let entry = self.entries.remove(&key)?;
                Some(InvalidatedProduct {
                    source: key.source,
                    product: key.product,
                    provider: entry.provider,
                })
            })
            .collect()
    }

    pub(crate) fn evict_input(&mut self, input: InputId) -> Vec<InvalidatedProduct> {
        let mut keys: Vec<_> = self
            .entries
            .iter()
            .filter_map(|(key, entry)| entry.inputs.contains(&input).then_some(*key))
            .collect();
        sort_keys(&mut keys);
        keys.into_iter()
            .filter_map(|key| {
                let entry = self.entries.remove(&key)?;
                Some(InvalidatedProduct {
                    source: key.source,
                    product: key.product,
                    provider: entry.provider,
                })
            })
            .collect()
    }
}

fn sort_keys(keys: &mut [CacheKey]) {
    keys.sort_unstable_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then_with(|| left.product.name().cmp(right.product.name()))
    });
}
