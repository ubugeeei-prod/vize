//! Exact source occurrences for the public-instance loop-source projection.
//!
//! Croquis' historical UndefinedRefs fact relation locates each spelling at its
//! first occurrence. Keep that relation and its independent corpus oracle intact;
//! Canon needs each authored read to own a separate diagnostic and mapping.

use vize_carton::FxHashSet;
use vize_croquis::{Croquis, ScopeData, UndefinedRef, analyzer::extract_identifier_refs_oxc};

pub(super) fn authored_references(
    summary: &Croquis,
    references: Vec<UndefinedRef>,
) -> Vec<UndefinedRef> {
    let mut result = Vec::with_capacity(references.len());
    let mut expanded = FxHashSet::default();
    for reference in &references {
        let source = summary.scopes.iter().find_map(|scope| {
            let ScopeData::VFor(data) = scope.data() else {
                return None;
            };
            let start = summary.scopes.v_for_source_offset(scope.id)?;
            let end = start + data.source.len() as u32;
            (start <= reference.offset && reference.offset < end).then_some((
                scope.id,
                start,
                end,
                data.source.as_str(),
            ))
        });
        let Some((scope, start, end, source)) = source else {
            result.push(reference.clone());
            continue;
        };
        if !expanded.insert(scope) {
            continue;
        }
        let unresolved: FxHashSet<_> = references
            .iter()
            .filter(|item| start <= item.offset && item.offset < end)
            .map(|item| item.name.as_str())
            .collect();
        let reads = extract_identifier_refs_oxc(source);
        // A refused or partial parse must preserve the original diagnostics.
        if unresolved
            .iter()
            .any(|name| !reads.iter().any(|read| read.name.as_str() == *name))
        {
            result.extend(
                references
                    .iter()
                    .filter(|item| start <= item.offset && item.offset < end)
                    .cloned(),
            );
            continue;
        }
        for read in reads {
            if unresolved.contains(read.name.as_str()) {
                result.push(UndefinedRef {
                    name: read.name,
                    offset: start + read.offset,
                    context: reference.context.clone(),
                });
            }
        }
    }
    result
}
