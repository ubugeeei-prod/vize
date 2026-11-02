//! Module identities and lexical bindings for externally resolved public types.
//!
//! This store supplements the compatibility resolver's flat name maps. A name
//! resolves only through its declaring module's bindings and actual exports.

mod props;
mod resolve;
pub use props::{ScopedTypeProperties, ScopedTypeProperty};
#[cfg(test)]
mod tests;

#[expect(
    clippy::disallowed_types,
    reason = "worlds share immutable resolved dependency modules by Arc instead of cloning them per component"
)]
use std::sync::Arc;
use vize_carton::{CompactString, FxHashMap, FxHashSet};

#[derive(Debug, Clone, Default)]
pub struct ResolvedTypeWorld {
    pub root_module: CompactString,
    /// The root module is private to this world; dependency modules are the
    /// resolved facts every world built from the same source snapshot shares.
    #[expect(
        clippy::disallowed_types,
        reason = "dependency modules are shared across the worlds of one source snapshot"
    )]
    pub modules: FxHashMap<CompactString, Arc<TypeModule>>,
}

#[derive(Debug, Clone, Default)]
pub struct TypeModule {
    pub declarations: FxHashMap<CompactString, TypeDeclaration>,
    /// Declaration merging needs semantic elaboration; never select one part.
    pub unsupported_declarations: FxHashSet<CompactString>,
    pub unsupported_contracts: FxHashMap<CompactString, Vec<CompactString>>,
    pub imports: FxHashMap<CompactString, TypeImport>,
    pub exports: FxHashMap<CompactString, TypeExportBinding>,
    pub star_exports: Vec<TypeModuleReference>,
    pub direct_imports: FxHashMap<CompactString, TypeModuleReference>,
    /// False when parsing failed or a bounded traversal could not load a file.
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeDeclarationId {
    pub module: CompactString,
    pub name: CompactString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDeclaration {
    pub kind: TypeDeclarationKind,
    pub body: CompactString,
    pub extends: Vec<CompactString>,
    /// Includes constraints, defaults and variance modifiers.
    pub type_parameters: Option<CompactString>,
    /// Exact declaration text, without the enclosing export statement.
    pub declaration_source: CompactString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDeclarationKind {
    Interface,
    Alias,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeModuleReference {
    pub specifier: CompactString,
    /// Canonical resolved module identity; absent means resolution failed.
    pub module: Option<CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeImport {
    pub target: TypeModuleReference,
    /// None represents a namespace import.
    pub exported: Option<CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExportBinding {
    Local(CompactString),
    Forward {
        target: TypeModuleReference,
        exported: CompactString,
    },
    Namespace(TypeModuleReference),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeLookup {
    Found(TypeDeclarationId),
    Unknown(UnknownTypeReference),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTypeReference {
    pub module: CompactString,
    pub name: CompactString,
    pub reason: UnknownTypeReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownTypeReason {
    Unbound,
    MissingModule,
    IncompleteModule,
    UnsupportedQualification,
    UnsupportedDeclaration,
    AmbiguousExport,
    ResolutionCycle,
    ResolutionLimit,
}

impl ResolvedTypeWorld {
    pub fn declaration(&self, id: &TypeDeclarationId) -> Option<&TypeDeclaration> {
        self.modules.get(&id.module)?.declarations.get(&id.name)
    }

    /// Keep every authored type declaration part observable even when semantic
    /// declaration merging is unsupported. Runtime implementation text is absent.
    pub fn unknown_contract(&self, reference: &UnknownTypeReference) -> Option<&[CompactString]> {
        self.modules
            .get(&reference.module)?
            .unsupported_contracts
            .get(&reference.name)
            .map(Vec::as_slice)
    }
}

impl std::fmt::Debug for super::TypeResolver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Keep the compatibility analysis dump stable when an authoritative
        // public-type world is attached. The world has its own Debug view.
        formatter
            .debug_struct("TypeResolver")
            .field("definitions", &self.definitions)
            .finish()
    }
}
