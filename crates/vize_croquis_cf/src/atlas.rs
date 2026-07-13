//! Atlas providers for demand-driven cross-file semantic analysis.

#[path = "atlas/analysis.rs"]
mod analysis;

pub use analysis::{
    CrossFileAnalysisArtifact, CrossFileAnalysisInput, CrossFileAnalysisProduct,
    CrossFileAnalysisProvider, CrossFileAnalysisRequest, CrossFileOffsetRegion,
    CrossFileSourceLayout,
};

use std::{collections::BTreeMap, path::Path};

use vize_atlas::{
    Compilation, PlanningContext, ProductRequest, Provider, ProviderContext, ProviderError,
    RegisterProviderError, SourceId,
};
use vize_carton::CompactString;
use vize_croquis::{CroquisSemanticProduct, CroquisSemanticSnapshot};

use crate::{
    CroquisProjectComponentUsage, CroquisProjectInjectionGroup, CroquisProjectProduct,
    CroquisProjectSnapshot, CroquisProjectSource,
};

/// Cross-source provider selected only when a project product is requested.
#[derive(Debug, Clone, Copy, Default)]
pub struct CroquisProjectProvider;

impl Provider for CroquisProjectProvider {
    type Product = CroquisProjectProduct;

    fn dependency_requests(&self, context: &PlanningContext<'_>) -> Vec<ProductRequest> {
        semantic_source_ids(context)
            .into_iter()
            .map(ProductRequest::for_product::<CroquisSemanticProduct>)
            .collect()
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<CroquisProjectSnapshot, ProviderError> {
        let sources: Vec<_> = context
            .sources()
            .iter()
            .filter(|source| is_semantic_source(source.name()))
            .cloned()
            .collect();
        let mut semantic_sources = Vec::new();
        for source in sources {
            let semantics = context.get_for_source::<CroquisSemanticProduct>(source.id())?;
            semantic_sources.push((source, semantics));
        }
        semantic_sources.sort_by_key(|(source, _)| source.id());
        Ok(build_project_snapshot(
            context.source().id(),
            &semantic_sources,
        ))
    }
}

/// Register opt-in project aggregation without allocating per-source state.
pub fn register_atlas_provider(compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
    compilation.register_provider(CroquisProjectProvider)?;
    compilation.register_provider(CrossFileAnalysisProvider)
}

fn semantic_source_ids(context: &PlanningContext<'_>) -> Vec<SourceId> {
    let mut sources: Vec<_> = context
        .sources()
        .iter()
        .filter(|source| is_semantic_source(source.name()))
        .map(|source| source.id())
        .collect();
    sources.sort_unstable();
    sources
}

fn is_semantic_source(name: &str) -> bool {
    matches!(
        Path::new(name)
            .extension()
            .and_then(|extension| extension.to_str()),
        Some("vue" | "jsx" | "tsx")
    )
}

fn build_project_snapshot(
    anchor: SourceId,
    semantic_sources: &[(
        vize_atlas::SourceSnapshot,
        vize_atlas::Shared<CroquisSemanticSnapshot>,
    )],
) -> CroquisProjectSnapshot {
    let component_index = component_index(semantic_sources);
    let mut snapshot = CroquisProjectSnapshot {
        anchor: Some(anchor),
        sources: semantic_sources
            .iter()
            .map(|(source, semantics)| CroquisProjectSource {
                source: source.id(),
                revision: source.revision(),
                name: CompactString::new(source.name()),
                binding_count: semantics.bindings.len(),
                scope_count: semantics.scopes.len(),
                template_expression_count: semantics.template_expressions.len(),
                component_usage_count: semantics.component_usages.len(),
            })
            .collect(),
        component_usages: Vec::new(),
        injection_groups: injection_groups(semantic_sources),
    };

    for (source, semantics) in semantic_sources {
        for usage in &semantics.component_usages {
            let candidates = component_index
                .get(&normalize_name(&usage.name))
                .cloned()
                .unwrap_or_default();
            snapshot
                .component_usages
                .push(CroquisProjectComponentUsage {
                    source: source.id(),
                    name: usage.name.clone(),
                    range: usage.range,
                    candidates,
                });
        }
    }
    snapshot.component_usages.sort_by(|left, right| {
        (left.source, left.range.start, left.name.as_str()).cmp(&(
            right.source,
            right.range.start,
            right.name.as_str(),
        ))
    });
    snapshot
}

fn component_index(
    semantic_sources: &[(
        vize_atlas::SourceSnapshot,
        vize_atlas::Shared<CroquisSemanticSnapshot>,
    )],
) -> BTreeMap<CompactString, Vec<SourceId>> {
    let mut index = BTreeMap::<CompactString, Vec<SourceId>>::new();
    for (source, _) in semantic_sources {
        let stem = Path::new(source.name())
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(source.name());
        index
            .entry(normalize_name(stem))
            .or_default()
            .push(source.id());
    }
    index
}

fn injection_groups(
    semantic_sources: &[(
        vize_atlas::SourceSnapshot,
        vize_atlas::Shared<CroquisSemanticSnapshot>,
    )],
) -> Vec<CroquisProjectInjectionGroup> {
    let mut groups = BTreeMap::<CompactString, (Vec<SourceId>, Vec<SourceId>)>::new();
    for (source, semantics) in semantic_sources {
        for provide in &semantics.provides {
            groups
                .entry(provide.key.clone())
                .or_default()
                .0
                .push(source.id());
        }
        for inject in &semantics.injects {
            groups
                .entry(inject.key.clone())
                .or_default()
                .1
                .push(source.id());
        }
    }
    groups
        .into_iter()
        .map(|(key, (mut providers, mut consumers))| {
            providers.sort_unstable();
            providers.dedup();
            consumers.sort_unstable();
            consumers.dedup();
            CroquisProjectInjectionGroup {
                key,
                providers,
                consumers,
            }
        })
        .collect()
}

fn normalize_name(name: &str) -> CompactString {
    let mut normalized = CompactString::new("");
    for character in name.chars().filter(char::is_ascii_alphanumeric) {
        normalized.push(character.to_ascii_lowercase());
    }
    normalized
}
