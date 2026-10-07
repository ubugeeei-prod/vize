//! Native property identity for statically named calls to the actual macro result.

use std::ops::Range;

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, Program, Statement,
    VariableDeclarationKind,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::Scoping;
use oxc_syntax::symbol::SymbolId;
use vize_carton::{CompactString, FxHashSet, String, append};
use vize_croquis::Croquis;

use super::declarations::SetupHelperPlan;
use crate::virtual_ts::{VizeMapping, props::extract_generic_names};

pub(super) struct OwnedEmitCall {
    literal: CompactString,
    content: Range<usize>,
}

pub(super) fn collect(
    program: &Program<'_>,
    scoping: &Scoping,
    source: &str,
    shadowed: &FxHashSet<CompactString>,
    summary: &Croquis,
) -> Vec<OwnedEmitCall> {
    let Some(macro_call) = summary.macros.define_emits() else {
        return Vec::new();
    };
    if shadowed.contains("defineEmits") {
        return Vec::new();
    }
    let mut bindings = Vec::new();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        if declaration.kind != VariableDeclarationKind::Const {
            continue;
        }
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
                continue;
            };
            if let Some(Expression::CallExpression(call)) = declarator
                .init
                .as_ref()
                .map(Expression::get_inner_expression)
                && call.span.start == macro_call.start
                && call.span.end == macro_call.end
                && let Some(symbol) = binding.symbol_id.get()
                && scoping
                    .get_resolved_references(symbol)
                    .all(|reference| !reference.is_write())
            {
                bindings.push(symbol);
            }
        }
    }
    if bindings.is_empty() {
        return Vec::new();
    }
    let mut collector = EmitCalls {
        bindings,
        scoping,
        source,
        summary,
        calls: Vec::new(),
    };
    collector.visit_program(program);
    collector.calls
}

struct EmitCalls<'s> {
    bindings: Vec<SymbolId>,
    scoping: &'s Scoping,
    source: &'s str,
    summary: &'s Croquis,
    calls: Vec<OwnedEmitCall>,
}

impl<'a> Visit<'a> for EmitCalls<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = call.callee.get_inner_expression()
            && let Some(reference) = callee.reference_id.get()
            && let Some(symbol) = self.scoping.get_reference(reference).symbol_id()
            && self.bindings.contains(&symbol)
            && let Some(Argument::StringLiteral(event)) = call.arguments.first()
            && self
                .summary
                .macros
                .emits()
                .iter()
                .any(|definition| definition.name.as_str() == event.value.as_str())
        {
            let content = event.span.start as usize + 1..event.span.end as usize - 1;
            // Escaped source spelling has no byte-exact property projection.
            if !content.is_empty()
                && self.source.get(content.clone()) == Some(event.value.as_str())
                && let Some(literal) = self
                    .source
                    .get(event.span.start as usize..event.span.end as usize)
            {
                self.calls.push(OwnedEmitCall {
                    literal: CompactString::from(literal),
                    content,
                });
            }
        }
        walk::walk_call_expression(self, call);
    }
}

impl SetupHelperPlan {
    pub(crate) fn emit_event_navigation(
        &self,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        summary: &Croquis,
        generic_param: Option<&str>,
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        if self.emit_calls.is_empty() {
            return;
        }
        let generic_names = generic_param
            .filter(|_| super::define_emits_runtime_args(summary).is_none())
            .filter(|_| {
                !summary
                    .type_exports
                    .iter()
                    .any(|export| export.name == "Emits")
            })
            .map(extract_generic_names)
            .unwrap_or_default();
        let generic_suffix = if generic_names.is_empty() {
            String::default()
        } else {
            vize_carton::cstr!("<{generic_names}>")
        };
        append!(
            *ts,
            "  const __vize_emit_calls_nav = undefined as unknown as __VizeAuthoredEventMap{generic_suffix};\n"
        );
        for call in &self.emit_calls {
            ts.push_str("  void __vize_emit_calls_nav[");
            let start = ts.len() + 1;
            ts.push_str(&call.literal);
            mappings.push(VizeMapping::new(
                start..start + call.content.len(),
                source_offset(call.content.start)..source_offset(call.content.end),
            ));
            ts.push_str("];\n");
        }
    }
}

#[cfg(test)]
mod tests;
