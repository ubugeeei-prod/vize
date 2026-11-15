//! Generic defaults for `no-unnecessary-type-arguments`.
//!
//! Local declarations already expose their default spans in the listener tree.
//! Imported and library declarations stay checker-owned: resolve the selected
//! signature or declared type, then ask the checker for the corresponding type
//! parameter default. No source parsing or rendered type text participates in
//! the decision.
use super::checker::Checker;
use super::{DefaultAsk, DefaultReply, Located, Span, flags};
use corsa::api::{
    NodeHandle, SignatureHandle, SignatureResponse, SymbolHandle, SymbolResponse, TypeHandle,
    TypeResponse,
};
use vize_l0::FxHashMap;

const SYMBOL_ALIAS: u32 = 1 << 21;

type SignatureKey = (SignatureHandle, usize);
type TypeKey = (TypeHandle, usize);
type DeclarationKey = (NodeHandle, usize);
type DefaultResolution = Option<(usize, Option<TypeResponse>)>;

#[derive(Default)]
pub(super) struct Cache {
    aliases: FxHashMap<SymbolHandle, Option<SymbolResponse>>,
    signatures: FxHashMap<SignatureKey, DefaultResolution>,
    types: FxHashMap<TypeKey, DefaultResolution>,
    declarations: FxHashMap<DeclarationKey, DefaultResolution>,
    defaults: FxHashMap<TypeHandle, Option<TypeResponse>>,
}

pub(super) async fn check(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &DefaultAsk,
    translate: &dyn Fn(Span) -> Span,
    symbol: Option<SymbolResponse>,
    cache: &mut Cache,
) -> DefaultReply {
    let Some(argument) = located.ty(translate(ask.argument)).cloned() else {
        return DefaultReply::default();
    };
    if let Some(default) = ask.local_default {
        let Some(default) = located.ty(translate(default)).cloned() else {
            return DefaultReply::default();
        };
        return DefaultReply {
            same: if comparable(&argument, &default) {
                Some(checker.same_type(&argument, &default).await)
            } else {
                None
            },
            parameter_count: ask.parameter_count,
            last_has_default: true,
        };
    }

    let resolved = external_default(checker, located, ask, translate, symbol, cache).await;
    let Some((count, default)) = resolved else {
        return DefaultReply::default();
    };
    let last_has_default = default.is_some();
    let same = match default {
        Some(default) if comparable(&argument, &default) => {
            Some(checker.same_type(&argument, &default).await)
        }
        _ => None,
    };
    DefaultReply {
        same,
        parameter_count: Some(count),
        last_has_default,
    }
}

fn comparable(left: &TypeResponse, right: &TypeResponse) -> bool {
    left.flags & flags::ANY == 0 && right.flags & flags::ANY == 0
}

async fn external_default(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &DefaultAsk,
    translate: &dyn Fn(Span) -> Span,
    symbol: Option<SymbolResponse>,
    cache: &mut Cache,
) -> Option<(usize, Option<TypeResponse>)> {
    let mut resolved_without_default = None;
    if let Some(call) = ask.call
        && let Some(call) = located.handle(translate(call))
        && let Some(signature) = resolved_signature(checker, call).await
        && let Some(found) = signature_default(checker, &signature, ask.argument_index, cache).await
    {
        if found.1.is_some() || ask.constructed {
            return Some(found);
        }
        resolved_without_default = Some(found);
    }

    let mut symbol = match symbol {
        Some(symbol) => symbol,
        None => {
            let Some(owner) = located.handle(translate(ask.owner)).cloned() else {
                return resolved_without_default;
            };
            let Some(symbol) = checker.symbols_at(vec![owner]).await.pop().flatten() else {
                return resolved_without_default;
            };
            symbol
        }
    };
    if symbol.flags & SYMBOL_ALIAS != 0 {
        if let Some(known) = cache.aliases.get(&symbol.id) {
            let Some(known) = known.clone() else {
                return resolved_without_default;
            };
            symbol = known;
        } else {
            let id = symbol.id.clone();
            let aliased = checker.aliased_symbol(&symbol).await;
            cache.aliases.insert(id, aliased.clone());
            let Some(aliased) = aliased else {
                return resolved_without_default;
            };
            symbol = aliased;
        }
    }
    symbol_default(checker, &symbol, ask, cache)
        .await
        .or(resolved_without_default)
}

async fn resolved_signature(checker: &Checker<'_>, call: &NodeHandle) -> Option<SignatureResponse> {
    let session = checker.session();
    session
        .client()
        .get_resolved_signature(
            session.snapshot().handle.clone(),
            session.project_handle(),
            call.clone(),
        )
        .await
        .ok()
        .flatten()
}

async fn symbol_default(
    checker: &mut Checker<'_>,
    symbol: &SymbolResponse,
    ask: &DefaultAsk,
    cache: &mut Cache,
) -> Option<(usize, Option<TypeResponse>)> {
    if ask.call.is_some() {
        if let [declaration] = symbol.declarations.as_slice() {
            return declaration_default(checker, declaration, ask.argument_index, cache).await;
        }
        // Some stable runtimes omit the resolved declaration for overloaded
        // library members. Keep the selected-signature result when available;
        // otherwise accept only a checker-confirmed default-bearing overload.
        for declaration in &symbol.declarations {
            if let Some(found @ (_, Some(_))) =
                declaration_default(checker, declaration, ask.argument_index, cache).await
            {
                return Some(found);
            }
        }
        return None;
    }

    // Merged type symbols retain the previous single-declaration boundary.
    if symbol.declarations.len() != 1 || ask.constructed {
        return None;
    }
    symbol_type_default(checker, symbol, ask.argument_index, cache).await
}

async fn declaration_default(
    checker: &mut Checker<'_>,
    declaration: &NodeHandle,
    argument_index: usize,
    cache: &mut Cache,
) -> Option<(usize, Option<TypeResponse>)> {
    let key = (declaration.clone(), argument_index);
    if let Some(found) = cache.declarations.get(&key) {
        return found.clone();
    }
    let session = checker.session();
    let found = match session
        .client()
        .get_signature_from_declaration(
            session.snapshot().handle.clone(),
            session.project_handle(),
            declaration.clone(),
        )
        .await
        .ok()
        .flatten()
    {
        Some(signature) => signature_default(checker, &signature, argument_index, cache).await,
        None => None,
    };
    cache.declarations.insert(key, found.clone());
    found
}

async fn signature_default(
    checker: &mut Checker<'_>,
    signature: &SignatureResponse,
    argument_index: usize,
    cache: &mut Cache,
) -> Option<(usize, Option<TypeResponse>)> {
    let key = (signature.id.clone(), argument_index);
    if let Some(found) = cache.signatures.get(&key) {
        return found.clone();
    }
    let found = match signature_parameters(checker, signature).await {
        Some(parameters) => parameter_default(checker, parameters, argument_index, cache).await,
        None => None,
    };
    cache.signatures.insert(key, found.clone());
    found
}

async fn signature_parameters(
    checker: &Checker<'_>,
    signature: &SignatureResponse,
) -> Option<Vec<TypeHandle>> {
    if !signature.type_parameters.is_empty() {
        return Some(signature.type_parameters.clone());
    }
    let session = checker.session();
    let client = session.client();
    let snapshot = session.snapshot().handle.clone();
    let project = session.project_handle();
    let current = client
        .get_type_parameters_of_signature(snapshot.clone(), project.clone(), signature.id.clone())
        .await
        .ok();
    if let Some(parameters) = current.as_ref().filter(|parameters| !parameters.is_empty()) {
        return Some(
            parameters
                .iter()
                .map(|parameter| parameter.id.clone())
                .collect(),
        );
    }

    let target = if let Some(target) = signature.target.clone() {
        Some((target, Vec::new()))
    } else {
        client
            .get_target_of_signature(snapshot.clone(), project.clone(), signature.id.clone())
            .await
            .ok()
            .flatten()
            .map(|target| (target.id, target.type_parameters))
    };
    if let Some((target, parameters)) = target {
        if !parameters.is_empty() {
            return Some(parameters);
        }
        if let Ok(parameters) = client
            .get_type_parameters_of_signature(snapshot, project, target)
            .await
        {
            return Some(
                parameters
                    .into_iter()
                    .map(|parameter| parameter.id)
                    .collect(),
            );
        }
    }
    current.map(|parameters| {
        parameters
            .into_iter()
            .map(|parameter| parameter.id)
            .collect()
    })
}

async fn symbol_type_default(
    checker: &mut Checker<'_>,
    symbol: &SymbolResponse,
    argument_index: usize,
    cache: &mut Cache,
) -> Option<(usize, Option<TypeResponse>)> {
    let session = checker.session();
    let ty = session
        .client()
        .get_declared_type_of_symbol(
            session.snapshot().handle.clone(),
            session.project_handle(),
            symbol.id.clone(),
        )
        .await
        .ok()
        .flatten()?;
    type_default(checker, &ty, argument_index, cache).await
}

async fn type_default(
    checker: &mut Checker<'_>,
    ty: &TypeResponse,
    argument_index: usize,
    cache: &mut Cache,
) -> Option<(usize, Option<TypeResponse>)> {
    let key = (ty.id.clone(), argument_index);
    if let Some(found) = cache.types.get(&key) {
        return found.clone();
    }
    let parameters = if ty.type_parameters.is_empty() {
        let session = checker.session();
        session
            .client()
            .get_type_parameters_of_type_in_project(
                session.snapshot().handle.clone(),
                session.project_handle(),
                ty.id.clone(),
            )
            .await
            .ok()
            .map(|parameters| {
                parameters
                    .into_iter()
                    .map(|parameter| parameter.id)
                    .collect()
            })
    } else {
        Some(ty.type_parameters.clone())
    };
    let found = match parameters {
        Some(parameters) => parameter_default(checker, parameters, argument_index, cache).await,
        None => None,
    };
    cache.types.insert(key, found.clone());
    found
}

async fn parameter_default(
    checker: &Checker<'_>,
    parameters: Vec<TypeHandle>,
    argument_index: usize,
    cache: &mut Cache,
) -> Option<(usize, Option<TypeResponse>)> {
    let count = parameters.len();
    let Some(parameter) = parameters.get(argument_index) else {
        return Some((count, None));
    };
    if let Some(default) = cache.defaults.get(parameter) {
        return Some((count, default.clone()));
    }
    let session = checker.session();
    let default = session
        .client()
        .get_default_from_type_parameter(
            session.snapshot().handle.clone(),
            session.project_handle(),
            parameter.clone(),
        )
        .await
        .ok()
        .flatten();
    cache.defaults.insert(parameter.clone(), default.clone());
    Some((count, default))
}
