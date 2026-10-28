//! Revisioned declaration provenance; never infer providers from callable shapes.
use oxc_syntax::symbol::SymbolId;
use vize_carton::{FxHashMap, String};

mod collect;
mod declarations;
mod globals;
mod graph;
mod type_queries;
pub use collect::{collect, collect_requested_globals};
pub use graph::seal;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_ambient;
#[cfg(test)]
mod tests_graph;
#[cfg(test)]
mod tests_requested_globals;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Namespace {
    Value,
    Type,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Usage {
    Runtime,
    TypeQuery,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectRevision {
    pub owner: u64,
    pub generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceRevision {
    pub owner: u64,
    pub source: u32,
    pub version: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocalSymbol {
    pub symbol: SymbolId,
    pub namespace: Namespace,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BindingRef {
    pub source: u32,
    pub local: LocalSymbol,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbolIdentity {
    pub source: SourceRevision,
    pub local: LocalSymbol,
}

/// Only identity-preserving aliases belong here. Callable types, casts, generic
/// instantiations and mutable inferred aliases remain opaque.
#[derive(Clone, Debug)]
pub enum BindingTarget {
    Opaque,
    Alias {
        binding: BindingRef,
        usage: Usage,
    },
    Global {
        name: String,
        namespace: Namespace,
        usage: Usage,
    },
    Import {
        module: String,
        exported: String,
        namespace: Namespace,
        type_only: bool,
        usage: Option<Usage>,
    },
    Candidates(Vec<BindingTarget>),
    Ambiguous,
}
#[derive(Debug)]
pub struct SourceFacts {
    pub revision: SourceRevision,
    pub bindings: FxHashMap<LocalSymbol, BindingTarget>,
    pub globals: Vec<(String, LocalSymbol)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportResolution {
    Binding(SymbolIdentity),
    External(SymbolIdentity),
    Missing,
    Ambiguous,
}
/// This adapter must consume the host's actual source/resolution snapshot.
/// A literal module name or structural signature cannot authorize an identity.
pub trait Resolver {
    fn resolve_import(
        &self,
        from: SourceRevision,
        module: &str,
        exported: &str,
        namespace: Namespace,
        usage: Usage,
    ) -> ImportResolution;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingOrigin {
    Vue,
    Custom(SymbolIdentity),
    Ambiguous,
    Unresolved,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginError {
    ForeignOwner,
    MissingSemanticScope,
    ConflictingSourceRevision,
    DuplicateSourceFacts,
    UnregisteredReferenceProvider,
    StaleProjectRevision,
    InvalidGraphIndex,
    UnregisteredSource,
}

pub struct ProjectFacts<'a> {
    revision: ProjectRevision,
    versions: FxHashMap<u32, SourceRevision>,
    sources: FxHashMap<u32, &'a SourceFacts>,
}
impl<'a> ProjectFacts<'a> {
    pub fn new(revision: ProjectRevision) -> Self {
        Self {
            revision,
            versions: FxHashMap::default(),
            sources: FxHashMap::default(),
        }
    }
    /// A provider may need only identity metadata when its actual symbol was
    /// already resolved by the host. No AST or file read is repeated here.
    pub fn register_revision(&mut self, revision: SourceRevision) -> Result<(), OriginError> {
        if revision.owner != self.revision.owner {
            return Err(OriginError::ForeignOwner);
        }
        if self
            .versions
            .get(&revision.source)
            .is_some_and(|known| *known != revision)
        {
            return Err(OriginError::ConflictingSourceRevision);
        }
        self.versions.insert(revision.source, revision);
        Ok(())
    }
    pub fn add_source(&mut self, source: &'a SourceFacts) -> Result<(), OriginError> {
        self.register_revision(source.revision)?;
        if self.sources.contains_key(&source.revision.source) {
            return Err(OriginError::DuplicateSourceFacts);
        }
        self.sources.insert(source.revision.source, source);
        Ok(())
    }
}

/// Immutable, project-scoped lookup. Existing serialized DTOs and default
/// inference behavior are intentionally unchanged until every host is wired.
#[derive(Debug)]
pub struct OriginSnapshot {
    revision: ProjectRevision,
    reference: SymbolIdentity,
    origins: FxHashMap<String, BindingOrigin>,
}
impl OriginSnapshot {
    pub fn get(&self, name: &str, revision: ProjectRevision) -> Result<BindingOrigin, OriginError> {
        if revision != self.revision {
            return Err(OriginError::StaleProjectRevision);
        }
        Ok(self
            .origins
            .get(name)
            .copied()
            .unwrap_or(BindingOrigin::Unresolved))
    }
    pub fn reference(&self) -> SymbolIdentity {
        self.reference
    }
}

#[cfg(all(test, feature = "native", target_os = "linux"))]
mod native_api_probe;
