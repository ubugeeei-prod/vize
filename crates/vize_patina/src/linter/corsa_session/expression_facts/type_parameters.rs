//! typescript-eslint's `countTypeParameterUsage` over the checker (#12947).
//!
//! no-unnecessary-type-parameters reports a type parameter that its owner's
//! signature uses at most once. Written uses are counted from the syntax; the
//! uses an inferred type adds (a return type, a method of a class) need the
//! checker. This walk follows upstream's `visitType`: each time the owner's
//! type reaches one of its own type parameters, that parameter counts once,
//! or twice where the position implies several uses (a type argument of a
//! generic other than a readonly array or tuple, an index signature, an alias
//! argument).
//!
//! The answer is all-or-nothing: a request that fails, a mapped type whose
//! internals the API does not expose, or a walk over its request budget leaves
//! every parameter of the owner uncounted, and the rule stays silent rather
//! than report a parameter whose other uses it could not see.
use super::super::checker_memo::CheckerMemo;
mod walk;
use corsa::api::{NodeHandle, TypeResponse};
use walk::Walk;

/// One owner's type parameters and what to walk, in generated ranges.
#[derive(Clone, Debug, Default)]
pub(in crate::linter) struct TypeParameterUsage {
    /// Each own type parameter's declaration range, in order.
    pub parameters: Vec<(u32, u32)>,
    /// Each own type parameter's default type range, aligned.
    pub defaults: Vec<Option<(u32, u32)>>,
    /// Class members `(start, end, constructor)`; the owner itself is not
    /// walked for a class.
    pub members: Vec<(u32, u32, bool)>,
    /// The owner is a class (`fromClass`: every type argument counts twice).
    pub class: bool,
    /// The owner is a call signature: walk its signature, not a type.
    pub signature: bool,
}

/// Where a root of the walk comes from.
enum Root {
    Type(Box<TypeResponse>),
    Signature(NodeHandle),
}

/// Counts per own type parameter, or `None` when they could not all be seen.
pub(super) async fn count<'r>(
    memo: &CheckerMemo<'_>,
    request: &TypeParameterUsage,
    owner: Option<&TypeResponse>,
    owner_location: Option<&NodeHandle>,
    response_at: impl Fn(&(u32, u32)) -> Option<&'r TypeResponse>,
    location_at: impl Fn(&(u32, u32)) -> Option<&'r NodeHandle>,
) -> Option<Vec<u32>> {
    let parameters = request
        .parameters
        .iter()
        .map(|range| response_at(range).map(|ty| ty.id.clone()))
        .collect::<Option<Vec<_>>>()?;
    let defaults = request
        .defaults
        .iter()
        .map(|range| range.as_ref().and_then(&response_at).cloned())
        .collect();
    let mut roots = Vec::new();
    if request.class {
        for range in &request.parameters {
            roots.push(Root::Type(Box::new(response_at(range)?.clone())));
        }
        for &(start, end, constructor) in &request.members {
            let range = (start, end);
            roots.push(if constructor {
                Root::Signature(location_at(&range)?.clone())
            } else {
                match response_at(&range) {
                    Some(ty) => Root::Type(Box::new(ty.clone())),
                    // A member the checker does not type (a static block)
                    // uses nothing.
                    None => continue,
                }
            });
        }
    } else if request.signature {
        roots.push(Root::Signature(owner_location?.clone()));
    } else {
        roots.push(Root::Type(Box::new(owner?.clone())));
    }
    let mut names = Vec::with_capacity(request.parameters.len());
    for range in &request.parameters {
        let texts = memo.type_texts(response_at(range)?).await;
        names.push(texts.into_iter().next()?);
    }
    let mut walk = Walk::new(memo, parameters, names, defaults, request.class);
    let locations = request
        .parameters
        .iter()
        .map(|range| location_at(range).cloned())
        .collect::<Vec<_>>();
    let counted = walk.resolve_own(&locations).await;
    for root in roots {
        walk.reset_root();
        match root {
            Root::Type(ty) => walk.visit(*ty, false, false).await,
            Root::Signature(location) => {
                if !walk.spend() {
                    break;
                }
                match memo
                    .client()
                    .get_signature_from_declaration(
                        memo.snapshot().handle.clone(),
                        memo.project_handle(),
                        location,
                    )
                    .await
                {
                    Ok(Some(signature)) => walk.signature(signature).await,
                    _ => walk.uncertain = true,
                }
            }
        }
        if walk.uncertain {
            break;
        }
    }
    if walk.uncertain {
        return None;
    }
    let mut counts = walk.counts;
    for (count, counted) in counts.iter_mut().zip(counted) {
        if !counted {
            *count = 0;
        }
    }
    Some(counts)
}
