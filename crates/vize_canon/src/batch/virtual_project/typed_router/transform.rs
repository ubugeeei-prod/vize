//! Authored setup references specialized only through proved projection spans.

use std::ops::Range;

use oxc_ast::ast::{
    Argument, CallExpression, ChainElement, Expression, IdentifierReference,
    ImportDeclarationSpecifier, ImportOrExportKind, ObjectPropertyKind, PropertyKey, PropertyKind,
    Statement, TSTypeQuery, TSTypeQueryExprName,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder, SymbolId};
use oxc_span::{GetSpan, SourceType, Span};
use vize_carton::{String, cstr};

use crate::virtual_ts::{ProjectionMapping, VirtualTsOutput};

/// Sites collected by one parse before generation; edits apply afterwards.
pub(super) struct Plan {
    javascript: bool,
    sites: Vec<Site>,
    /// `defineProps` calls that receive the route-props helper, with their type argument.
    pub(super) props_calls: Vec<(Span, Option<Span>)>,
}

/// `route_props` means the project's own router types admit the experimental
/// component generic and `props: true` helper; otherwise only the file applies.
pub(super) fn plan(
    script: &str,
    source_offset: impl Fn(usize) -> usize,
    setup_range: Range<usize>,
    source_type: SourceType,
    route_props: bool,
) -> Option<Plan> {
    let allocator = oxc_allocator::Allocator::default();
    let javascript = source_type.is_javascript();
    let parsed = Parser::new(&allocator, script, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let built = SemanticBuilder::new().build(&parsed.program);
    if !built.diagnostics.is_empty() {
        return None;
    }
    let mut imports = Vec::new();
    // The plugin reads only setup statements, by name, before any rewrite.
    let mut props_true = false;
    for statement in &parsed.program.body {
        if let Statement::ExpressionStatement(statement) = statement
            && setup_range.contains(&source_offset(statement.span.start as usize))
            && let Some(call) = match &statement.expression {
                Expression::CallExpression(call) => Some(&**call),
                Expression::ChainExpression(chain) => match &chain.expression {
                    ChainElement::CallExpression(call) => Some(&**call),
                    _ => None,
                },
                _ => None,
            }
            && matches!(&call.callee, Expression::Identifier(name) if name.name == "definePage")
            && matches!(props_option(call), Some(Expression::BooleanLiteral(value)) if value.value)
        {
            props_true = true;
        }
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if !matches!(
            import.source.value.as_str(),
            "vue-router" | "vue-router/auto" | "vue-router/experimental"
        ) || import.import_kind == ImportOrExportKind::Type
        {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier
                && specifier.import_kind != ImportOrExportKind::Type
                && specifier.imported.name().as_str() == specifier.local.name.as_str()
                && specifier.imported.span() == specifier.local.span
                && match (import.source.value.as_str(), specifier.local.name.as_str()) {
                    // The published experimental entry exports the page macro;
                    // useRoute still belongs to the stable entry in Router 5.4.
                    ("vue-router/experimental", "definePage") => true,
                    ("vue-router" | "vue-router/auto", "useRoute" | "definePage") => true,
                    _ => false,
                }
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
        route_props: route_props && !javascript,
        props_true: route_props && props_true && !javascript,
        sites: Vec::new(),
        props_calls: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    Some(Plan {
        javascript,
        sites: collector.sites,
        props_calls: collector.props_calls,
    })
}

/// The first identifier-keyed `props` assignment of the macro's object argument.
fn props_option<'a, 'b>(call: &'b CallExpression<'a>) -> Option<&'b Expression<'a>> {
    let Some(Argument::ObjectExpression(object)) = call.arguments.first() else {
        return None;
    };
    object.properties.iter().find_map(|property| match property {
        ObjectPropertyKind::ObjectProperty(property)
            if property.kind == PropertyKind::Init
                && !property.method
                && !property.shorthand
                && !property.computed
                && matches!(&property.key, PropertyKey::StaticIdentifier(key) if key.name == "props") =>
        {
            Some(&property.value)
        }
        _ => None,
    })
}

/// The plugin's `props: true` helper for one file identity.
pub(super) fn route_props_type(file_literal: &str) -> String {
    cstr!("import('vue-router/experimental')._RoutePropsForFilePath<{file_literal}>")
}

/// Returns whether an actual edit introduced a generated route type import.
/// `component` is the self-import literal admitted with the route-props plan.
pub(super) fn apply(
    output: &mut VirtualTsOutput,
    plan: &Plan,
    script: &str,
    source_offset: impl Fn(usize) -> usize,
    setup_range: Range<usize>,
    (file_literal, component): (&str, Option<&str>),
    diagnostic_ownership: Option<
        &crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector,
    >,
) -> bool {
    if serde_json::from_str::<String>(file_literal).is_err()
        || component.is_some_and(|literal| serde_json::from_str::<String>(literal).is_err())
    {
        return false;
    }
    let javascript = plan.javascript;
    let route_type =
        cstr!("<import('vue-router/auto-routes')._RouteNamesForFilePath<{file_literal}>>");
    let mut edits = Vec::new();
    for site in &plan.sites {
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
            Kind::Page => match component.filter(|_| site.component) {
                Some(component) => proposed.push((
                    token.end,
                    cstr!("<{file_literal}, typeof import({component}).default>"),
                )),
                None => proposed.push((token.end, cstr!("<{file_literal}>"))),
            },
            Kind::Route if javascript => {
                proposed.push((start, String::from("(")));
                proposed.push((end, cstr!(" as ReturnType<typeof useRoute{route_type}>)")));
            }
            Kind::Route | Kind::Query => proposed.push((token.end, route_type.clone())),
            Kind::Props if component.is_none() => continue,
            Kind::Props if site.token == site.span => {
                proposed.push((start, cstr!("{} & (", route_props_type(file_literal))));
                proposed.push((end, String::from(")")));
            }
            Kind::Props => {
                proposed.push((token.end, cstr!("<{}>", route_props_type(file_literal))))
            }
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
        if let Some(collector) = diagnostic_ownership {
            collector
                .borrow_mut()
                .note_generated_insertion(at, text.len());
        }
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
    /// A `props: true` page's `defineProps()`; a type argument is its own token.
    Props,
}

struct Site {
    span: Span,
    token: Span,
    kind: Kind,
    /// The complete call/query, or the macro header before its sole argument.
    validation_end: u32,
    /// A page macro whose `props` option is present and not `false`.
    component: bool,
}

struct Collector<'a> {
    scoping: &'a Scoping,
    imports: Vec<SymbolId>,
    javascript: bool,
    route_props: bool,
    props_true: bool,
    sites: Vec<Site>,
    props_calls: Vec<(Span, Option<Span>)>,
}

impl Collector<'_> {
    fn authorized(&self, identifier: &IdentifierReference<'_>) -> bool {
        identifier.reference_id.get().is_some_and(|reference| {
            self.scoping
                .get_reference(reference)
                .symbol_id()
                .map_or_else(
                    || {
                        // Type-only imports are deliberately unresolved in the value
                        // namespace; they must not become auto-imported global macros.
                        self.scoping
                            .get_root_binding(identifier.name.as_str().into())
                            .is_none_or(|symbol| {
                                !self.scoping.symbol_flags(symbol).is_type_import()
                            })
                    },
                    |symbol| self.imports.contains(&symbol),
                )
        })
    }
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.props_true
            && call.arguments.is_empty()
            && let Expression::Identifier(identifier) = &call.callee
            && identifier.name == "defineProps"
        {
            // `props: true` passes route params as props; the plugin stops here.
            let type_argument = call
                .type_arguments
                .as_ref()
                .and_then(|arguments| arguments.params.first())
                .map(GetSpan::span);
            self.props_calls.push((call.span, type_argument));
            let (span, token) =
                type_argument.map_or((call.span, identifier.span), |span| (span, span));
            self.sites.push(Site {
                span,
                token,
                kind: Kind::Props,
                validation_end: span.end,
                component: false,
            });
            return;
        }
        if !call.optional
            && call.type_arguments.is_none()
            && let Expression::Identifier(identifier) = &call.callee
            && matches!(identifier.name.as_str(), "useRoute" | "definePage")
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
                    component: kind == Kind::Page
                        && self.route_props
                        && props_option(call).is_some_and(|props| {
                            !matches!(props, Expression::BooleanLiteral(value) if !value.value)
                        }),
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
                component: false,
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
#[expect(clippy::disallowed_macros, reason = "test fixtures use std strings")]
mod tests;
