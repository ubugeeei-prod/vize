use super::RULE_NO_UNSAFE_TEMPLATE_BINDING;
use crate::diagnostic::LintDiagnostic;
use vize_croquis::virtual_ts::VirtualTsOutput;
use vize_relief::ast::{ExpressionNode, RootNode};

mod calls;
mod collector;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum TemplateQueryKind {
    CallCallee,
    Expression,
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

pub(super) fn collect_template_queries(
    virtual_ts: &VirtualTsOutput,
    template_ast: &RootNode<'_>,
    template_offset: u32,
) -> Vec<TemplateQuery> {
    collector::collect_template_queries(virtual_ts, template_ast, template_offset)
}

pub(super) fn absolute_expression_range(
    expression: &ExpressionNode<'_>,
    template_offset: u32,
) -> Option<(u32, u32)> {
    let location = expression.loc();
    let source_start = template_offset + location.start.offset;
    let source_end = template_offset + location.end.offset;
    (source_end > source_start).then_some((source_start, source_end))
}

pub(super) fn generated_offset_for_text(
    virtual_ts: &VirtualTsOutput,
    source_start: u32,
    source_text: &str,
) -> Option<u32> {
    let probe_offset = probe_offset_for_text(source_start, source_text)?;
    virtual_ts.source_map.to_generated(probe_offset)
}

fn probe_offset_for_text(source_start: u32, source_text: &str) -> Option<u32> {
    let trimmed = source_text.trim_end_matches(char::is_whitespace);
    (!trimmed.is_empty()).then_some(source_start + trimmed.len() as u32 - 1)
}
