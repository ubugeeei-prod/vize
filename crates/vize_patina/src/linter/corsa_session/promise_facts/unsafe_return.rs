//! Awaited-`any` facts for `no-unsafe-return`.
use super::{
    ANY, CALL, DEPTH, HAS_REST_PARAMETER, INTERSECTION, PromiseChecker, UNION, from_default_library,
};
use corsa::api::{TypeHandle, TypeResponse};
use vize_l0::FxHashSet;

impl PromiseChecker<'_> {
    /// The final intrinsic `any` awaited from a thenable constituent, matching
    /// typescript-eslint's `discriminateAnyType` walk. The outer expression is
    /// intentionally not classified as `any`: callers use this only for its
    /// `PromiseAny` branch.
    pub(in crate::linter) async fn promise_any_awaited(
        &mut self,
        ty: &TypeResponse,
    ) -> corsa::Result<Option<TypeResponse>> {
        let mut visited = FxHashSet::default();
        self.promise_any_in_type(ty, &mut visited, 0).await
    }

    fn promise_any_in_type<'a>(
        &'a mut self,
        ty: &'a TypeResponse,
        visited: &'a mut FxHashSet<TypeHandle>,
        depth: u32,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = corsa::Result<Option<TypeResponse>>> + 'a>,
    > {
        Box::pin(async move {
            if depth > DEPTH || !visited.insert(ty.id.clone()) {
                return Ok(None);
            }
            for part in self.type_constituents(ty).await {
                if !self.is_thenable(&part, depth).await?
                    || self.is_builtin_awaited_alias(&part).await?
                {
                    continue;
                }
                for value in self.then_value_types(&part).await? {
                    if let Some(any) = self.awaited_any(&value, visited, depth + 1).await? {
                        return Ok(Some(any));
                    }
                }
            }
            Ok(None)
        })
    }

    fn awaited_any<'a>(
        &'a mut self,
        ty: &'a TypeResponse,
        visited: &'a mut FxHashSet<TypeHandle>,
        depth: u32,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = corsa::Result<Option<TypeResponse>>> + 'a>,
    > {
        Box::pin(async move {
            if ty.flags & ANY != 0 {
                return Ok(Some(ty.clone()));
            }
            if depth > DEPTH || !visited.insert(ty.id.clone()) {
                return Ok(None);
            }
            for part in self.type_constituents(ty).await {
                if !self.is_thenable(&part, depth).await?
                    || self.is_builtin_awaited_alias(&part).await?
                {
                    continue;
                }
                for value in self.then_value_types(&part).await? {
                    if let Some(any) = self.awaited_any(&value, visited, depth + 1).await? {
                        return Ok(Some(any));
                    }
                }
            }
            Ok(None)
        })
    }

    async fn type_constituents(&mut self, ty: &TypeResponse) -> Vec<TypeResponse> {
        if ty.flags & (UNION | INTERSECTION) == 0 {
            vec![ty.clone()]
        } else {
            self.session.constituents(ty).await
        }
    }

    async fn is_builtin_awaited_alias(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
        let alias = self
            .session
            .client()
            .get_alias_symbol_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await?;
        Ok(alias.is_some_and(|alias| alias.name == "Awaited" && from_default_library(&alias)))
    }

    /// Fulfilment-value types from the callback accepted by a thenable's
    /// `then` signatures. This handles inherited and structural thenables,
    /// where a reference's first generic argument is not the awaited value.
    async fn then_value_types(&mut self, ty: &TypeResponse) -> corsa::Result<Vec<TypeResponse>> {
        let mut values = Vec::new();
        for signature in self.then_signatures(ty).await? {
            let Some(mut callback) = self.parameter_type(&signature, 0).await? else {
                continue;
            };
            if signature.flags & HAS_REST_PARAMETER != 0 && signature.parameters.len() == 1 {
                let client = self.session.client();
                let snapshot = self.session.snapshot().handle.clone();
                let project = self.session.project_handle();
                if !client
                    .is_array_type(snapshot.clone(), project.clone(), callback.id.clone())
                    .await?
                {
                    continue;
                }
                let Some(element) = client
                    .get_type_arguments(snapshot, project, callback.id.clone())
                    .await?
                    .into_iter()
                    .next()
                else {
                    continue;
                };
                callback = element;
            }
            let callback = self.apparent(&callback).await?;
            for part in self.type_constituents(&callback).await {
                for callback_signature in self.signatures(&part, CALL).await? {
                    if let Some(value) = self.parameter_type(&callback_signature, 0).await? {
                        values.push(value);
                    }
                }
            }
        }
        Ok(values)
    }
}
