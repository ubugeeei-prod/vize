//! The type batch behind the fact pass, and the per-query lookups (signatures, promise facts, symbols) answered off it.
use super::super::boolean::checker_node_locations;
use super::super::checker_memo::CheckerMemo;
use super::super::promise_facts::PromiseChecker;
use super::{ExpressionFact, ExpressionQuery};
use corsa::api::{NodeHandle, ProjectSession, TypeResponse};
use vize_l0::String as CompactString;

type Responses = (Vec<Option<TypeResponse>>, Option<Vec<Option<NodeHandle>>>);

/// Types for `ranges`, with the checker node handles when the source file's
/// syntax tree could be read.
pub(super) async fn type_responses(
    session: &ProjectSession,
    file: &str,
    source: &str,
    ranges: &[(u32, u32)],
    widen: &[bool],
) -> corsa::Result<Responses> {
    let usable = ranges.iter().all(|&(start, end)| start < end);
    if usable
        && let Some((responses, handles)) = node_types(session, file, source, ranges, widen).await?
    {
        return Ok((responses, Some(handles)));
    }
    let positions = ranges.iter().map(|&(start, _)| start).collect();
    let responses = session
        .client()
        .get_types_at_positions(
            session.snapshot().handle.clone(),
            session.project_handle(),
            file,
            positions,
        )
        .await?;
    Ok((responses, None))
}

pub(super) async fn node_types(
    session: &ProjectSession,
    file: &str,
    source: &str,
    ranges: &[(u32, u32)],
    widen: &[bool],
) -> corsa::Result<Option<(Vec<Option<TypeResponse>>, Vec<Option<NodeHandle>>)>> {
    let encoded = session
        .client()
        .get_source_file(
            session.snapshot().handle.clone(),
            session.project_handle(),
            file,
        )
        .await?;
    let Some(encoded) = encoded else {
        return Ok(None);
    };
    // Only an unreadable tree falls back to caret types. A range with no node
    // stays untyped on its own: a caret type describes the token at the start
    // (`isListening` for `isListening.value`), so falling back for the whole
    // batch would mistype every other expression in the script.
    let Ok(each) = checker_node_locations(encoded.as_bytes(), source, ranges, widen) else {
        return Ok(None);
    };
    let handles = each.into_iter().map(Result::ok).collect::<Vec<_>>();
    let located = handles.iter().flatten().cloned().collect::<Vec<_>>();
    let mut types = if located.is_empty() {
        Vec::new()
    } else {
        session
            .client()
            .get_type_at_locations(
                session.snapshot().handle.clone(),
                session.project_handle(),
                located,
            )
            .await?
    }
    .into_iter();
    let responses = handles
        .iter()
        .map(|handle| handle.as_ref().and_then(|_| types.next().flatten()))
        .collect();
    Ok(Some((responses, handles)))
}

/// Parameter type texts of the first call signature, once per type.
pub(super) async fn parameter_types(
    memo: &CheckerMemo<'_>,
    response: &TypeResponse,
) -> Vec<Vec<CompactString>> {
    memo.get_signatures_of_type(response.id.clone(), 0)
        .await
        .unwrap_or_default()
        .into_iter()
        .next()
        .map(|signature| {
            signature
                .parameter_type_texts
                .into_iter()
                .map(|texts| {
                    texts
                        .into_iter()
                        .map(|text| CompactString::from(text.as_str()))
                        .collect()
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The promise-rule facts `query` asks for.
pub(super) async fn promise_fact(
    memo: &CheckerMemo<'_>,
    checker: &mut PromiseChecker<'_>,
    response: &TypeResponse,
    location: Option<&NodeHandle>,
    query: &ExpressionQuery,
    fact: &mut ExpressionFact,
) -> corsa::Result<()> {
    if !query.promise.is_empty() {
        fact.promise = Some(checker.facts(response, query.promise).await?);
    }
    if let Some((construct, count)) = query.void_arguments {
        fact.void_arguments = Some(checker.void_arguments(response, construct, count).await?);
    }
    if let Some(heritage) = &query.heritage {
        fact.heritage_voids = checker
            .heritage_voids(response, heritage.base_types, &heritage.members)
            .await?;
    }
    if !query.contextual_function && query.contextual_methods.is_empty() {
        return Ok(());
    }
    let Some(contextual) = (match location {
        Some(location) => memo.contextual_type(location).await,
        None => None,
    }) else {
        return Ok(());
    };
    if query.contextual_function {
        fact.contextual_void = Some(checker.void_function(&contextual).await?);
        fact.contextual_thenable = Some(checker.thenable_function(&contextual).await?);
    }
    if !query.contextual_methods.is_empty() {
        fact.contextual_members = checker
            .member_void_functions(&contextual, &query.contextual_methods)
            .await?;
    }
    Ok(())
}

/// Whether the symbol at each `symbols` query's start is declared in `file`.
pub(super) async fn apply_symbols(
    session: &ProjectSession,
    file: &str,
    ranges: &[(u32, u32)],
    queries: &[ExpressionQuery],
    facts: &mut [ExpressionFact],
) {
    let asked = queries
        .iter()
        .zip(ranges)
        .enumerate()
        .filter(|(_, (query, _))| query.symbols)
        .map(|(index, (_, &(start, _)))| (index, start))
        .collect::<Vec<_>>();
    let symbols = session
        .client()
        .get_symbols_at_positions(
            session.snapshot().handle.clone(),
            session.project_handle(),
            file,
            asked.iter().map(|&(_, start)| start).collect(),
        )
        .await
        .unwrap_or_default();
    for ((index, _), symbol) in asked.into_iter().zip(symbols) {
        let (Some(symbol), Some(fact)) = (symbol, facts.get_mut(index)) else {
            continue;
        };
        fact.declared_in_file = Some(symbol.declarations.iter().any(|declaration| {
            declaration
                .declaring_path()
                .is_some_and(|path| same_file(&path, file))
        }));
    }
}

/// Declaration handles carry a lower-cased path; the wire file may be a URI.
pub(super) fn same_file(path: &str, file: &str) -> bool {
    let file = file.strip_prefix("file://").unwrap_or(file);
    path.eq_ignore_ascii_case(file)
}
