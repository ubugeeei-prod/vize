use corsa_core::fast::CompactString;

use super::{
    DocumentIdentifier, ProjectSession, SignatureHandle, SymbolHandle, SymbolResponse, TypeHandle,
    TypeResponse,
};
use crate::Result;

/// Version of the `corsa-bind` semantic query contract.
///
/// This number describes the *`corsa-bind`-owned* fact vocabulary in
/// [`SemanticQuery`], not the upstream Corsa API. Upstream endpoints may be
/// renamed, split, or replaced without changing this version, as long as the
/// questions below can still be answered. It is bumped only when the meaning of
/// an existing query changes.
pub const SEMANTIC_QUERY_VERSION: u32 = 1;

/// Symbol answer returned by [`SemanticQuery`].
///
/// This is deliberately *not* a mirror of the checker's `Symbol`. It carries an
/// opaque handle to ask further questions with, plus the display name that
/// Corsa already returned alongside it. Consumers that need the full upstream
/// shape should drop down to [`ProjectSession`] or `ApiClient` and accept that
/// they are coupling themselves to upstream's representation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SymbolRef {
    /// Opaque handle used to ask follow-up questions about this symbol.
    pub id: SymbolHandle,
    /// Display name of the symbol.
    pub name: CompactString,
}

/// Type answer returned by [`SemanticQuery`].
///
/// Like [`SymbolRef`], this is an opaque handle plus the rendering Corsa
/// already produced, not a projection of the checker's internal type
/// representation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeRef {
    /// Opaque handle used to ask follow-up questions about this type.
    pub id: TypeHandle,
    /// First non-empty rendering Corsa returned with the type, when it returned
    /// one. Use [`SemanticQuery::type_text`] to force a rendering.
    pub text: Option<CompactString>,
}

/// Signature kind accepted by [`SemanticQuery::signatures_of`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SignatureKind {
    /// Call signatures, as in `(): void`.
    Call,
    /// Construct signatures, as in `new (): Foo`.
    Construct,
}

impl SignatureKind {
    /// Returns the upstream wire tag for this kind.
    pub fn as_wire_value(self) -> i32 {
        match self {
            Self::Call => 0,
            Self::Construct => 1,
        }
    }
}

/// Declaration space a name is resolved in.
///
/// The numeric values mirror TypeScript's `SymbolFlags.Value`,
/// `SymbolFlags.Type`, and `SymbolFlags.Namespace` composites, which upstream
/// defines in `internal/ast/symbolflags.go`. They are wire inputs rather than
/// checker semantics: `corsa-bind` forwards them, it does not interpret them.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NameMeaning {
    /// Value space: variables, functions, classes, enum members, and friends.
    Value,
    /// Type space: classes, interfaces, enums, type literals, aliases.
    Type,
    /// Namespace space: modules and enums.
    Namespace,
}

impl NameMeaning {
    /// Returns the upstream `SymbolFlags` composite for this meaning.
    pub fn as_wire_value(self) -> u32 {
        match self {
            Self::Value => 111_551,
            Self::Type => 788_968,
            Self::Namespace => 1_920,
        }
    }
}

/// Stable semantic-fact queries against a live [`ProjectSession`].
///
/// # Why this exists
///
/// [`ProjectSession`] and `ApiClient` mirror upstream Corsa endpoint names on
/// purpose, so that new upstream capability is cheap to surface and easy to
/// audit. That makes them a moving target: when upstream renames an endpoint or
/// reshapes a response, every consumer moves with it.
///
/// `SemanticQuery` is the other half of the contract. It is the small,
/// `corsa-bind`-owned vocabulary that foreign hosts — Oxlint rules, framework
/// tooling, other language bindings — are expected to build against:
///
/// - answers are **opaque handles**, never mirrored checker object graphs
/// - method names are ours and stay stable across upstream churn
/// - every query is a question about the checker, never a claim about it
///
/// In other words, `corsa-bind` owns *how you ask*; upstream owns *what the
/// answer means*. See `docs/architecture_charter.md`.
///
/// # Examples
///
/// ```no_run
/// use corsa_client::{ApiSpawnConfig, ProjectSession};
///
/// # async fn example() -> corsa_client::Result<()> {
/// let session = ProjectSession::spawn(
///     ApiSpawnConfig::new("corsa"),
///     "/workspace/tsconfig.json",
///     Some("/workspace/src/index.ts".into()),
/// )
/// .await?;
///
/// let facts = session.semantics();
/// if let Some(symbol) = facts.symbol_at("/workspace/src/index.ts", 1).await? {
///     if let Some(ty) = facts.type_of(&symbol.id).await? {
///         println!("{} : {}", symbol.name, facts.type_text(&ty.id).await?);
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct SemanticQuery<'session> {
    session: &'session ProjectSession,
}

impl<'session> SemanticQuery<'session> {
    pub(crate) fn new(session: &'session ProjectSession) -> Self {
        Self { session }
    }

    /// Returns the session this query reads from.
    ///
    /// Use it when a caller genuinely needs an upstream-shaped endpoint that the
    /// stable vocabulary does not cover.
    pub fn session(&self) -> &'session ProjectSession {
        self.session
    }

    /// Resolves the symbol visible at a UTF-16 position.
    pub async fn symbol_at(
        &self,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<SymbolRef>> {
        Ok(self
            .session
            .get_symbol_at_position(file, position)
            .await?
            .map(symbol_ref))
    }

    /// Resolves the type visible at a UTF-16 position.
    pub async fn type_at(
        &self,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<TypeRef>> {
        Ok(self
            .session
            .get_type_at_position(file, position)
            .await?
            .map(type_ref))
    }

    /// Resolves types for several UTF-16 positions in one file with one request.
    ///
    /// Output order matches `positions`.
    pub async fn types_at(
        &self,
        file: impl Into<DocumentIdentifier>,
        positions: Vec<u32>,
    ) -> Result<Vec<Option<TypeRef>>> {
        let session = self.session;
        Ok(session
            .client()
            .get_types_at_positions(
                session.snapshot().handle.clone(),
                session.project_handle(),
                file,
                positions,
            )
            .await?
            .into_iter()
            .map(|response| response.map(type_ref))
            .collect())
    }

    /// Returns the type of a symbol at its declaration site.
    pub async fn type_of(&self, symbol: &SymbolHandle) -> Result<Option<TypeRef>> {
        Ok(self
            .session
            .get_type_of_symbol(symbol.clone())
            .await?
            .map(type_ref))
    }

    /// Returns the types of several symbols with one request.
    ///
    /// Output order matches `symbols`.
    pub async fn types_of(&self, symbols: Vec<SymbolHandle>) -> Result<Vec<Option<TypeRef>>> {
        Ok(self
            .session
            .get_types_of_symbols(symbols)
            .await?
            .into_iter()
            .map(|response| response.map(type_ref))
            .collect())
    }

    /// Returns the type a type-space symbol declares, as in `type Foo = ...`.
    pub async fn declared_type_of(&self, symbol: &SymbolHandle) -> Result<Option<TypeRef>> {
        let session = self.session;
        Ok(session
            .client()
            .get_declared_type_of_symbol(
                session.snapshot().handle.clone(),
                session.project_handle(),
                symbol.clone(),
            )
            .await?
            .map(type_ref))
    }

    /// Resolves a name to a symbol as the checker would see it from `position`.
    pub async fn resolve_symbol(
        &self,
        name: impl Into<String>,
        meaning: NameMeaning,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<SymbolRef>> {
        let session = self.session;
        Ok(session
            .client()
            .resolve_name(
                session.snapshot().handle.clone(),
                session.project_handle(),
                name,
                meaning.as_wire_value(),
                None,
                Some(file.into()),
                Some(position),
                None,
            )
            .await?
            .map(symbol_ref))
    }

    /// Resolves a type name such as `"Foo"` to the type it declares.
    ///
    /// This is the two-step "resolve in type space, then take the declared
    /// type" flow that type-aware rules keep re-implementing by hand.
    pub async fn resolve_type(
        &self,
        name: impl Into<String>,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<TypeRef>> {
        let Some(symbol) = self
            .resolve_symbol(name, NameMeaning::Type, file, position)
            .await?
        else {
            return Ok(None);
        };
        self.declared_type_of(&symbol.id).await
    }

    /// Returns the property symbols of a type.
    pub async fn properties_of(&self, r#type: &TypeHandle) -> Result<Vec<SymbolRef>> {
        Ok(self
            .session
            .get_properties_of_type(r#type.clone())
            .await?
            .into_iter()
            .map(symbol_ref)
            .collect())
    }

    /// Returns one named property symbol of a type, when it has one.
    pub async fn property_of(
        &self,
        r#type: &TypeHandle,
        name: impl Into<String>,
    ) -> Result<Option<SymbolRef>> {
        let session = self.session;
        Ok(session
            .client()
            .get_property_of_type(
                session.snapshot().handle.clone(),
                session.project_handle(),
                r#type.clone(),
                name,
            )
            .await?
            .map(symbol_ref))
    }

    /// Returns the signatures of a type for one signature kind.
    pub async fn signatures_of(
        &self,
        r#type: &TypeHandle,
        kind: SignatureKind,
    ) -> Result<Vec<SignatureHandle>> {
        Ok(self
            .session
            .get_signatures_of_type(r#type.clone(), kind.as_wire_value())
            .await?
            .into_iter()
            .map(|signature| signature.id)
            .collect())
    }

    /// Returns the return type of a signature.
    pub async fn return_type_of(&self, signature: &SignatureHandle) -> Result<Option<TypeRef>> {
        Ok(self
            .session
            .get_return_type_of_signature(signature.clone())
            .await?
            .map(type_ref))
    }

    /// Returns whether `source` is assignable to `target`.
    ///
    /// This asks the checker's own assignability relation. Prefer it over
    /// comparing rendered type texts, which cannot model structural
    /// assignability.
    pub async fn is_assignable(&self, source: &TypeHandle, target: &TypeHandle) -> Result<bool> {
        let session = self.session;
        session
            .client()
            .is_type_assignable_to(
                session.snapshot().handle.clone(),
                session.project_handle(),
                source.clone(),
                target.clone(),
            )
            .await
    }

    /// Renders a type back into TypeScript source text.
    pub async fn type_text(&self, r#type: &TypeHandle) -> Result<CompactString> {
        Ok(CompactString::from(
            self.session
                .type_to_string(r#type.clone(), None, None)
                .await?,
        ))
    }
}

fn symbol_ref(response: SymbolResponse) -> SymbolRef {
    SymbolRef {
        id: response.id,
        name: CompactString::from(response.name),
    }
}

fn type_ref(response: TypeResponse) -> TypeRef {
    let text = response
        .texts
        .into_iter()
        .map(|text| text.trim().to_owned())
        .find(|text| !text.is_empty())
        .map(CompactString::from);
    TypeRef {
        id: response.id,
        text,
    }
}

impl ProjectSession {
    /// Returns the stable semantic-fact query surface for this session.
    ///
    /// See [`SemanticQuery`] for why this exists next to the upstream-shaped
    /// endpoint methods.
    pub fn semantics(&self) -> SemanticQuery<'_> {
        SemanticQuery::new(self)
    }
}
