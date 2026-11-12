//! Awaitable, thenable and handler classification of one type.
use super::{
    ANY, Awaitable, CALL, DEPTH, HAS_REST_PARAMETER, INSTANTIABLE, INTERSECTION, NEVER, NUMBER,
    PromiseChecker, REFERENCE, TYPE_PARAMETER, UNION, UNKNOWN,
};
use corsa::api::TypeResponse;
use vize_l0::String as CompactString;

impl PromiseChecker<'_> {
    pub(super) async fn classify_argument(
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
    pub(super) async fn constrained(&mut self, ty: &TypeResponse) -> corsa::Result<TypeResponse> {
        if ty.flags & INSTANTIABLE == 0 {
            return Ok(ty.clone());
        }
        Ok(self
            .session
            .base_constraint(ty)
            .await
            .unwrap_or_else(|| ty.clone()))
    }

    /// `hasMatchingPromiseTypeArgument`: the awaited types of the thenable
    /// parts and the other parts are the same set of types. A thenable's
    /// awaited type is read from its first type argument.
    pub(super) async fn matching_promise_union(
        &mut self,
        ty: &TypeResponse,
    ) -> corsa::Result<bool> {
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
    pub(super) async fn awaited(
        &mut self,
        ty: &TypeResponse,
    ) -> corsa::Result<Option<TypeResponse>> {
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

    pub(super) async fn any_equivalent(
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

    pub(super) async fn array_or_tuple(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let client = self.session.client();
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        for part in self.union_parts(ty).await? {
            let members = if part.flags & INTERSECTION != 0 {
                self.session.constituents(&part).await
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
    pub(super) async fn flaggable_handler(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
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
    pub(super) async fn always_non_awaitable(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        for part in self.union_parts(ty).await? {
            if self.awaitable(&part).await? != Awaitable::Never {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `containsNonAwaitableType`.
    pub(super) async fn contains_non_awaitable(
        &mut self,
        ty: &TypeResponse,
    ) -> corsa::Result<bool> {
        for part in self.union_parts(ty).await? {
            if self.awaitable(&part).await? == Awaitable::Never {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `isInvalidPromiseAggregatorInput`.
    pub(super) async fn invalid_aggregator_input(
        &mut self,
        ty: &TypeResponse,
    ) -> corsa::Result<bool> {
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
    pub(super) async fn maybe_awaitable_value(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
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
    pub(super) async fn array_like_values(
        &mut self,
        ty: &TypeResponse,
    ) -> corsa::Result<Vec<TypeResponse>> {
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

    pub(super) async fn some_part_has_symbol(
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
    pub(super) async fn has_symbol_member(
        &mut self,
        ty: &TypeResponse,
        symbol: &str,
    ) -> corsa::Result<bool> {
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
    pub(super) async fn awaitable(&mut self, ty: &TypeResponse) -> corsa::Result<Awaitable> {
        let constrained;
        let ty = if ty.flags & TYPE_PARAMETER != 0 {
            constrained = self.session.base_constraint(ty).await;
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
    pub(super) async fn is_thenable(
        &mut self,
        ty: &TypeResponse,
        depth: u32,
    ) -> corsa::Result<bool> {
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
    pub(super) async fn always_thenable(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
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
    pub(super) async fn returns_thenable(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
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
    pub(super) async fn promise_like(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let apparent = self.apparent(ty).await?;
        for part in self.union_parts(&apparent).await? {
            if self.builtin(&part, &["Promise"], 0).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
