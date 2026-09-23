//! Relocate ambient declarations, evaluating captured types in their authored scope.

mod projection;
pub(in super::super) use projection::AmbientProjection;

use oxc_allocator::Allocator;
use oxc_ast::{AstKind, ast::Statement};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType, Span};
use vize_croquis::{Croquis, ScopeData, ScopeKind};

use super::super::script_module::include_leading_ts_directive_comments;
use crate::virtual_ts::helpers::SETUP_SCOPE_HELPER_NAMES;

pub(super) fn extend_module_spans(
    summary: &Croquis,
    script: Option<&str>,
    module_spans: &mut Vec<(u32, u32)>,
) -> AmbientProjection {
    let Some(script) = script.filter(|source| source.contains("declare")) else {
        return AmbientProjection::default();
    };
    let has_setup = summary
        .scopes
        .iter()
        .any(|scope| matches!(scope.kind, ScopeKind::ScriptSetup));
    if summary.scopes.iter().any(|scope| {
        (has_setup && matches!(scope.kind, ScopeKind::NonScriptSetup))
            || matches!(scope.data(), ScopeData::ScriptSetup(data) if data.generic.is_some())
    }) {
        return AmbientProjection::default();
    }

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    // Recoverable parser errors still leave an authored declaration AST. Keep
    // its lexical ownership so an invalid ambient initializer gets TS1039,
    // without also inventing TS1184 by placing `declare` inside setup().
    if parsed.panicked {
        return AmbientProjection::default();
    }
    let comments: Vec<Span> = parsed
        .program
        .comments
        .iter()
        .map(|comment| comment.span)
        .collect();
    let candidates: Vec<Span> = parsed
        .program
        .body
        .iter()
        .filter_map(|statement| {
            // A module augmentation is already a module statement. It is a
            // candidate only for its captures: `typeof` a setup value inside
            // `declare module 'vue' { … }` must be evaluated in setup scope.
            let (ambient, hoisted) = match statement {
                Statement::VariableDeclaration(declaration) => (declaration.declare, false),
                Statement::FunctionDeclaration(declaration) => (declaration.declare, false),
                Statement::ClassDeclaration(declaration) => (declaration.declare, false),
                Statement::TSModuleDeclaration(declaration) => (declaration.declare, true),
                _ => (false, false),
            };
            let span = statement.span();
            (ambient
                && (hoisted || !covered(module_spans, span))
                && relocatable_line(script, span, &comments))
            .then_some(span)
        })
        .collect();
    if candidates.is_empty() {
        return AmbientProjection::default();
    }
    let capture_allowed: Vec<bool> = candidates
        .iter()
        .map(|span| starts_without_leading_directive(script, *span))
        .collect();

    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program);
    if !built.diagnostics.is_empty() {
        return AmbientProjection::default();
    }
    let semantic = built.semantic;
    let scoping = semantic.scoping();
    let mut dependencies = vec![Vec::new(); candidates.len()];
    let mut projection = AmbientProjection::default();
    let mut captured = vec![Vec::new(); candidates.len()];
    // Both references and declaration merging must follow their resolved symbol.
    // Moving just one side of an ambient/runtime var or overload changes its type.
    for symbol in scoping.symbol_ids() {
        let declarations: Vec<Span> = scoping
            .symbol_declarations(symbol)
            .map(|node| semantic.nodes().get_node(node).kind().span())
            .collect();
        let mut owners: Vec<usize> = declarations
            .iter()
            .filter_map(|declaration| containing(&candidates, *declaration))
            .collect();
        owners.sort_unstable();
        owners.dedup();
        let representative = owners.first().copied();
        // A shared representative avoids a quadratic clique for overload sets.
        let representative_span = representative.and_then(|first| candidates.get(first).copied());
        if let Some((first, first_span)) = representative.zip(representative_span) {
            if let Some(first_dependencies) = dependencies.get_mut(first) {
                first_dependencies.extend(declarations.iter().copied());
            }
            for &owner in owners.iter().skip(1) {
                if let Some(owner_dependencies) = dependencies.get_mut(owner) {
                    owner_dependencies.push(first_span);
                }
            }
        }
        for reference in semantic.symbol_references(symbol) {
            if let Some(index) = containing(&candidates, semantic.reference_span(reference))
                && let Some(reference_dependencies) = dependencies.get_mut(index)
            {
                if let Some(first_span) = representative_span {
                    reference_dependencies.push(first_span);
                } else if scoping.symbol_scope_id(symbol) == scoping.root_scope_id()
                    && declarations
                        .iter()
                        .all(|span| !covered(module_spans, *span))
                    && capture_allowed.get(index).copied().unwrap_or(false)
                    && let Some(captures) = captured.get_mut(index)
                    && let Some((span, value)) = semantic
                        .nodes()
                        .ancestor_kinds(reference.node_id())
                        .find_map(|kind| {
                            let reference = semantic.reference_span(reference);
                            match kind {
                                AstKind::TSTypeQuery(query)
                                    if query.expr_name.span().contains_inclusive(reference) =>
                                {
                                    Some((query.span, false))
                                }
                                AstKind::TSTypeReference(ty)
                                    if ty.type_name.span().contains_inclusive(reference) =>
                                {
                                    Some((ty.span, false))
                                }
                                AstKind::Class(class) => class
                                    .super_class
                                    .as_ref()
                                    .filter(|base| base.span().contains_inclusive(reference))
                                    .map(|base| (base.span(), true)),
                                _ => None,
                            }
                        })
                {
                    captures.push((span, value));
                } else {
                    reference_dependencies.extend(declarations.iter().copied());
                }
            }
        }
    }

    let mut blocked = vec![false; candidates.len()];
    for captures in &mut captured {
        captures.sort_by_key(|(span, _)| (span.start, std::cmp::Reverse(span.end)));
        let mut end = 0;
        captures.retain(|(span, _)| {
            if span.start < end {
                return false;
            }
            end = span.end;
            true
        });
    }
    // A captured type expression must not lose a signature-local type parameter.
    for symbol in scoping.symbol_ids() {
        if scoping.symbol_scope_id(symbol) == scoping.root_scope_id() {
            continue;
        }
        for reference in semantic.symbol_references(symbol) {
            let reference = semantic.reference_span(reference);
            let Some(index) = containing(&candidates, reference) else {
                continue;
            };
            let Some(captures) = captured.get(index) else {
                continue;
            };
            if let Some(&(span, _)) = captures
                .partition_point(|(span, _)| span.start <= reference.start)
                .checked_sub(1)
                .and_then(|capture| captures.get(capture))
                && span.contains_inclusive(reference)
                && !span.contains_inclusive(scoping.symbol_span(symbol))
                && let Some(blocked) = blocked.get_mut(index)
            {
                *blocked = true;
            }
        }
    }
    // An unresolved authored name resolves through the same project globals in
    // either scope, including value queries and computed keys such as
    // `typeof document` and `[Symbol.iterator]`. Only helpers introduced by our
    // setup projection have a different lexical home.
    for reference_id in scoping.root_unresolved_references_ids().flatten() {
        let reference = scoping.get_reference(reference_id);
        if !SETUP_SCOPE_HELPER_NAMES.contains(&semantic.reference_name(reference)) {
            continue;
        }
        if let Some(blocked) = containing(&candidates, semantic.reference_span(reference))
            .and_then(|index| blocked.get_mut(index))
        {
            *blocked = true;
        }
    }
    let mut dependents = vec![Vec::new(); candidates.len()];
    for (index, dependencies) in dependencies.into_iter().enumerate() {
        for dependency in dependencies {
            if covered(module_spans, dependency) {
                continue;
            }
            if let Some(owner) = containing(&candidates, dependency) {
                if owner != index
                    && let Some(owner_dependents) = dependents.get_mut(owner)
                {
                    owner_dependents.push(index);
                }
            } else if let Some(blocked) = blocked.get_mut(index) {
                *blocked = true;
            }
        }
    }
    let mut queue: Vec<usize> = blocked
        .iter()
        .enumerate()
        .filter_map(|(index, blocked)| blocked.then_some(index))
        .collect();
    let mut cursor = 0;
    while let Some(&blocker) = queue.get(cursor) {
        for &dependent in dependents.get(blocker).into_iter().flatten() {
            if let Some(blocked) = blocked.get_mut(dependent)
                && !*blocked
            {
                *blocked = true;
                queue.push(dependent);
            }
        }
        cursor += 1;
    }
    projection.collect(
        &candidates,
        captured,
        &blocked,
        scoping
            .symbol_ids()
            .map(|id| scoping.symbol_name(id))
            .chain(
                scoping
                    .root_unresolved_references_ids()
                    .flatten()
                    .map(|id| semantic.reference_name(scoping.get_reference(id))),
            ),
    );
    let spans = candidates
        .into_iter()
        .zip(blocked)
        .filter_map(|(span, blocked)| (!blocked).then_some((span.start, span.end)))
        .collect();
    module_spans.extend(include_leading_ts_directive_comments(script, spans));
    *module_spans = super::super::spans::merge_overlapping_spans(std::mem::take(module_spans));
    projection
}

fn covered(spans: &[(u32, u32)], span: Span) -> bool {
    spans
        .iter()
        .any(|&(start, end)| start <= span.start && span.end <= end)
}

fn containing(candidates: &[Span], span: Span) -> Option<usize> {
    let index = candidates
        .partition_point(|candidate| candidate.start <= span.start)
        .checked_sub(1)?;
    (span.end <= candidates.get(index)?.end).then_some(index)
}

fn relocatable_line(script: &str, span: Span, comments: &[Span]) -> bool {
    let start = span.start as usize;
    let end = span.end as usize;
    let (Some(before), Some(after)) = (script.get(..start), script.get(end..)) else {
        return false;
    };
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    let line_end = after.find('\n').map_or(script.len(), |index| end + index);
    let leading = before.get(line_start..).unwrap_or_default();
    let trailing = after.get(..line_end - end).unwrap_or_default();
    if leading.trim().is_empty() && trailing.trim().is_empty() {
        return true;
    }
    // Setup emission masks exact spans, so neighboring statements stay put.
    // Shared-line comments and directives must not move with only one statement.
    let first_comment = comments.partition_point(|comment| comment.end as usize <= line_start);
    if comments
        .get(first_comment)
        .is_some_and(|comment| (comment.start as usize) < line_end)
    {
        return false;
    }
    starts_without_leading_directive(script, span)
}

/// Whether no leading `@ts-*` directive comment attaches to `span`.
fn starts_without_leading_directive(script: &str, span: Span) -> bool {
    include_leading_ts_directive_comments(script, vec![(span.start, span.end)])
        .first()
        .is_some_and(|&(start, _)| start == span.start)
}
