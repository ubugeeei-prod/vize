use super::super::document::TypeAwareDocument;
use super::{
    TemplateContext, TemplatePromiseQuery, TemplateQuery, TemplateQueryKind,
    calls::collect_template_call_ranges, generated_offset_for_expression,
    generated_offset_for_text, slot_spread_callee_offset, v_for_source_binding_offset,
};
use crate::markup::surface_source::{
    DecodedSource, DirectiveArgument, directive, element_verbatim,
};
use oxc_allocator::Allocator as OxcAllocator;
use oxc_span::{SourceType, Span as OxcSpan};
use vize_l0::{CompactString, profile};
use vize_l1::markup::{Attribute, SurfaceChild, SurfaceTree};

pub(super) fn collect_template_query_sets(
    virtual_ts: &TypeAwareDocument,
    template: &SurfaceTree<'_>,
    template_offset: u32,
    include_template_queries: bool,
    include_template_promise_queries: bool,
) -> (Vec<TemplateQuery>, Vec<TemplatePromiseQuery>) {
    // One traversal populates both query vectors. `TemplateQuerySinks` keeps
    // disabled rules out of the hot path, preserves the source-order discovery
    // the sort/dedup stages rely on, and carries the span-slicing source text.
    let mut template_queries = Vec::new();
    let mut template_promise_queries = Vec::new();
    let mut sinks = TemplateQuerySinks {
        template_queries: include_template_queries.then_some(&mut template_queries),
        template_promise_queries: include_template_promise_queries
            .then_some(&mut template_promise_queries),
    };

    // Build the parsing arena and source type once per file; every template
    // expression reuses both (the allocator is reset between expressions).
    let mut allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("template.ts").unwrap_or_default();

    profile!(
        "patina.type_aware.template_query_sets.walk",
        collect_children(
            virtual_ts,
            template,
            &template.children,
            template_offset,
            false,
            &mut allocator,
            source_type,
            &mut sinks,
        )
    );
    if include_template_queries {
        profile!(
            "patina.type_aware.template_queries.dedupe",
            dedupe_template_queries(&mut template_queries)
        );
    }
    if include_template_promise_queries {
        profile!(
            "patina.type_aware.template_promise_queries.dedupe",
            dedupe_template_promise_queries(&mut template_promise_queries)
        );
    }

    (template_queries, template_promise_queries)
}

struct TemplateQuerySinks<'a> {
    template_queries: Option<&'a mut Vec<TemplateQuery>>,
    template_promise_queries: Option<&'a mut Vec<TemplatePromiseQuery>>,
}

impl TemplateQuerySinks<'_> {
    fn has_queries(&self) -> bool {
        self.template_queries.is_some() || self.template_promise_queries.is_some()
    }
}

fn dedupe_template_queries(queries: &mut Vec<TemplateQuery>) {
    queries.sort_unstable_by_key(|query| {
        (
            query.owner_start,
            query.owner_end,
            query.kind,
            query.context,
            query.source_start,
            query.source_end,
        )
    });
    queries.dedup_by(|left, right| {
        left.kind == right.kind
            && left.context == right.context
            && left.source_start == right.source_start
            && left.source_end == right.source_end
            && left.owner_start == right.owner_start
            && left.owner_end == right.owner_end
    });
}

fn dedupe_template_promise_queries(queries: &mut Vec<TemplatePromiseQuery>) {
    queries.sort_unstable_by_key(|query| {
        (
            query.context,
            query.source_start,
            query.source_end,
            query.generated_offset,
        )
    });
    queries.dedup_by(|left, right| {
        left.context == right.context
            && left.source_start == right.source_start
            && left.source_end == right.source_end
    });
}

#[expect(
    clippy::too_many_arguments,
    reason = "explicit per-template walk state"
)]
fn collect_children(
    virtual_ts: &TypeAwareDocument,
    template: &SurfaceTree<'_>,
    children: &[SurfaceChild<'_>],
    template_offset: u32,
    inherited_verbatim: bool,
    allocator: &mut OxcAllocator,
    source_type: SourceType,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    for child in children {
        match child {
            SurfaceChild::Element(element) => {
                let verbatim = element_verbatim(element, inherited_verbatim);
                if !verbatim {
                    for attribute in &element.open.attrs {
                        if sinks.has_queries() {
                            profile!(
                                "patina.type_aware.template_query_sets.directive",
                                collect_directive(
                                    virtual_ts,
                                    template,
                                    attribute,
                                    template_offset,
                                    allocator,
                                    source_type,
                                    sinks,
                                )
                            );
                        }
                    }
                }
                profile!(
                    "patina.type_aware.template_query_sets.children",
                    collect_children(
                        virtual_ts,
                        template,
                        &element.children,
                        template_offset,
                        verbatim,
                        allocator,
                        source_type,
                        sinks,
                    )
                );
            }
            SurfaceChild::Interpolation(interpolation) if !inherited_verbatim => {
                if let Some(expression) =
                    DecodedSource::interpolation(template.source, &interpolation.content)
                        .map(SurfaceExpression::decoded)
                    && sinks.has_queries()
                {
                    collect_expression(
                        virtual_ts,
                        &expression,
                        template_offset,
                        TemplateContext::Interpolation,
                        false,
                        allocator,
                        source_type,
                        sinks,
                    );
                }
            }
            _ => {}
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "explicit per-block state")]
fn collect_expression(
    virtual_ts: &TypeAwareDocument,
    expression: &SurfaceExpression<'_>,
    template_offset: u32,
    context: TemplateContext,
    allow_statement_fallback: bool,
    allocator: &mut OxcAllocator,
    source_type: SourceType,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    let include_template_queries = sinks.template_queries.is_some();
    let include_template_promise_queries = sinks.template_promise_queries.is_some();
    if include_template_queries || include_template_promise_queries {
        profile!(
            "patina.type_aware.template_query_sets.expression",
            collect_expression_query_sets(
                virtual_ts,
                expression,
                template_offset,
                context,
                allow_statement_fallback,
                allocator,
                source_type,
                sinks,
            )
        );
    }
}

fn collect_directive(
    virtual_ts: &TypeAwareDocument,
    template: &SurfaceTree<'_>,
    attribute: &Attribute<'_>,
    template_offset: u32,
    allocator: &mut OxcAllocator,
    source_type: SourceType,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    let Some(directive) = SurfaceDirective::new(template, attribute) else {
        return;
    };
    let context = match directive.name.as_str() {
        "bind" => TemplateContext::Binding,
        "on" => TemplateContext::Event,
        _ => TemplateContext::Directive,
    };
    let Some(expression) = directive.expression else {
        return;
    };
    let first_query = sinks
        .template_queries
        .as_ref()
        .map_or(0, |queries| queries.len());
    collect_expression(
        virtual_ts,
        &expression,
        template_offset,
        context,
        matches!(context, TemplateContext::Event),
        allocator,
        source_type,
        sinks,
    );
    if directive.name == "slot" {
        // A slot parameter declares bindings. Its computed keys can still
        // call unsafe or async values, and descendants still query their uses.
        if let Some(queries) = sinks.template_queries.as_deref_mut()
            && let Some(index) = queries.get(first_query..).and_then(|recent| {
                recent
                    .iter()
                    .position(|query| query.kind == TemplateQueryKind::Expression)
            })
        {
            queries.remove(first_query + index);
        }
    } else if directive.name == "for" {
        retarget_v_for_source_queries(virtual_ts, sinks, first_query);
    } else if directive.name == "bind" && !directive.has_argument {
        retarget_slot_spread_call_queries(virtual_ts, expression.source_text, sinks, first_query);
    }
}

struct SurfaceExpression<'a> {
    source_text: &'a str,
    content: CompactString,
    start: u32,
}

impl<'a> SurfaceExpression<'a> {
    fn new(source: &str, authored: &'a str, content: Option<CompactString>) -> Option<Self> {
        if authored.is_empty() {
            return None;
        }
        Some(Self {
            source_text: authored,
            content: content.unwrap_or_else(|| CompactString::new(authored)),
            start: source_offset(source, authored)?,
        })
    }

    fn decoded(source: DecodedSource<'a>) -> Self {
        Self {
            source_text: source.authored(),
            content: CompactString::new(source.text()),
            start: source.start,
        }
    }
}

struct SurfaceDirective<'a> {
    name: CompactString,
    has_argument: bool,
    expression: Option<SurfaceExpression<'a>>,
}

impl<'a> SurfaceDirective<'a> {
    fn new(template: &SurfaceTree<'a>, attribute: &'a Attribute<'a>) -> Option<Self> {
        let parsed = directive(template, attribute, false)?;
        let name = CompactString::new(parsed.name);
        let expression = if let Some(value) = parsed.value {
            if parsed.name == "for" {
                let (_, iterable) = vize_croquis::drawer::parse_v_for_expression(value.text());
                let iterable = iterable.as_str();
                let authored_value = value.authored();
                let decoded_start = value.text().rfind(iterable)?;
                let decoded_end = decoded_start.checked_add(iterable.len())?;
                let authored = value.authored_span(OxcSpan::new(
                    u32::try_from(decoded_start).ok()?,
                    u32::try_from(decoded_end).ok()?,
                ))?;
                let source_text =
                    authored_value.get(authored.start as usize..authored.end as usize)?;
                Some(SurfaceExpression {
                    source_text,
                    content: CompactString::new(iterable),
                    start: value.start.checked_add(authored.start)?,
                })
            } else {
                Some(SurfaceExpression::decoded(value))
            }
        } else if parsed.name == "bind" {
            match parsed.argument {
                Some(DirectiveArgument::Static(argument)) => SurfaceExpression::new(
                    template.source,
                    argument,
                    Some(vize_l0::camelize(argument)),
                ),
                Some(DirectiveArgument::Dynamic(_)) | None => None,
            }
        } else {
            None
        };
        Some(Self {
            name,
            has_argument: parsed.argument.is_some(),
            expression,
        })
    }
}

fn source_offset(source: &str, slice: &str) -> Option<u32> {
    let source_start = source.as_ptr() as usize;
    let source_end = source_start.checked_add(source.len())?;
    let slice_start = slice.as_ptr() as usize;
    let slice_end = slice_start.checked_add(slice.len())?;
    (slice_start >= source_start && slice_end <= source_end)
        .then(|| u32::try_from(slice_start - source_start).ok())?
}

/// A slot outlet's unargumented `v-bind` is wrapped in a generated spread
/// helper. The closing parenthesis of its authored call can resolve to that
/// helper's `any`, so query the root callee's return type instead.
fn retarget_slot_spread_call_queries(
    virtual_ts: &TypeAwareDocument,
    expression_source: &str,
    sinks: &mut TemplateQuerySinks<'_>,
    first_query: usize,
) {
    if let Some(queries) = sinks.template_queries.as_deref_mut() {
        let Some(recent) = queries.get_mut(first_query..) else {
            return;
        };
        let Some((expression_index, expression)) = recent
            .iter()
            .enumerate()
            .find(|(_, query)| query.kind == TemplateQueryKind::Expression)
        else {
            return;
        };
        let expression_start = expression.source_start;
        let expression_end = expression.source_end;
        let expression_offset = expression.generated_offset;
        let Some((callee_index, callee)) = recent.iter().enumerate().find(|(_, query)| {
            query.kind == TemplateQueryKind::CallCallee
                && query.source_start == expression_start
                && query.source_end < expression_end
        }) else {
            return;
        };
        let callee_len = (callee.source_end - expression_start) as usize;
        let Some(callee_text) = expression_source.get(..callee_len) else {
            return;
        };
        let Some(callee_offset) =
            slot_spread_callee_offset(&virtual_ts.content, expression_offset, callee_text)
        else {
            return;
        };
        if let Some(callee) = recent.get_mut(callee_index) {
            callee.generated_offset = callee_offset;
        }
        if let Some(expression) = recent.get_mut(expression_index) {
            expression.generated_offset = callee_offset;
            expression.kind = TemplateQueryKind::CallReturn;
        }
    }
}

fn retarget_v_for_source_queries(
    virtual_ts: &TypeAwareDocument,
    sinks: &mut TemplateQuerySinks<'_>,
    first_query: usize,
) {
    if let Some(queries) = sinks.template_queries.as_deref_mut() {
        for query in queries.iter_mut().skip(first_query) {
            if query.kind == TemplateQueryKind::Expression
                && let Some(offset) =
                    v_for_source_binding_offset(&virtual_ts.content, query.generated_offset)
            {
                query.generated_offset = offset;
                query.kind = TemplateQueryKind::ForSource;
            }
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "explicit per-block state")]
fn collect_expression_query_sets(
    virtual_ts: &TypeAwareDocument,
    expression: &SurfaceExpression<'_>,
    template_offset: u32,
    context: TemplateContext,
    allow_statement_fallback: bool,
    allocator: &mut OxcAllocator,
    source_type: SourceType,
    sinks: &mut TemplateQuerySinks<'_>,
) {
    let source_start = template_offset + expression.start;
    let Some(source_end) = source_start.checked_add(expression.source_text.len() as u32) else {
        return;
    };
    let source_text = expression.source_text;
    let include_template_queries = sinks.template_queries.is_some();
    let include_template_promise_queries = sinks.template_promise_queries.is_some();
    // Resolve the full expression's generated offset once, then reuse the parsed
    // call-range result for both callee probes and floating-Promise probes. This
    // is the key optimization from the type-aware perf work: template expressions
    // are often short but numerous, so reparsing per rule dominated Corsa time.
    let expression_generated_offset = include_template_queries
        .then(|| {
            generated_offset_for_expression(
                virtual_ts,
                source_start,
                source_text,
                &expression.content,
            )
        })
        .flatten();
    let include_call_callees = expression_generated_offset.is_some();
    let call_ranges = profile!(
        "patina.type_aware.template_query_sets.call_ranges",
        collect_template_call_ranges(
            allocator,
            source_type,
            source_text,
            allow_statement_fallback,
            include_call_callees,
            include_template_promise_queries,
        )
    );
    // `call_ranges` owns every range (plain integers / bools) and borrows nothing
    // from the arena, so the arena can be recycled before the ranges are consumed.
    // Resetting here reuses the parse memory for the next template expression.
    allocator.reset();

    if let Some(queries) = sinks.template_queries.as_deref_mut() {
        if call_ranges.expression_consumes_source
            && let Some(generated_offset) = expression_generated_offset
        {
            let generated_offset =
                expression_binding_generated_offset(&virtual_ts.content, generated_offset)
                    .unwrap_or(generated_offset);
            queries.push(TemplateQuery {
                kind: TemplateQueryKind::Expression,
                context,
                generated_offset,
                source_start,
                source_end,
                owner_start: source_start,
                owner_end: source_end,
            });
        }
        for callee in call_ranges.callees {
            let callee_start = source_start + callee.start;
            let callee_end = source_start + callee.end;
            let Some(callee_source) = source_text.get(callee.start as usize..callee.end as usize)
            else {
                continue;
            };
            let Some(generated_offset) = generated_offset_for_nested_text(
                virtual_ts,
                source_start,
                source_text,
                callee.start,
                callee_source,
            )
            .or_else(|| generated_offset_for_text(virtual_ts, callee_start, callee_source)) else {
                continue;
            };
            queries.push(TemplateQuery {
                kind: TemplateQueryKind::CallCallee,
                context,
                generated_offset,
                source_start: callee_start,
                source_end: callee_end,
                owner_start: source_start,
                owner_end: source_end,
            });
        }
    }

    if let Some(queries) = sinks.template_promise_queries.as_deref_mut() {
        for candidate in call_ranges.floating_promises {
            let candidate_start = source_start + candidate.start;
            let candidate_end = source_start + candidate.end;
            let probe_start = source_start + candidate.probe_start;
            let Some(probe_source) =
                source_text.get(candidate.probe_start as usize..candidate.probe_end as usize)
            else {
                continue;
            };
            let Some(generated_offset) = generated_offset_for_nested_text(
                virtual_ts,
                source_start,
                source_text,
                candidate.probe_start,
                probe_source,
            )
            .or_else(|| generated_offset_for_text(virtual_ts, probe_start, probe_source)) else {
                continue;
            };
            queries.push(TemplatePromiseQuery {
                context,
                generated_offset,
                source_start: candidate_start,
                source_end: candidate_end,
            });
        }
    }
}

/// Map a nested token through the comment-stripped expression Canon emitted.
///
/// Projection rows own the complete authored expression. When ordinary
/// comments are removed, source and generated bytes stop being affine after
/// the first comment. The stripping witness restores that exact byte mapping
/// for callees and Promise probes without guessing from identifier text.
fn generated_offset_for_nested_text(
    virtual_ts: &TypeAwareDocument,
    source_start: u32,
    source_text: &str,
    nested_start: u32,
    nested_text: &str,
) -> Option<u32> {
    let (stripped, witness) = vize_croquis::drawer::strip_js_comments_with_offsets(source_text);
    let witness = witness?;
    let nested_start = usize::try_from(nested_start).ok()?;
    let stripped_start = witness
        .iter()
        .position(|authored| *authored == nested_start as u32)?;
    let stripped_end = stripped_start.checked_add(nested_text.len())?;
    if stripped.get(stripped_start..stripped_end) != Some(nested_text)
        || !(stripped_start..stripped_end).all(|at| {
            witness.get(at).copied()
                == nested_start
                    .checked_add(at - stripped_start)
                    .and_then(|offset| u32::try_from(offset).ok())
        })
    {
        return None;
    }

    let authored = source_start as usize..source_start as usize + source_text.len();
    for row in virtual_ts.mapping.rows() {
        let spans = std::iter::once((&row.span.src_range, &row.span.gen_range)).chain(
            row.span
                .sub_spans
                .iter()
                .map(|span| (&span.src_range, &span.gen_range)),
        );
        for (source, generated) in spans {
            if source != &authored {
                continue;
            }
            let start = generated.start.checked_add(stripped_start)?;
            let end = start.checked_add(nested_text.len())?;
            if end <= generated.end && virtual_ts.content.get(start..end) == Some(nested_text) {
                return u32::try_from(end.checked_sub(1)?).ok();
            }
        }
    }
    None
}

fn expression_binding_generated_offset(generated: &str, expression_offset: u32) -> Option<u32> {
    super::super::expression_bindings::binding_offset(generated, expression_offset)
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
