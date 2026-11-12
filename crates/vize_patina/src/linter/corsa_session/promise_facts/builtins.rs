//! Promise arrays, built-in symbol walks, `then` signatures and the memoized checker lookups behind them.
use super::{
    ANY, CALL, CLASS, DEPTH, INTERFACE, INTERSECTION, NEVER, PromiseChecker, REFERENCE, TUPLE,
    TYPE_PARAMETER, UNDEFINED, UNION, UNKNOWN, VOID, from_default_library,
};
use corsa::api::{SignatureResponse, SymbolHandle, SymbolResponse, TypeResponse};

impl PromiseChecker<'_> {
    /// no-floating-promises `isPromiseArray`.
    pub(super) async fn promise_array(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
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
    pub(super) fn builtin<'b>(
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
                for part in self.session.constituents(ty).await {
                    if self.builtin(&part, names, depth + 1).await? {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            if ty.flags & UNION != 0 {
                let parts = self.session.constituents(ty).await;
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
    pub(super) fn readonly_error<'b>(
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
                for part in self.session.constituents(ty).await {
                    if self.readonly_error(&part, depth + 1).await? {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            if ty.flags & UNION != 0 {
                let parts = self.session.constituents(ty).await;
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

    pub(super) async fn has_then(&mut self, part: &TypeResponse) -> corsa::Result<bool> {
        Ok(self.then_symbol(part).await?.is_some())
    }

    pub(super) async fn then_symbol(
        &mut self,
        part: &TypeResponse,
    ) -> corsa::Result<Option<SymbolResponse>> {
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
    pub(super) async fn then_signatures(
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
                let members = self.session.constituents(part).await;
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
    pub(super) async fn is_callback(
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
    pub(super) async fn is_tuple(&mut self, ty: &TypeResponse) -> corsa::Result<bool> {
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

    pub(super) async fn apparent(&mut self, ty: &TypeResponse) -> corsa::Result<TypeResponse> {
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
    pub(super) async fn union_parts(
        &mut self,
        ty: &TypeResponse,
    ) -> corsa::Result<Vec<TypeResponse>> {
        if ty.flags & UNION == 0 {
            return Ok(vec![ty.clone()]);
        }
        Ok(self.session.constituents(ty).await)
    }

    pub(super) async fn signatures(
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

    pub(super) async fn return_type(
        &mut self,
        signature: &SignatureResponse,
    ) -> corsa::Result<Option<TypeResponse>> {
        self.session
            .get_return_type_of_signature(signature.id.clone())
            .await
    }

    /// Type of the parameter at `index`, without touching its (possibly
    /// transient) symbol.
    pub(super) async fn parameter_type(
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

    pub(super) async fn symbol_type(
        &mut self,
        symbol: &SymbolHandle,
    ) -> corsa::Result<Option<TypeResponse>> {
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
