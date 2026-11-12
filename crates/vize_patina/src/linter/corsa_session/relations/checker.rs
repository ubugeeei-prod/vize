//! Cached checker queries shared by the relation replies.
use super::Shape;
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
    pub const STRING: u32 = 1 << 5;
    pub const NUMBER: u32 = 1 << 6;
    pub const STRING_LITERAL: u32 = 1 << 10;
    pub const NUMBER_LITERAL: u32 = 1 << 11;
    pub const BIGINT_LITERAL: u32 = 1 << 12;
    pub const BOOLEAN_LITERAL: u32 = 1 << 13;
    pub const ENUM: u32 = 1 << 16;
    pub const NON_PRIMITIVE: u32 = 1 << 17;
    pub const TYPE_PARAMETER: u32 = 1 << 19;
    pub const OBJECT: u32 = 1 << 20;
    pub const INDEX: u32 = 1 << 21;
    pub const INDEXED_ACCESS: u32 = 1 << 25;
    pub const UNION: u32 = 1 << 27;
    pub const INTERSECTION: u32 = 1 << 28;
    /// `isTypeLiteralType(t) || isBooleanLiteralType(t)`.
    pub const LITERAL: u32 = STRING_LITERAL | NUMBER_LITERAL | BIGINT_LITERAL | BOOLEAN_LITERAL;
    /// `TypeFlags.NumberLike`.
    pub const NUMBER_LIKE: u32 = NUMBER | NUMBER_LITERAL | ENUM;
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
    session: &'a ProjectSession,
    parts: FxHashMap<TypeHandle, Vec<TypeResponse>>,
    properties: FxHashMap<TypeHandle, Vec<SymbolResponse>>,
    signatures: FxHashMap<(TypeHandle, i32), Vec<SignatureResponse>>,
    assignable: FxHashMap<(TypeHandle, TypeHandle), bool>,
    files: FxHashMap<CompactString, Option<EncodedPayload>>,
    deprecations: FxHashMap<SymbolHandle, Option<CompactString>>,
}

impl<'a> Checker<'a> {
    pub(super) fn new(session: &'a ProjectSession) -> Self {
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
        let parts = self
            .session
            .client()
            .get_types_of_type_in_project(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or_default();
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
        self.session
            .client()
            .get_base_constraint_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .ok()
            .flatten()
    }

    pub(super) async fn contextual(&mut self, node: &NodeHandle) -> Option<TypeResponse> {
        self.session
            .client()
            .get_contextual_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                node.clone(),
            )
            .await
            .ok()
            .flatten()
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

    pub(super) async fn properties(&mut self, ty: &TypeResponse) -> Vec<SymbolResponse> {
        if let Some(properties) = self.properties.get(&ty.id) {
            return properties.clone();
        }
        let properties = self
            .session
            .get_properties_of_type(ty.id.clone())
            .await
            .unwrap_or_default();
        self.properties.insert(ty.id.clone(), properties.clone());
        properties
    }

    pub(super) async fn property(
        &mut self,
        ty: &TypeResponse,
        name: &str,
    ) -> Option<SymbolResponse> {
        self.session
            .client()
            .get_property_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
                name,
            )
            .await
            .ok()
            .flatten()
    }

    pub(super) async fn type_of_symbol(&mut self, symbol: &SymbolResponse) -> Option<TypeResponse> {
        self.session
            .get_type_of_symbol(symbol.id.clone())
            .await
            .ok()
            .flatten()
    }

    pub(super) async fn signatures(
        &mut self,
        ty: &TypeResponse,
        kind: i32,
    ) -> Vec<SignatureResponse> {
        let key = (ty.id.clone(), kind);
        if let Some(signatures) = self.signatures.get(&key) {
            return signatures.clone();
        }
        let signatures = self
            .session
            .get_signatures_of_type(ty.id.clone(), kind)
            .await
            .unwrap_or_default();
        self.signatures.insert(key, signatures.clone());
        signatures
    }

    pub(super) async fn return_type(
        &mut self,
        signature: &SignatureResponse,
    ) -> Option<TypeResponse> {
        self.session
            .get_return_type_of_signature(signature.id.clone())
            .await
            .ok()
            .flatten()
    }

    /// Types of a signature's parameters, in order.
    pub(super) async fn parameter_types(
        &mut self,
        signature: &SignatureResponse,
    ) -> Vec<TypeResponse> {
        let mut types = Vec::new();
        for parameter in &signature.parameters {
            if let Ok(Some(ty)) = self.session.get_type_of_symbol(parameter.clone()).await {
                types.push(ty);
            }
        }
        types
    }

    pub(super) async fn index_info_count(&mut self, ty: &TypeResponse) -> usize {
        self.session
            .client()
            .get_index_infos_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .map(|infos| infos.len())
            .unwrap_or(0)
    }

    /// `aliasTypeArguments ?? (isTypeReference(type) ? checker.getTypeArguments(type) : [])`.
    pub(super) async fn type_arguments(&mut self, ty: &TypeResponse) -> Vec<TypeResponse> {
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let alias = self
            .session
            .client()
            .get_alias_type_arguments_of_type(snapshot.clone(), project.clone(), ty.id.clone())
            .await
            .unwrap_or_default();
        if !alias.is_empty() || ty.flags & flags::OBJECT == 0 {
            return alias;
        }
        self.session
            .client()
            .get_type_arguments(snapshot, project, ty.id.clone())
            .await
            .unwrap_or_default()
    }

    /// The uncast type of an immediately invoked function: the return type of
    /// its callee's first call signature, or `void` when the callee declares
    /// no return type and returns `undefined`.
    pub(super) async fn iife_return(
        &mut self,
        callee: &TypeResponse,
        annotated: bool,
    ) -> Option<TypeResponse> {
        let signature = self.signatures(callee, CALL).await.into_iter().next()?;
        let returned = self.return_type(&signature).await?;
        if !annotated && self.union_flags(&returned).await & flags::UNDEFINED != 0 {
            return self
                .session
                .client()
                .get_void_type(
                    self.session.snapshot().handle.clone(),
                    self.session.project_handle(),
                )
                .await
                .ok();
        }
        Some(returned)
    }

    async fn is_array(&mut self, ty: &TypeResponse) -> bool {
        self.session
            .client()
            .is_array_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or(false)
    }

    /// `checker.isTupleType`: a reference whose target is a tuple type.
    async fn is_tuple(&mut self, ty: &TypeResponse) -> bool {
        const REFERENCE: u32 = 1 << 2;
        const TUPLE: u32 = 1 << 3;
        if ty.object_flags.unwrap_or(0) & REFERENCE != 0 && ty.target.is_some() {
            let target = self
                .session
                .client()
                .get_target_of_type_in_project(
                    self.session.snapshot().handle.clone(),
                    self.session.project_handle(),
                    ty.id.clone(),
                )
                .await
                .ok()
                .flatten();
            if target.is_some_and(|target| target.object_flags.unwrap_or(0) & TUPLE != 0) {
                return true;
            }
        }
        self.session
            .client()
            .is_tuple_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or(false)
    }

    async fn has_number_index(&mut self, ty: &TypeResponse) -> bool {
        self.session
            .client()
            .get_index_infos_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or_default()
            .iter()
            .any(|info| info.key_type.flags & flags::NUMBER != 0)
    }

    async fn has_number_length(&mut self, ty: &TypeResponse) -> bool {
        let Some(length) = self.property(ty, "length").await else {
            return false;
        };
        let Some(length) = self.type_of_symbol(&length).await else {
            return false;
        };
        self.union_flags(&length).await & flags::NUMBER_LIKE != 0
    }

    /// Union/intersection structure of the base-constrained type, two levels deep.
    pub(super) async fn shape(&mut self, ty: &TypeResponse) -> Option<Shape> {
        let constrained = self.base_constraint(ty).await.unwrap_or_else(|| ty.clone());
        let mut root = self.shape_of(&constrained).await;
        let mut parts = Vec::new();
        for part in self.parts(&constrained).await {
            let mut shape = self.shape_of(&part).await;
            for inner in self.parts(&part).await {
                let inner_shape = self.shape_of(&inner).await;
                shape.parts.push(inner_shape);
            }
            parts.push(shape);
        }
        root.parts = parts;
        Some(root)
    }

    async fn shape_of(&mut self, ty: &TypeResponse) -> Shape {
        let composite = ty.flags & (flags::UNION | flags::INTERSECTION) != 0;
        let object = ty.flags & flags::OBJECT != 0;
        Shape {
            flags: ty.flags,
            union: ty.flags & flags::UNION != 0,
            intersection: ty.flags & flags::INTERSECTION != 0,
            array: object && self.is_array(ty).await,
            tuple: object && self.is_tuple(ty).await,
            number_index: !composite && self.has_number_index(ty).await,
            number_length: !composite && self.has_number_length(ty).await,
            parts: Vec::new(),
        }
    }
}
