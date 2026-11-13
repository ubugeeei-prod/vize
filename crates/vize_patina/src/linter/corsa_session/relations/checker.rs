//! Cached checker queries shared by the relation replies.
mod lookups;
use super::super::checker_memo::CheckerMemo;
use corsa::api::{
    EncodedPayload, NodeHandle, ProjectSession, SignatureResponse, SymbolHandle, SymbolResponse,
    TypeHandle, TypeResponse,
};
use vize_l0::{FxHashMap, String as CompactString};

/// TypeScript 7 `TypeFlags`, from microsoft/typescript-go internal/checker/types.go.
pub(in crate::linter) mod flags {
    pub const ANY: u32 = 1;
    pub const UNKNOWN: u32 = 1 << 1;
    pub const UNDEFINED: u32 = 1 << 2;
    pub const NULL: u32 = 1 << 3;
    pub const VOID: u32 = 1 << 4;
    pub const STRING_LITERAL: u32 = 1 << 10;
    pub const NUMBER_LITERAL: u32 = 1 << 11;
    pub const BIGINT_LITERAL: u32 = 1 << 12;
    pub const BOOLEAN_LITERAL: u32 = 1 << 13;
    pub const NON_PRIMITIVE: u32 = 1 << 17;
    pub const TYPE_PARAMETER: u32 = 1 << 19;
    pub const OBJECT: u32 = 1 << 20;
    pub const INDEX: u32 = 1 << 21;
    pub const INDEXED_ACCESS: u32 = 1 << 25;
    pub const UNION: u32 = 1 << 27;
    pub const INTERSECTION: u32 = 1 << 28;
    /// `isTypeLiteralType(t) || isBooleanLiteralType(t)`.
    pub const LITERAL: u32 = STRING_LITERAL | NUMBER_LITERAL | BIGINT_LITERAL | BOOLEAN_LITERAL;
    /// `TypeFlags.TypeVariable | TypeFlags.Index`.
    pub const TYPE_VARIABLE: u32 = TYPE_PARAMETER | INDEXED_ACCESS | INDEX;
    /// Types `getBaseConstraintOfType` can resolve to something other than
    /// themselves: type variables, template/mapping types, and compositions.
    pub const INSTANTIABLE: u32 = TYPE_PARAMETER
        | INDEX
        | (1 << 22)
        | (1 << 23)
        | (1 << 24)
        | INDEXED_ACCESS
        | (1 << 26)
        | INTERSECTION;
    /// `isNullableType`: any, unknown, null, undefined or void.
    pub const NULLABLE: u32 = ANY | UNKNOWN | NULL | UNDEFINED | VOID;
}

/// `SignatureKind.Call` / `SignatureKind.Construct`.
pub(super) const CALL: i32 = 0;
pub(super) const CONSTRUCT: i32 = 1;

pub(super) struct Checker<'a> {
    session: &'a CheckerMemo<'a>,
    parts: FxHashMap<TypeHandle, Vec<TypeResponse>>,
    properties: FxHashMap<TypeHandle, Vec<SymbolResponse>>,
    signatures: FxHashMap<(TypeHandle, i32), Vec<SignatureResponse>>,
    assignable: FxHashMap<(TypeHandle, TypeHandle), bool>,
    files: FxHashMap<CompactString, Option<EncodedPayload>>,
    deprecations: FxHashMap<SymbolHandle, Option<CompactString>>,
}

impl<'a> Checker<'a> {
    pub(super) fn new(session: &'a CheckerMemo<'a>) -> Self {
        Self {
            session,
            parts: FxHashMap::default(),
            properties: FxHashMap::default(),
            signatures: FxHashMap::default(),
            assignable: FxHashMap::default(),
            files: FxHashMap::default(),
            deprecations: FxHashMap::default(),
        }
    }

    pub(super) fn session(&self) -> &'a ProjectSession {
        self.session
    }

    /// Encoded syntax of a declaring file, fetched once per batch.
    pub(super) async fn source_file(&mut self, path: &str) -> Option<&EncodedPayload> {
        if !self.files.contains_key(path) {
            let file = self
                .session
                .client()
                .get_source_file(
                    self.session.snapshot().handle.clone(),
                    self.session.project_handle(),
                    path,
                )
                .await
                .ok()
                .flatten();
            self.files.insert(CompactString::from(path), file);
        }
        self.files.get(path).and_then(Option::as_ref)
    }

    /// The text of a symbol's `@deprecated` JSDoc tag, once per symbol.
    pub(super) async fn deprecation_tag(
        &mut self,
        symbol: &SymbolResponse,
    ) -> Option<CompactString> {
        if let Some(known) = self.deprecations.get(&symbol.id) {
            return known.clone();
        }
        let found = self
            .session
            .client()
            .get_js_doc_tags(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                symbol.id.clone(),
            )
            .await
            .unwrap_or_default()
            .into_iter()
            .find(|tag| tag.name == "deprecated")
            .map(|tag| CompactString::from(tag.text.trim()));
        self.deprecations.insert(symbol.id.clone(), found.clone());
        found
    }

    pub(super) async fn aliased_symbol(
        &mut self,
        symbol: &SymbolResponse,
    ) -> Option<SymbolResponse> {
        self.session
            .client()
            .get_aliased_symbol(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                symbol.id.clone(),
            )
            .await
            .ok()
            .flatten()
    }

    pub(super) async fn immediate_aliased_symbol(
        &mut self,
        symbol: &SymbolResponse,
    ) -> Option<SymbolResponse> {
        self.session
            .client()
            .get_immediate_aliased_symbol(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                symbol.id.clone(),
            )
            .await
            .ok()
            .flatten()
    }

    pub(super) async fn shorthand_value_symbol(
        &mut self,
        node: &NodeHandle,
    ) -> Option<SymbolResponse> {
        self.session
            .client()
            .get_shorthand_assignment_value_symbol(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                node.clone(),
            )
            .await
            .ok()
            .flatten()
    }

    /// `checker.getSymbolAtLocation` for many nodes in one request.
    pub(super) async fn symbols_at(
        &mut self,
        nodes: Vec<NodeHandle>,
    ) -> Vec<Option<SymbolResponse>> {
        let count = nodes.len();
        let mut symbols = self
            .session
            .client()
            .get_symbols_at_locations(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                nodes,
            )
            .await
            .unwrap_or_default();
        symbols.resize(count, None);
        symbols
    }

    /// typescript-eslint `no-unnecessary-type-arguments`' comparison: the same
    /// type, or references to the same target with the same type arguments.
    pub(super) async fn same_type(&mut self, left: &TypeResponse, right: &TypeResponse) -> bool {
        if left.id == right.id {
            return true;
        }
        let (Some(left_target), Some(right_target)) = (&left.target, &right.target) else {
            return false;
        };
        if left_target != right_target || left.flags & flags::OBJECT == 0 {
            return false;
        }
        let left_arguments = self.reference_arguments(left).await;
        let right_arguments = self.reference_arguments(right).await;
        left_arguments.len() == right_arguments.len()
            && left_arguments
                .iter()
                .zip(&right_arguments)
                .all(|(left, right)| left.id == right.id)
    }

    async fn reference_arguments(&mut self, ty: &TypeResponse) -> Vec<TypeResponse> {
        self.session
            .client()
            .get_type_arguments(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or_default()
    }

    /// Union or intersection constituents; empty for any other type.
    pub(super) async fn parts(&mut self, ty: &TypeResponse) -> Vec<TypeResponse> {
        if ty.flags & (flags::UNION | flags::INTERSECTION) == 0 {
            return Vec::new();
        }
        if let Some(parts) = self.parts.get(&ty.id) {
            return parts.clone();
        }
        let parts = self.session.constituents(ty).await;
        self.parts.insert(ty.id.clone(), parts.clone());
        parts
    }

    /// `tsutils.unionConstituents(type)`.
    pub(super) async fn union_parts(&mut self, ty: &TypeResponse) -> Vec<TypeResponse> {
        if ty.flags & flags::UNION == 0 {
            return vec![ty.clone()];
        }
        let parts = self.parts(ty).await;
        if parts.is_empty() {
            vec![ty.clone()]
        } else {
            parts
        }
    }

    /// `getTypeFlags`: flags of every union constituent combined.
    pub(super) async fn union_flags(&mut self, ty: &TypeResponse) -> u32 {
        self.union_parts(ty)
            .await
            .iter()
            .fold(0, |combined, part| combined | part.flags)
    }

    /// `checker.getBaseConstraintOfType(type)`.
    pub(super) async fn base_constraint(&mut self, ty: &TypeResponse) -> Option<TypeResponse> {
        self.session.base_constraint(ty).await
    }

    pub(super) async fn contextual(&mut self, node: &NodeHandle) -> Option<TypeResponse> {
        self.session.contextual_type(node).await
    }

    /// `checker.getNonNullableType(type)`.
    pub(super) async fn non_nullable(&mut self, ty: &TypeResponse) -> Option<TypeResponse> {
        if ty.flags & (flags::UNION | flags::NULLABLE) == 0 {
            return Some(ty.clone());
        }
        self.session
            .client()
            .get_non_nullable_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .ok()
            .flatten()
    }

    /// `checker.isTypeAssignableTo(source, target)`.
    pub(super) async fn assignable(
        &mut self,
        source: &TypeResponse,
        target: &TypeResponse,
    ) -> bool {
        let key = (source.id.clone(), target.id.clone());
        if let Some(known) = self.assignable.get(&key) {
            return *known;
        }
        let value = self
            .session
            .client()
            .is_type_assignable_to(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                source.id.clone(),
                target.id.clone(),
            )
            .await
            .unwrap_or(false);
        self.assignable.insert(key, value);
        value
    }
}
