//! Authored setup references specialized only through proved projection spans.

use std::ops::Range;

use oxc_ast::ast::{
    CallExpression, Expression, IdentifierReference, ImportDeclarationSpecifier,
    ImportOrExportKind, Statement, TSTypeQuery, TSTypeQueryExprName,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder, SymbolId};
use oxc_span::{GetSpan, SourceType, Span};
use vize_carton::{String, cstr};

use crate::virtual_ts::{ProjectionMapping, VirtualTsOutput};

/// Returns whether an actual edit introduced the generated route-name import.
pub(super) fn apply(
    output: &mut VirtualTsOutput,
    script: &str,
    source_offset: impl Fn(usize) -> usize,
    setup_range: Range<usize>,
    source_type: SourceType,
    file_literal: &str,
) -> bool {
    if serde_json::from_str::<String>(file_literal).is_err() {
        return false;
    }
    let allocator = oxc_allocator::Allocator::default();
    let javascript = source_type.is_javascript();
    let parsed = Parser::new(&allocator, script, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return false;
    }
    let built = SemanticBuilder::new().build(&parsed.program);
    if !built.diagnostics.is_empty() {
        return false;
    }
    let mut imports = Vec::new();
    for statement in &parsed.program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if !matches!(
            import.source.value.as_str(),
            "vue-router" | "vue-router/auto"
        ) || import.import_kind == ImportOrExportKind::Type
        {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier
                && specifier.import_kind != ImportOrExportKind::Type
                && specifier.imported.name().as_str() == specifier.local.name.as_str()
                && specifier.imported.span() == specifier.local.span
                && matches!(specifier.local.name.as_str(), "useRoute" | "definePage")
                && let Some(symbol) = specifier.local.symbol_id.get()
            {
                imports.push(symbol);
            }
        }
    }
    let mut collector = Collector {
        scoping: built.semantic.scoping(),
        imports,
        javascript,
        sites: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    let route_type =
        cstr!("<import('vue-router/auto-routes')._RouteNamesForFilePath<{file_literal}>>");
    let mut edits = Vec::new();
    for site in collector.sites {
        let source = site.span.start as usize..site.span.end as usize;
        if site.validation_end < site.token.end
            || site.validation_end > site.span.end
            || (site.kind == Kind::Page && site.validation_end <= site.token.end)
        {
            continue;
        }
        let physical = source_offset(source.start);
        if !setup_range.contains(&physical)
            || physical
                .checked_add(source.len())
                .is_none_or(|end| end > setup_range.end)
            || source
                .clone()
                .any(|offset| source_offset(offset) != physical + (offset - source.start))
        {
            continue;
        }
        let token_source = source_offset(site.token.start as usize);
        let Some(token) = mapped_token(&output.mapping, token_source, site.token) else {
            continue;
        };
        let Some(start) = token
            .start
            .checked_sub((site.token.start - site.span.start) as usize)
        else {
            continue;
        };
        // Filename-only macro edits preserve the generated argument body. Setup
        // emission indents each body line, so validate the exact call header
        // through its opening parenthesis instead of requiring flat body bytes.
        let Some(authored) = script.get(source.start..site.validation_end as usize) else {
            continue;
        };
        let Some(end) = start.checked_add(authored.len()) else {
            continue;
        };
        if output.code.get(start..end) != Some(authored) {
            continue;
        }
        let mut proposed = Vec::new();
        match site.kind {
            Kind::Page if javascript => continue,
            Kind::Page => proposed.push((token.end, cstr!("<{file_literal}>"))),
            Kind::Route if javascript => {
                proposed.push((start, String::from("(")));
                proposed.push((end, cstr!(" as ReturnType<typeof useRoute{route_type}>)")));
            }
            Kind::Route | Kind::Query => proposed.push((token.end, route_type.clone())),
        }
        if proposed
            .iter()
            .all(|(at, _)| safe_insertion(&output.mapping, *at))
        {
            edits.extend(
                proposed
                    .into_iter()
                    .map(|(at, text)| (at, text, site.kind != Kind::Page)),
            );
        }
    }
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
    edits.dedup();
    let introduced_import = edits.iter().any(|(_, _, route)| *route);
    for (at, text, _) in edits {
        output.mapping.note_generated_replacement(at, 0, text.len());
        output.code.insert_str(at, &text);
    }
    introduced_import
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Route,
    Query,
    Page,
}

struct Site {
    span: Span,
    token: Span,
    kind: Kind,
    /// The complete call/query, or the macro header before its sole argument.
    validation_end: u32,
}

struct Collector<'a> {
    scoping: &'a Scoping,
    imports: Vec<SymbolId>,
    javascript: bool,
    sites: Vec<Site>,
}

impl Collector<'_> {
    fn authorized(&self, identifier: &IdentifierReference<'_>) -> bool {
        identifier.reference_id.get().is_some_and(|reference| {
            self.scoping
                .get_reference(reference)
                .symbol_id()
                .is_none_or(|symbol| self.imports.contains(&symbol))
        })
    }
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if !call.optional
            && call.type_arguments.is_none()
            && let Expression::Identifier(identifier) = &call.callee
            && self.authorized(identifier)
        {
            let kind = match identifier.name.as_str() {
                "useRoute" if call.arguments.is_empty() => Some(Kind::Route),
                "definePage" if call.arguments.len() == 1 => Some(Kind::Page),
                _ => None,
            };
            if let Some(kind) = kind {
                self.sites.push(Site {
                    span: call.span,
                    token: identifier.span,
                    kind,
                    validation_end: if kind == Kind::Page {
                        call.arguments
                            .first()
                            .map_or(call.span.end, |argument| argument.span().start)
                    } else {
                        call.span.end
                    },
                });
                // The provider stops at this macro; its argument keeps authored route types.
                if kind == Kind::Page && !self.javascript {
                    return;
                }
            }
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_ts_type_query(&mut self, query: &TSTypeQuery<'a>) {
        if !self.javascript
            && query.type_arguments.is_none()
            && let TSTypeQueryExprName::IdentifierReference(identifier) = &query.expr_name
            && identifier.name == "useRoute"
            && self.authorized(identifier)
        {
            self.sites.push(Site {
                span: query.span,
                token: identifier.span,
                kind: Kind::Query,
                validation_end: query.span.end,
            });
        }
        walk::walk_ts_type_query(self, query);
    }
}

fn mapped_token(mapping: &ProjectionMapping, authored: usize, token: Span) -> Option<Range<usize>> {
    let start = authored.checked_sub(mapping.authored_base())?;
    let source = start..start.checked_add((token.end - token.start) as usize)?;
    let mut candidates = Vec::new();
    for row in mapping.spans() {
        if row.sub_spans.is_empty()
            && row.gen_range.len() == row.src_range.len()
            && source.start >= row.src_range.start
            && source.end <= row.src_range.end
        {
            let at = row
                .gen_range
                .start
                .checked_add(source.start - row.src_range.start)?;
            candidates.push(at..at.checked_add(source.len())?);
        }
        for sub in &row.sub_spans {
            if sub.src_range == source && sub.gen_range.len() == source.len() {
                candidates.push(sub.gen_range.clone());
            }
        }
    }
    candidates.sort_by_key(|range| range.start);
    candidates.dedup();
    if candidates.len() == 1 {
        candidates.first().cloned()
    } else {
        None
    }
}

fn safe_insertion(mapping: &ProjectionMapping, at: usize) -> bool {
    mapping.spans().iter().all(|row| {
        if row.gen_range.start < at && at < row.gen_range.end {
            row.sub_spans.is_empty() && row.gen_range.len() == row.src_range.len()
        } else {
            row.sub_spans
                .iter()
                .all(|sub| !(sub.gen_range.start < at && at < sub.gen_range.end))
        }
    })
}

#[cfg(test)]
#[path = "tests.rs"]
#[expect(clippy::disallowed_macros, reason = "test fixtures use std strings")]
mod tests;
