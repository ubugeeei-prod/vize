//! Promise, thenable and Error facts from the actual checker.
//!
//! These mirror typescript-eslint's type helpers (`isThenableType`,
//! `needsToBeAwaited`, `isBuiltinSymbolLike`, `isVoidReturningFunctionType`,
//! ...) so the promise rules decide on checker semantics, not on rendered
//! type names. Every answer is memoized per type handle.
use corsa::api::{
    ProjectSession, SignatureResponse, SymbolHandle, SymbolResponse, TypeHandle, TypeResponse,
};
use vize_l0::{FxHashMap, String as CompactString};

// TypeScript 7 TypeFlags, from microsoft/typescript-go internal/checker/types.go.
const ANY: u32 = 1;
const UNKNOWN: u32 = 1 << 1;
const UNDEFINED: u32 = 1 << 2;
const VOID: u32 = 1 << 4;
const NUMBER: u32 = 1 << 6;
const NEVER: u32 = 1 << 18;
const TYPE_PARAMETER: u32 = 1 << 19;
const UNION: u32 = 1 << 27;
const INTERSECTION: u32 = 1 << 28;
// TypeParameter | Index | IndexedAccess | Conditional | Substitution.
const INSTANTIABLE: u32 = TYPE_PARAMETER | (1 << 21) | (1 << 25) | (1 << 26) | (1 << 24);
// ObjectFlags.
const REFERENCE: u32 = 1 << 2;
const TUPLE: u32 = 1 << 3;
// SymbolFlags.
const CLASS: u32 = 1 << 5;
const INTERFACE: u32 = 1 << 6;
// SignatureKind and SignatureFlags.
const CALL: i32 = 0;
const CONSTRUCT: i32 = 1;
const HAS_REST_PARAMETER: u32 = 1;
/// Bounds every structural walk, so self-referential types cannot loop.
const DEPTH: u32 = 16;

/// `needsToBeAwaited` certainty.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::linter) enum Awaitable {
    #[default]
    Never,
    May,
    Always,
}

impl Awaitable {
    pub(in crate::linter) const fn as_str(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::May => "may",
            Self::Always => "always",
        }
    }
}

/// Which promise facts a query wants, as a bit set. The host asks only for
/// what the enabled rules read at that syntax position.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::linter) struct Facets(pub u16);

impl Facets {
    pub(in crate::linter) const AWAITABLE: u16 = 1;
    pub(in crate::linter) const THENABLE: u16 = 1 << 1;
    pub(in crate::linter) const ALWAYS_THENABLE: u16 = 1 << 2;
    pub(in crate::linter) const RETURNS_THENABLE: u16 = 1 << 3;
    pub(in crate::linter) const VOID_FUNCTION: u16 = 1 << 4;
    pub(in crate::linter) const PROMISE: u16 = 1 << 5;
    pub(in crate::linter) const PROMISE_ARRAY: u16 = 1 << 6;
    pub(in crate::linter) const CALLABLE: u16 = 1 << 7;
    pub(in crate::linter) const ERROR: u16 = 1 << 8;
    pub(in crate::linter) const PROMISE_CONSTRUCTOR: u16 = 1 << 9;
    /// await-thenable's promise aggregator input facts.
    pub(in crate::linter) const AGGREGATOR: u16 = 1 << 10;
    pub(in crate::linter) const ASYNC_ITERABLE: u16 = 1 << 11;
    pub(in crate::linter) const ASYNC_DISPOSABLE: u16 = 1 << 12;
    /// no-misused-promises `hasMatchingPromiseTypeArgument`.
    pub(in crate::linter) const MATCHING_UNION: u16 = 1 << 13;
    /// use-unknown-in-catch-callback-variable `isFlaggableHandlerType`.
    pub(in crate::linter) const CATCH_HANDLER: u16 = 1 << 14;
    /// `isArrayMethodCallWithPredicate`'s array or tuple receiver.
    pub(in crate::linter) const ARRAY_OR_TUPLE: u16 = 1 << 15;

    pub(in crate::linter) const fn has(self, facet: u16) -> bool {
        self.0 & facet != 0
    }

    pub(in crate::linter) const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Facts about one expression type; `None` when not requested.
#[derive(Clone, Copy, Debug, Default)]
pub(in crate::linter) struct PromiseFacts {
    /// Raw TypeFlags of the expression type.
    pub flags: u32,
    /// `needsToBeAwaited` on the expression type.
    pub awaitable: Option<Awaitable>,
    /// Some union part of the apparent type is thenable.
    pub thenable: Option<bool>,
    /// Every union part of the apparent type is thenable (`isAlwaysThenable`).
    pub always_thenable: Option<bool>,
    /// A call signature of the apparent type returns a thenable (`returnsThenable`).
    pub returns_thenable: Option<bool>,
    /// The type is a function returning `void` and never a thenable.
    pub void_function: Option<bool>,
    /// A union part of the apparent type is the built-in `Promise`.
    pub promise: Option<bool>,
    /// The type is an array or tuple of built-in Promises.
    pub promise_array: Option<bool>,
    /// The type has call signatures.
    pub callable: Option<bool>,
    /// `isErrorLike || isReadonlyErrorLike`.
    pub error: Option<bool>,
    /// `isPromiseConstructorLike` of the constrained type.
    pub promise_constructor: Option<bool>,
    /// `isAlwaysNonAwaitableType` of the constrained type.
    pub non_awaitable: Option<bool>,
    /// `isInvalidPromiseAggregatorInput` of the constrained type.
    pub invalid_aggregator_input: Option<bool>,
    /// `hasMaybeAwaitableValue` of the constrained type.
    pub maybe_awaitable_value: Option<bool>,
    /// A union part has a `[Symbol.asyncIterator]` member.
    pub async_iterable: Option<bool>,
    /// A union part has a `[Symbol.asyncDispose]` member.
    pub async_disposable: Option<bool>,
    /// `hasMatchingPromiseTypeArgument`: `Promise<T> | T`.
    pub matching_promise_union: Option<bool>,
    /// A call signature's first parameter is not the intrinsic `unknown`.
    pub flaggable_handler: Option<bool>,
    /// A union or intersection part of the constrained type is an array or tuple.
    pub array_or_tuple: Option<bool>,
}

/// Memoized checker walks for one snapshot.
pub(in crate::linter) struct PromiseChecker<'a> {
    session: &'a ProjectSession,
    facts: FxHashMap<(TypeHandle, u16), PromiseFacts>,
    thenable: FxHashMap<TypeHandle, bool>,
    apparent: FxHashMap<TypeHandle, TypeResponse>,
    parts: FxHashMap<TypeHandle, Vec<TypeResponse>>,
    signatures: FxHashMap<(TypeHandle, i32), Vec<SignatureResponse>>,
    symbol_types: FxHashMap<SymbolHandle, Option<TypeResponse>>,
    members: FxHashMap<TypeHandle, Vec<CompactString>>,
}

impl<'a> PromiseChecker<'a> {
    pub(in crate::linter) fn new(session: &'a ProjectSession) -> Self {
        Self {
            session,
            facts: FxHashMap::default(),
            thenable: FxHashMap::default(),
            apparent: FxHashMap::default(),
            parts: FxHashMap::default(),
            signatures: FxHashMap::default(),
            symbol_types: FxHashMap::default(),
            members: FxHashMap::default(),
        }
    }

    pub(in crate::linter) async fn facts(
        &mut self,
        ty: &TypeResponse,
        wants: Facets,
    ) -> corsa::Result<PromiseFacts> {
        let key = (ty.id.clone(), wants.0);
        if let Some(facts) = self.facts.get(&key) {
            return Ok(*facts);
        }
        let mut facts = PromiseFacts {
            flags: ty.flags,
            ..PromiseFacts::default()
        };
        if wants.has(Facets::AWAITABLE) {
            facts.awaitable = Some(self.awaitable(ty).await?);
        }
        if wants.has(Facets::THENABLE) {
            facts.thenable = Some(self.is_thenable(ty, 0).await?);
        }
        if wants.has(Facets::ALWAYS_THENABLE) {
            facts.always_thenable = Some(self.always_thenable(ty).await?);
        }
        if wants.has(Facets::RETURNS_THENABLE) {
            facts.returns_thenable = Some(self.returns_thenable(ty).await?);
        }
        if wants.has(Facets::VOID_FUNCTION) {
            facts.void_function = Some(self.void_function(ty).await?);
        }
        if wants.has(Facets::PROMISE) {
            facts.promise = Some(self.promise_like(ty).await?);
        }
        if wants.has(Facets::PROMISE_ARRAY) {
            facts.promise_array = Some(self.promise_array(ty).await?);
        }
        if wants.has(Facets::CALLABLE) {
            facts.callable = Some(!self.signatures(ty, CALL).await?.is_empty());
        }
        if wants.has(Facets::ERROR) {
            facts.error =
                Some(self.builtin(ty, &["Error"], 0).await? || self.readonly_error(ty, 0).await?);
        }
        if wants.has(Facets::PROMISE_CONSTRUCTOR) {
            let constrained = self.constrained(ty).await?;
            facts.promise_constructor = Some(
                self.builtin(&constrained, &["PromiseConstructor"], 0)
                    .await?,
            );
        }
        if wants.has(Facets::AGGREGATOR) {
            let constrained = self.constrained(ty).await?;
            facts.non_awaitable = Some(self.always_non_awaitable(&constrained).await?);
            facts.invalid_aggregator_input =
                Some(self.invalid_aggregator_input(&constrained).await?);
            facts.maybe_awaitable_value = Some(self.maybe_awaitable_value(&constrained).await?);
        }
        if wants.has(Facets::ASYNC_ITERABLE) {
            facts.async_iterable = Some(self.some_part_has_symbol(ty, "asyncIterator").await?);
        }
        if wants.has(Facets::MATCHING_UNION) {
            facts.matching_promise_union = Some(self.matching_promise_union(ty).await?);
        }
        if wants.has(Facets::ARRAY_OR_TUPLE) {
            let constrained = self.constrained(ty).await?;
            facts.array_or_tuple = Some(self.array_or_tuple(&constrained).await?);
        }
        if wants.has(Facets::CATCH_HANDLER) {
            facts.flaggable_handler = Some(self.flaggable_handler(ty).await?);
        }
        if wants.has(Facets::ASYNC_DISPOSABLE) {
            facts.async_disposable = Some(self.some_part_has_symbol(ty, "asyncDispose").await?);
        }
        self.facts.insert(key, facts);
        Ok(facts)
    }

    /// `isVoidReturningFunctionType` over the parts of `ty`.
    pub(in crate::linter) async fn void_function(
        &mut self,
        ty: &TypeResponse,
    ) -> corsa::Result<bool> {
        let mut had_void = false;
        for part in self.union_parts(ty).await? {
            for signature in self.signatures(&part, CALL).await? {
                let Some(returned) = self.return_type(&signature).await? else {
                    continue;
                };
                if self.is_thenable(&returned, 0).await? {
                    return Ok(false);
                }
                had_void |= returned.flags & VOID != 0;
            }
        }
        Ok(had_void)
    }

    /// `isThenableReturningFunctionType` over the parts of `ty`.
    pub(in crate::linter) async fn thenable_function(
        &mut self,
        ty: &TypeResponse,
    ) -> corsa::Result<bool> {
        for part in self.union_parts(ty).await? {
            for signature in self.signatures(&part, CALL).await? {
                if let Some(returned) = self.return_type(&signature).await?
                    && self.is_thenable(&returned, 0).await?
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Argument indices whose parameter expects a void-returning (and never a
    /// thenable-returning) function, as in `voidFunctionArguments`.
    pub(in crate::linter) async fn void_arguments(
        &mut self,
        callee: &TypeResponse,
        construct: bool,
        argument_count: usize,
    ) -> corsa::Result<(Vec<usize>, Vec<usize>)> {
        let mut thenable = Vec::new();
        let mut void = Vec::new();
        let kind = if construct { CONSTRUCT } else { CALL };
        for part in self.union_parts(callee).await? {
            for signature in self.signatures(&part, kind).await? {
                let count = signature.parameters.len();
                let rest = signature.flags & HAS_REST_PARAMETER != 0;
                // `getTypeAtPosition` already spreads a rest array or tuple
                // over the trailing argument positions.
                for slot in 0..argument_count {
                    if slot >= count && !rest {
                        break;
                    }
                    let Some(ty) = self.parameter_type(&signature, slot).await? else {
                        continue;
                    };
                    self.classify_argument(&ty, slot, &mut thenable, &mut void)
                        .await?;
                }
            }
        }
        void.retain(|index| !thenable.contains(index));
        void.sort_unstable();
        void.dedup();
        thenable.sort_unstable();
        thenable.dedup();
        Ok((void, thenable))
    }

    /// Heritage types of a class or interface, and for each member name that
    /// one of them declares as a void-returning function, the member and the
    /// heritage type's text (`checkHeritageTypeForMemberReturningVoid`).
    pub(in crate::linter) async fn heritage_voids(
        &mut self,
        ty: &TypeResponse,
        base_types: bool,
        members: &[CompactString],
    ) -> corsa::Result<Vec<(CompactString, CompactString)>> {
        let client = self.session.client();
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let heritage = if base_types {
            // A class expression's type is its constructor.
            let constructors = self.signatures(ty, CONSTRUCT).await?;
            let instance = match constructors.first() {
                Some(signature) => self
                    .return_type(signature)
                    .await?
                    .unwrap_or_else(|| ty.clone()),
                None => ty.clone(),
            };
            client
                .get_base_types(snapshot.clone(), project.clone(), instance.id.clone())
                .await
                .unwrap_or_default()
        } else {
            vec![ty.clone()]
        };
        let mut found = Vec::new();
        for base in &heritage {
            let mut text = None;
            for name in members {
                let property = client
                    .get_property_of_type(
                        snapshot.clone(),
                        project.clone(),
                        base.id.clone(),
                        name.as_str(),
                    )
                    .await
                    .unwrap_or_default();
                let Some(property) = property else {
                    continue;
                };
                let Some(member) = self.symbol_type(&property.id).await? else {
                    continue;
                };
                if !self.void_function(&member).await? {
                    continue;
                }
                if text.is_none() {
                    text = self
                        .session
                        .render_type_texts(base)
                        .await
                        .unwrap_or_default()
                        .into_iter()
                        .next();
                }
                if let Some(text) = &text {
                    found.push((name.clone(), text.clone()));
                }
            }
        }
        Ok(found)
    }

    /// For each method name, whether the property of that name on the
    /// contextual object type is a void-returning function.
    pub(in crate::linter) async fn member_void_functions(
        &mut self,
        object: &TypeResponse,
        names: &[CompactString],
    ) -> corsa::Result<Vec<(CompactString, bool)>> {
        let mut result = Vec::new();
        let parts = self.union_parts(object).await?;
        for name in names {
            for part in &parts {
                let property = self
                    .session
                    .client()
                    .get_property_of_type(
                        self.session.snapshot().handle.clone(),
                        self.session.project_handle(),
                        part.id.clone(),
                        name.as_str(),
                    )
                    .await
                    .unwrap_or_default();
                let Some(property) = property else {
                    continue;
                };
                if let Some(ty) = self.symbol_type(&property.id).await? {
                    let void = self.void_function(&ty).await?;
                    result.push((name.clone(), void));
                }
                break;
            }
        }
        Ok(result)
    }

    async fn classify_argument(
        &mut self,
        ty: &TypeResponse,
        index: usize,
        thenable: &mut Vec<usize>,
        void: &mut Vec<usize>,
    ) -> corsa::Result<()> {
        if self.thenable_function(ty).await? {
            thenable.push(index);
        } else if self.void_function(ty).await? && !thenable.contains(&index) {
            void.push(index);
        }
        Ok(())
    }

    /// `getConstrainedTypeAtLocation`: the base constraint of a generic type.
    async fn constrained(&mut self, ty: &TypeResponse) -> corsa::Result<TypeResponse> {
        if ty.flags & INSTANTIABLE == 0 {
            return Ok(ty.clone());
        }
        Ok(self
            .session
            .client()
            .get_base_constraint_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or_default()
            .unwrap_or_else(|| ty.clone()))
    }

    /// `hasMatchingPromiseTypeArgument`: the awaited types of the thenable
    /// parts and the other parts are the same set of types. A thenable's
    /// awaited type is read from its first type argument.
    async fn matching_promise_union(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let apparent = self.apparent(ty).await?;
        if apparent.flags & UNION == 0 {
            return Ok(false);
        }
        let mut thenables = Vec::new();
        let mut others = Vec::new();
        for part in self.union_parts(&apparent).await? {
            if self.is_thenable(&part, 0).await? {
                thenables.push(part);
            } else {
                others.push(part);
            }
        }
        if thenables.is_empty() {
            return Ok(false);
        }
        let mut awaited = Vec::new();
        for thenable in &thenables {
            let Some(inner) = self.awaited(thenable).await? else {
                return Ok(false);
            };
            awaited.extend(self.union_parts(&inner).await?);
        }
        for other in &others {
            if !self.any_equivalent(other, &awaited).await? {
                return Ok(false);
            }
        }
        for inner in &awaited {
            if !self.any_equivalent(inner, &others).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `getAwaitedType` for a thenable reference: unwrap first type arguments
    /// while they stay thenable.
    async fn awaited(&mut self, ty: &TypeResponse) -> corsa::Result<Option<TypeResponse>> {
        let mut current = ty.clone();
        for _ in 0..DEPTH {
            if !self.is_thenable(&current, 0).await? {
                return Ok(Some(current));
            }
            let arguments = self
                .session
                .client()
                .get_type_arguments(
                    self.session.snapshot().handle.clone(),
                    self.session.project_handle(),
                    current.id.clone(),
                )
                .await
                .unwrap_or_default();
            let Some(first) = arguments.into_iter().next() else {
                return Ok(None);
            };
            current = first;
        }
        Ok(None)
    }

    async fn any_equivalent(
        &mut self,
        ty: &TypeResponse,
        candidates: &[TypeResponse],
    ) -> corsa::Result<bool> {
        let client = self.session.client();
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        for candidate in candidates {
            if candidate.id == ty.id {
                return Ok(true);
            }
            if client
                .is_type_assignable_to(
                    snapshot.clone(),
                    project.clone(),
                    ty.id.clone(),
                    candidate.id.clone(),
                )
                .await?
                && client
                    .is_type_assignable_to(
                        snapshot.clone(),
                        project.clone(),
                        candidate.id.clone(),
                        ty.id.clone(),
                    )
                    .await?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn array_or_tuple(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let client = self.session.client();
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        for part in self.union_parts(ty).await? {
            let members = if part.flags & INTERSECTION != 0 {
                client
                    .get_types_of_type_in_project(
                        snapshot.clone(),
                        project.clone(),
                        part.id.clone(),
                    )
                    .await?
            } else {
                vec![part]
            };
            for member in &members {
                if self.is_tuple(member).await?
                    || client
                        .is_array_type(snapshot.clone(), project.clone(), member.id.clone())
                        .await
                        .unwrap_or(false)
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// `isFlaggableHandlerType`. The parameter type at position 0 already
    /// unwraps a rest array or tuple to its first element.
    async fn flaggable_handler(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        for part in self.union_parts(ty).await? {
            for signature in self.signatures(&part, CALL).await? {
                if signature.parameters.is_empty() {
                    continue;
                }
                let Some(first) = self.parameter_type(&signature, 0).await? else {
                    continue;
                };
                if first.flags != UNKNOWN {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// `isAlwaysNonAwaitableType`.
    async fn always_non_awaitable(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        for part in self.union_parts(ty).await? {
            if self.awaitable(&part).await? != Awaitable::Never {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `containsNonAwaitableType`.
    async fn contains_non_awaitable(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        for part in self.union_parts(ty).await? {
            if self.awaitable(&part).await? == Awaitable::Never {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `isInvalidPromiseAggregatorInput`.
    async fn invalid_aggregator_input(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let parts = self.union_parts(ty).await?;
        // Non-iterables are already a type error.
        for part in &parts {
            if !self.has_symbol_member(part, "iterator").await? {
                return Ok(false);
            }
        }
        for part in &parts {
            for value in self.array_like_values(part).await? {
                if self.contains_non_awaitable(&value).await? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// `hasMaybeAwaitableValue`.
    async fn maybe_awaitable_value(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        for part in self.union_parts(ty).await? {
            for value in self.array_like_values(&part).await? {
                if !self.always_non_awaitable(&value).await? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// `getValueTypesOfArrayLike`: tuple elements, the number index type of an
    /// array-like, or the first type argument of another reference.
    async fn array_like_values(&mut self, ty: &TypeResponse) -> corsa::Result<Vec<TypeResponse>> {
        let client = self.session.client();
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        if self.is_tuple(ty).await? {
            return client
                .get_type_arguments(snapshot, project, ty.id.clone())
                .await;
        }
        if client
            .is_array_like_type(snapshot.clone(), project.clone(), ty.id.clone())
            .await
            .unwrap_or(false)
        {
            let infos = client
                .get_index_infos_of_type(snapshot, project, ty.id.clone())
                .await
                .unwrap_or_default();
            return Ok(infos
                .into_iter()
                .find(|info| info.key_type.flags & NUMBER != 0)
                .map(|info| info.value_type)
                .into_iter()
                .collect());
        }
        if ty.object_flags.unwrap_or_default() & REFERENCE != 0 {
            let arguments = client
                .get_type_arguments(snapshot, project, ty.id.clone())
                .await?;
            return Ok(arguments.into_iter().take(1).collect());
        }
        Ok(Vec::new())
    }

    async fn some_part_has_symbol(
        &mut self,
        ty: &TypeResponse,
        symbol: &str,
    ) -> corsa::Result<bool> {
        for part in self.union_parts(ty).await? {
            if self.has_symbol_member(&part, symbol).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `getWellKnownSymbolPropertyOfType`: the checker names such members
    /// `__@<symbol>@<id>`.
    async fn has_symbol_member(&mut self, ty: &TypeResponse, symbol: &str) -> corsa::Result<bool> {
        if !self.members.contains_key(&ty.id) {
            let names = self
                .session
                .get_properties_of_type(ty.id.clone())
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|property| CompactString::from(property.name.as_str()))
                .collect::<Vec<_>>();
            self.members.insert(ty.id.clone(), names);
        }
        Ok(self.members.get(&ty.id).is_some_and(|names| {
            names.iter().any(|name| {
                name.strip_prefix("__@")
                    .and_then(|rest| rest.strip_prefix(symbol))
                    .is_some_and(|rest| rest.starts_with('@'))
            })
        }))
    }

    /// `needsToBeAwaited`.
    async fn awaitable(&mut self, ty: &TypeResponse) -> corsa::Result<Awaitable> {
        let constrained;
        let ty = if ty.flags & TYPE_PARAMETER != 0 {
            constrained = self
                .session
                .client()
                .get_base_constraint_of_type(
                    self.session.snapshot().handle.clone(),
                    self.session.project_handle(),
                    ty.id.clone(),
                )
                .await?;
            match constrained.as_ref() {
                Some(constraint) => constraint,
                None => return Ok(Awaitable::May),
            }
        } else {
            ty
        };
        if ty.flags & (ANY | UNKNOWN | TYPE_PARAMETER) != 0 {
            return Ok(Awaitable::May);
        }
        if self.is_thenable(ty, 0).await? {
            return Ok(Awaitable::Always);
        }
        Ok(Awaitable::Never)
    }

    /// `tsutils.isThenableType`.
    async fn is_thenable(&mut self, ty: &TypeResponse, depth: u32) -> corsa::Result<bool> {
        if let Some(known) = self.thenable.get(&ty.id) {
            return Ok(*known);
        }
        if depth > DEPTH || ty.flags & (ANY | UNKNOWN | NEVER) != 0 {
            return Ok(false);
        }
        // Seed the cache so a recursive `then` signature terminates.
        self.thenable.insert(ty.id.clone(), false);
        let apparent = self.apparent(ty).await?;
        let mut found = false;
        'parts: for part in self.union_parts(&apparent).await? {
            for signature in self.then_signatures(&part).await? {
                if signature.parameters.is_empty() {
                    continue;
                }
                let rest =
                    signature.flags & HAS_REST_PARAMETER != 0 && signature.parameters.len() == 1;
                if self.is_callback(&signature, rest).await? {
                    found = true;
                    break 'parts;
                }
            }
        }
        self.thenable.insert(ty.id.clone(), found);
        Ok(found)
    }

    /// `isAlwaysThenable` from no-misused-promises.
    async fn always_thenable(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let apparent = self.apparent(ty).await?;
        let parts = self.union_parts(&apparent).await?;
        if parts.is_empty() {
            return Ok(false);
        }
        for part in parts {
            if !self.has_then(&part).await? {
                return Ok(false);
            }
            let mut thenable = false;
            for signature in self.then_signatures(&part).await? {
                if !signature.parameters.is_empty() && self.is_callback(&signature, false).await? {
                    thenable = true;
                    break;
                }
            }
            if !thenable {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `returnsThenable` from no-misused-promises.
    async fn returns_thenable(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let apparent = self.apparent(ty).await?;
        for part in self.union_parts(&apparent).await? {
            for signature in self.signatures(&part, CALL).await? {
                if let Some(returned) = self.return_type(&signature).await?
                    && self.is_thenable(&returned, 0).await?
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// no-floating-promises `isPromiseLike` with the default options.
    async fn promise_like(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let apparent = self.apparent(ty).await?;
        for part in self.union_parts(&apparent).await? {
            if self.builtin(&part, &["Promise"], 0).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// no-floating-promises `isPromiseArray`.
    async fn promise_array(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let client = self.session.client();
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        for part in self.union_parts(ty).await? {
            let part = self.apparent(&part).await?;
            if client
                .is_array_type(snapshot.clone(), project.clone(), part.id.clone())
                .await?
            {
                let arguments = client
                    .get_type_arguments(snapshot.clone(), project.clone(), part.id.clone())
                    .await?;
                if let Some(element) = arguments.first()
                    && self.promise_like(element).await?
                {
                    return Ok(true);
                }
            }
            if self.is_tuple(&part).await? {
                let elements = client
                    .get_type_arguments(snapshot.clone(), project.clone(), part.id.clone())
                    .await?;
                for element in &elements {
                    if self.promise_like(element).await? {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// `isBuiltinSymbolLike`.
    fn builtin<'b>(
        &'b mut self,
        ty: &'b TypeResponse,
        names: &'b [&'b str],
        depth: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = corsa::Result<bool>> + 'b>> {
        Box::pin(async move {
            if depth > DEPTH {
                return Ok(false);
            }
            let client = self.session.client();
            let snapshot = self.session.snapshot().handle.clone();
            let project = self.session.project_handle();
            if ty.flags & INTERSECTION != 0 {
                for part in client
                    .get_types_of_type_in_project(snapshot, project.clone(), ty.id.clone())
                    .await?
                {
                    if self.builtin(&part, names, depth + 1).await? {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            if ty.flags & UNION != 0 {
                let parts = client
                    .get_types_of_type_in_project(snapshot, project.clone(), ty.id.clone())
                    .await?;
                for part in &parts {
                    if !self.builtin(part, names, depth + 1).await? {
                        return Ok(false);
                    }
                }
                return Ok(!parts.is_empty());
            }
            if ty.flags & TYPE_PARAMETER != 0 {
                let constraint = client
                    .get_constraint_of_type(snapshot, project, ty.id.clone())
                    .await?;
                return match constraint {
                    Some(constraint) => self.builtin(&constraint, names, depth + 1).await,
                    None => Ok(false),
                };
            }
            let Some(symbol) = client
                .get_symbol_of_type_in_project(snapshot.clone(), project.clone(), ty.id.clone())
                .await?
            else {
                return Ok(false);
            };
            if names.contains(&symbol.name.as_str()) && from_default_library(&symbol) {
                return Ok(true);
            }
            if symbol.flags & (CLASS | INTERFACE) == 0 {
                return Ok(false);
            }
            let Some(declared) = client
                .get_declared_type_of_symbol(snapshot.clone(), project.clone(), symbol.id.clone())
                .await?
            else {
                return Ok(false);
            };
            for base in client
                .get_base_types(snapshot, project, declared.id)
                .await?
            {
                if self.builtin(&base, names, depth + 1).await? {
                    return Ok(true);
                }
            }
            Ok(false)
        })
    }

    /// `isReadonlyErrorLike`: a lib `Readonly<T>` alias whose argument is Error-like.
    fn readonly_error<'b>(
        &'b mut self,
        ty: &'b TypeResponse,
        depth: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = corsa::Result<bool>> + 'b>> {
        Box::pin(async move {
            if depth > DEPTH {
                return Ok(false);
            }
            let client = self.session.client();
            let snapshot = self.session.snapshot().handle.clone();
            let project = self.session.project_handle();
            if ty.flags & INTERSECTION != 0 {
                for part in client
                    .get_types_of_type_in_project(snapshot, project.clone(), ty.id.clone())
                    .await?
                {
                    if self.readonly_error(&part, depth + 1).await? {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            if ty.flags & UNION != 0 {
                let parts = client
                    .get_types_of_type_in_project(snapshot, project.clone(), ty.id.clone())
                    .await?;
                for part in &parts {
                    if !self.readonly_error(part, depth + 1).await? {
                        return Ok(false);
                    }
                }
                return Ok(!parts.is_empty());
            }
            if ty.flags & TYPE_PARAMETER != 0 {
                let constraint = client
                    .get_constraint_of_type(snapshot, project, ty.id.clone())
                    .await?;
                return match constraint {
                    Some(constraint) => self.readonly_error(&constraint, depth + 1).await,
                    None => Ok(false),
                };
            }
            let alias = client
                .get_alias_symbol_of_type(snapshot.clone(), project.clone(), ty.id.clone())
                .await?;
            let arguments = client
                .get_alias_type_arguments_of_type(snapshot, project, ty.id.clone())
                .await?;
            let (Some(alias), Some(argument)) = (alias, arguments.first()) else {
                return Ok(false);
            };
            if !from_default_library(&alias) || alias.name != "Readonly" {
                return Ok(false);
            }
            Ok(self.builtin(argument, &["Error"], depth + 1).await?
                || self.readonly_error(argument, depth + 1).await?)
        })
    }

    async fn has_then(&mut self, part: &TypeResponse) -> corsa::Result<bool> {
        Ok(self.then_symbol(part).await?.is_some())
    }

    async fn then_symbol(&mut self, part: &TypeResponse) -> corsa::Result<Option<SymbolResponse>> {
        self.session
            .client()
            .get_property_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                part.id.clone(),
                "then",
            )
            .await
    }

    /// Call signatures of every union part of `part.then`.
    async fn then_signatures(
        &mut self,
        part: &TypeResponse,
    ) -> corsa::Result<Vec<SignatureResponse>> {
        let Some(then) = self.then_symbol(part).await? else {
            return Ok(Vec::new());
        };
        let Some(then_type) = self.symbol_type(&then.id).await? else {
            // A synthetic `then` of an intersection has no registered symbol;
            // read the members' own `then` instead.
            if part.flags & INTERSECTION != 0 {
                let members = self
                    .session
                    .client()
                    .get_types_of_type_in_project(
                        self.session.snapshot().handle.clone(),
                        self.session.project_handle(),
                        part.id.clone(),
                    )
                    .await?;
                let mut signatures = Vec::new();
                for member in &members {
                    signatures.extend(Box::pin(self.then_signatures(member)).await?);
                }
                return Ok(signatures);
            }
            return Ok(Vec::new());
        };
        let mut signatures = Vec::new();
        for sub in self.union_parts(&then_type).await? {
            signatures.extend(self.signatures(&sub, CALL).await?);
        }
        Ok(signatures)
    }

    /// `isCallback`: the parameter's apparent type has a callable union part.
    async fn is_callback(
        &mut self,
        signature: &SignatureResponse,
        rest: bool,
    ) -> corsa::Result<bool> {
        let Some(ty) = self.parameter_type(signature, 0).await? else {
            return Ok(false);
        };
        let mut ty = self.apparent(&ty).await?;
        if rest {
            let client = self.session.client();
            let snapshot = self.session.snapshot().handle.clone();
            let project = self.session.project_handle();
            if !client
                .is_array_type(snapshot.clone(), project.clone(), ty.id.clone())
                .await?
            {
                return Ok(false);
            }
            let Some(element) = client
                .get_type_arguments(snapshot, project, ty.id.clone())
                .await?
                .into_iter()
                .next()
            else {
                return Ok(false);
            };
            ty = element;
        }
        for part in self.union_parts(&ty).await? {
            if !self.signatures(&part, CALL).await?.is_empty() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `checker.isTupleType`: a reference to a tuple target.
    async fn is_tuple(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let object_flags = ty.object_flags.unwrap_or_default();
        if object_flags & TUPLE != 0 {
            return Ok(true);
        }
        if object_flags & REFERENCE == 0 {
            return Ok(false);
        }
        let target = self
            .session
            .client()
            .get_target_of_type_in_project(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await?;
        Ok(target.is_some_and(|target| target.object_flags.unwrap_or_default() & TUPLE != 0))
    }

    async fn apparent(&mut self, ty: &TypeResponse) -> corsa::Result<TypeResponse> {
        if let Some(apparent) = self.apparent.get(&ty.id) {
            return Ok(apparent.clone());
        }
        let apparent = self
            .session
            .client()
            .get_apparent_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await?
            .unwrap_or_else(|| ty.clone());
        self.apparent.insert(ty.id.clone(), apparent.clone());
        Ok(apparent)
    }

    /// `tsutils.unionConstituents`.
    async fn union_parts(&mut self, ty: &TypeResponse) -> corsa::Result<Vec<TypeResponse>> {
        if ty.flags & UNION == 0 {
            return Ok(vec![ty.clone()]);
        }
        if let Some(parts) = self.parts.get(&ty.id) {
            return Ok(parts.clone());
        }
        let parts = self
            .session
            .client()
            .get_types_of_type_in_project(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await?;
        self.parts.insert(ty.id.clone(), parts.clone());
        Ok(parts)
    }

    async fn signatures(
        &mut self,
        ty: &TypeResponse,
        kind: i32,
    ) -> corsa::Result<Vec<SignatureResponse>> {
        let key = (ty.id.clone(), kind);
        if let Some(signatures) = self.signatures.get(&key) {
            return Ok(signatures.clone());
        }
        if ty.flags & (ANY | UNKNOWN | NEVER | UNDEFINED | VOID) != 0 {
            self.signatures.insert(key, Vec::new());
            return Ok(Vec::new());
        }
        let signatures = self
            .session
            .get_signatures_of_type(ty.id.clone(), kind)
            .await
            .unwrap_or_default();
        self.signatures.insert(key, signatures.clone());
        Ok(signatures)
    }

    async fn return_type(
        &mut self,
        signature: &SignatureResponse,
    ) -> corsa::Result<Option<TypeResponse>> {
        self.session
            .get_return_type_of_signature(signature.id.clone())
            .await
    }

    /// Type of the parameter at `index`, without touching its (possibly
    /// transient) symbol.
    async fn parameter_type(
        &mut self,
        signature: &SignatureResponse,
        index: usize,
    ) -> corsa::Result<Option<TypeResponse>> {
        let Ok(index) = i32::try_from(index) else {
            return Ok(None);
        };
        Ok(self
            .session
            .client()
            .get_parameter_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                signature.id.clone(),
                index,
            )
            .await
            .unwrap_or_default())
    }

    async fn symbol_type(&mut self, symbol: &SymbolHandle) -> corsa::Result<Option<TypeResponse>> {
        if let Some(ty) = self.symbol_types.get(symbol) {
            return Ok(ty.clone());
        }
        // Transient symbols are not in the snapshot registry.
        let ty = self
            .session
            .get_type_of_symbol(symbol.clone())
            .await
            .unwrap_or_default();
        self.symbol_types.insert(symbol.clone(), ty.clone());
        Ok(ty)
    }
}

/// `isSymbolFromDefaultLibrary`: a declaration lives in a `lib.*.d.ts` file.
fn from_default_library(symbol: &SymbolResponse) -> bool {
    symbol.declarations.iter().any(|declaration| {
        declaration.declaring_path().is_some_and(|path| {
            let name = path.rsplit('/').next().unwrap_or_default();
            name.starts_with("lib.") && name.ends_with(".d.ts")
        })
    })
}
