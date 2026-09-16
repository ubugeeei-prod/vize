//! Relocate only ambient declarations whose complete scope survives at module level.

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType, Span};
use vize_croquis::{Croquis, ScopeData, ScopeKind};

use super::super::script_module::include_leading_ts_directive_comments;

pub(super) fn extend_module_spans(
    summary: &Croquis,
    script: Option<&str>,
    module_spans: &mut Vec<(u32, u32)>,
) {
    let Some(script) = script.filter(|source| source.contains("declare")) else {
        return;
    };
    if summary.scopes.iter().any(|scope| {
        matches!(scope.kind, ScopeKind::NonScriptSetup)
            || matches!(scope.data(), ScopeData::ScriptSetup(data) if data.generic.is_some())
    }) || !summary
        .scopes
        .iter()
        .any(|scope| matches!(scope.kind, ScopeKind::ScriptSetup))
    {
        return;
    }

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    let candidates: Vec<Span> = parsed
        .program
        .body
        .iter()
        .filter_map(|statement| {
            let ambient = match statement {
                Statement::VariableDeclaration(declaration) => declaration.declare,
                Statement::FunctionDeclaration(declaration) => declaration.declare,
                Statement::ClassDeclaration(declaration) => declaration.declare,
                _ => false,
            };
            let span = statement.span();
            (ambient && !covered(module_spans, span) && isolated_line(script, span)).then_some(span)
        })
        .collect();
    if candidates.is_empty() {
        return;
    }

    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program);
    if !built.diagnostics.is_empty() {
        return;
    }
    let semantic = built.semantic;
    let scoping = semantic.scoping();
    let mut dependencies = vec![Vec::new(); candidates.len()];
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
        if let Some(first) = representative {
            dependencies[first].extend(declarations.iter().copied());
            for &owner in &owners[1..] {
                dependencies[owner].push(candidates[first]);
            }
        }
        for reference in semantic.symbol_references(symbol) {
            if let Some(index) = containing(&candidates, semantic.reference_span(reference)) {
                if let Some(first) = representative {
                    dependencies[index].push(candidates[first]);
                } else {
                    dependencies[index].extend(declarations.iter().copied());
                }
            }
        }
    }

    let mut blocked = vec![false; candidates.len()];
    // Unresolved authored names may be supplied by setup-only macro/auto-import
    // helpers. Until their scope is known, keep the declaration where it was.
    for reference_id in scoping.root_unresolved_references_ids().flatten() {
        let reference = scoping.get_reference(reference_id);
        if let Some(index) = containing(&candidates, semantic.reference_span(reference)) {
            blocked[index] = true;
        }
    }
    let mut dependents = vec![Vec::new(); candidates.len()];
    for (index, dependencies) in dependencies.into_iter().enumerate() {
        for dependency in dependencies {
            if covered(module_spans, dependency) {
                continue;
            }
            if let Some(owner) = containing(&candidates, dependency) {
                if owner != index {
                    dependents[owner].push(index);
                }
            } else {
                blocked[index] = true;
            }
        }
    }
    let mut queue: Vec<usize> = blocked
        .iter()
        .enumerate()
        .filter_map(|(index, blocked)| blocked.then_some(index))
        .collect();
    let mut cursor = 0;
    while cursor < queue.len() {
        for &dependent in &dependents[queue[cursor]] {
            if !blocked[dependent] {
                blocked[dependent] = true;
                queue.push(dependent);
            }
        }
        cursor += 1;
    }
    let spans = candidates
        .into_iter()
        .zip(blocked)
        .filter_map(|(span, blocked)| (!blocked).then_some((span.start, span.end)))
        .collect();
    module_spans.extend(include_leading_ts_directive_comments(script, spans));
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
    (span.end <= candidates[index].end).then_some(index)
}

fn isolated_line(script: &str, span: Span) -> bool {
    let start = span.start as usize;
    let end = span.end as usize;
    let line_start = script[..start].rfind('\n').map_or(0, |index| index + 1);
    let line_end = script[end..]
        .find('\n')
        .map_or(script.len(), |index| end + index);
    script[line_start..start].trim().is_empty() && script[end..line_end].trim().is_empty()
}
