use super::document::TypeAwareDocument;
use super::{RULE_NO_FLOATING_PROMISES, RULE_NO_UNSAFE_TEMPLATE_BINDING};
use crate::diagnostic::LintDiagnostic;
use vize_relief::{ExpressionNode, RootNode};

mod calls;
mod collector;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum TemplateQueryKind {
    CallCallee,
    CallReturn,
    Expression,
    ForSource,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum TemplateContext {
    Binding,
    Directive,
    Event,
    Interpolation,
}

pub(super) struct TemplateQuery {
    pub kind: TemplateQueryKind,
    pub context: TemplateContext,
    pub generated_offset: u32,
    pub source_start: u32,
    pub source_end: u32,
    pub owner_start: u32,
    pub owner_end: u32,
}

pub(super) struct TemplatePromiseQuery {
    pub context: TemplateContext,
    pub generated_offset: u32,
    pub source_start: u32,
    pub source_end: u32,
}

impl TemplateQuery {
    pub fn owner_key(&self) -> u64 {
        ((self.owner_start as u64) << 32) | self.owner_end as u64
    }

    pub fn diagnostic(&self) -> LintDiagnostic {
        let message = match (self.kind, self.context) {
            (TemplateQueryKind::CallCallee, TemplateContext::Binding) => {
                "Template binding calls a value with an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::CallCallee, TemplateContext::Directive) => {
                "Template directive expression calls a value with an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::CallCallee, TemplateContext::Event) => {
                "Template event handler calls a value with an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::CallCallee, TemplateContext::Interpolation) => {
                "Template interpolation calls a value with an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::CallReturn, TemplateContext::Binding) => {
                "Template binding resolves to an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::CallReturn, TemplateContext::Directive) => {
                "Template directive expression resolves to an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::CallReturn, TemplateContext::Event) => {
                "Template event handler resolves to an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::CallReturn, TemplateContext::Interpolation) => {
                "Template interpolation resolves to an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::Expression, TemplateContext::Binding) => {
                "Template binding resolves to an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::Expression, TemplateContext::Directive) => {
                "Template directive expression resolves to an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::Expression, TemplateContext::Event) => {
                "Template event handler resolves to an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::Expression, TemplateContext::Interpolation) => {
                "Template interpolation resolves to an unsafe `any` or `unknown` type"
            }
            (TemplateQueryKind::ForSource, _) => {
                "Template v-for source contains an unsafe `any` or `unknown` item type"
            }
        };

        LintDiagnostic::warn(
            RULE_NO_UNSAFE_TEMPLATE_BINDING,
            message,
            self.source_start,
            self.source_end,
        )
        .with_help(
            "Give the value a concrete type in `<script setup>` or narrow it before using it in the template.",
        )
    }
}

impl TemplatePromiseQuery {
    pub fn diagnostic(&self) -> LintDiagnostic {
        let message = match self.context {
            TemplateContext::Binding => {
                "Template binding creates a Promise that is not awaited, returned, or explicitly ignored"
            }
            TemplateContext::Directive => {
                "Template directive expression creates a Promise that is not awaited, returned, or explicitly ignored"
            }
            TemplateContext::Event => {
                "Template event handler creates a Promise that is not awaited, returned, or explicitly ignored"
            }
            TemplateContext::Interpolation => {
                "Template interpolation creates a Promise that is not awaited, returned, or explicitly ignored"
            }
        };
        let help = match self.context {
            TemplateContext::Event => {
                "Move async work into a named handler and `await` it, or prefix the call with `void` when fire-and-forget behavior is intentional."
            }
            TemplateContext::Binding
            | TemplateContext::Directive
            | TemplateContext::Interpolation => {
                "Resolve the Promise in `<script setup>` and expose settled state to the template instead of creating async work during render."
            }
        };

        LintDiagnostic::warn(
            RULE_NO_FLOATING_PROMISES,
            message,
            self.source_start,
            self.source_end,
        )
        .with_help(help)
    }
}

pub(super) fn collect_template_query_sets(
    virtual_ts: &TypeAwareDocument,
    template_ast: &RootNode<'_>,
    template_offset: u32,
    include_template_queries: bool,
    include_template_promise_queries: bool,
) -> (Vec<TemplateQuery>, Vec<TemplatePromiseQuery>) {
    collector::collect_template_query_sets(
        virtual_ts,
        template_ast,
        template_offset,
        include_template_queries,
        include_template_promise_queries,
    )
}

pub(super) fn absolute_expression_range(
    expression: &ExpressionNode<'_>,
    template_offset: u32,
) -> Option<(u32, u32)> {
    let location = expression.loc();
    let source_start = template_offset + location.span.start;
    let source_end = template_offset + location.span.end;
    (source_end > source_start).then_some((source_start, source_end))
}

pub(super) fn generated_offset_for_text(
    virtual_ts: &TypeAwareDocument,
    source_start: u32,
    source_text: &str,
) -> Option<u32> {
    let trimmed = source_text.trim_end_matches(char::is_whitespace);
    let probe_offset = probe_offset_for_text(source_start, source_text)?;
    let source_end = source_start as usize + trimmed.len();
    // A shorthand binding (`:name`) and a bare event handler (`@click="save"`)
    // map the same authored text to both a synthetic check identifier and the
    // actual expression. Probe the expression's exact sub-span so synthetic
    // `unknown`/`any` types cannot become false unsafe-binding diagnostics.
    for row in virtual_ts
        .mapping
        .rows_containing_authored(probe_offset as usize)
    {
        for sub_span in &row.span.sub_spans {
            if sub_span.src_range == (source_start as usize..source_end)
                && virtual_ts.content.get(sub_span.gen_range.clone()) == Some(trimmed)
            {
                return u32::try_from(sub_span.gen_range.end.checked_sub(1)?).ok();
            }
        }
    }
    virtual_ts.generated_offset(probe_offset)
}

/// Probe the inferred iterable from a `v-for` source instead of the last byte
/// of its generated expression. A trailing call parenthesis or `as T[]` type
/// assertion has no useful symbol type even when the iterable is fully typed.
pub(super) fn v_for_source_binding_offset(generated: &str, expression_offset: u32) -> Option<u32> {
    let offset = expression_offset as usize;
    let line_start = generated.get(..offset)?.rfind('\n').map_or(0, |at| at + 1);
    let line_end = generated
        .get(offset..)?
        .find('\n')
        .map_or(generated.len(), |at| offset + at);
    let line = generated.get(line_start..line_end)?;
    let marker = "const __vize_v_for_source_";
    let name_start = line.find(marker)? + "const ".len();
    let name_end = name_start + line.get(name_start..)?.find(" = ")?;
    u32::try_from(line_start + name_end.checked_sub(1)?).ok()
}

fn probe_offset_for_text(source_start: u32, source_text: &str) -> Option<u32> {
    let trimmed = source_text.trim_end_matches(char::is_whitespace);
    (!trimmed.is_empty()).then_some(source_start + trimmed.len() as u32 - 1)
}
