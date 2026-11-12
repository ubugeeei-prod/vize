//! Promise, thenable and Error facts from the actual checker.
//!
//! These mirror typescript-eslint's type helpers (`isThenableType`,
//! `needsToBeAwaited`, `isBuiltinSymbolLike`, `isVoidReturningFunctionType`,
//! ...) so the promise rules decide on checker semantics, not on rendered
//! type names. Every answer is memoized per type handle.
mod awaitable;
mod builtins;
use super::checker_memo::CheckerMemo;
use corsa::api::{SignatureResponse, SymbolHandle, SymbolResponse, TypeHandle, TypeResponse};
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
    session: &'a CheckerMemo<'a>,
    facts: FxHashMap<(TypeHandle, u16), PromiseFacts>,
    thenable: FxHashMap<TypeHandle, bool>,
    apparent: FxHashMap<TypeHandle, TypeResponse>,
    signatures: FxHashMap<(TypeHandle, i32), Vec<SignatureResponse>>,
    symbol_types: FxHashMap<SymbolHandle, Option<TypeResponse>>,
    members: FxHashMap<TypeHandle, Vec<CompactString>>,
}

impl<'a> PromiseChecker<'a> {
    pub(in crate::linter) fn new(session: &'a CheckerMemo<'a>) -> Self {
        Self {
            session,
            facts: FxHashMap::default(),
            thenable: FxHashMap::default(),
            apparent: FxHashMap::default(),
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
