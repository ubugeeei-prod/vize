//! `v-if` chains and `v-for` loops: flushed `_push` blocks around runtime
//! control flow, with the legacy lane's fragment-marker rules.

use vize_atelier_core::RuntimeHelper;
use vize_davinci::id::NodeId;
use vize_s0::FxHashSet;
use vize_s1_to_s2::TransformContent;
use vize_s2::expr::ExprRef;
use vize_s2::op::{self as s2, Op};

use super::spans::{expression_span, quoted_value};
use super::{Emitter, Flags, Result};
use crate::codegen::helpers::extract_destructure_params;
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringSegment, SsrStringSegmentKind as Kind,
};
use crate::s4::{AdmissionFailure, LegacyReason};

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    /// `process_if`: one `if` / `else if` / `else` block per branch, and an
    /// empty-comment `else` when the chain has no unconditional branch.
    pub(super) fn if_chain(
        &mut self,
        if_op: &'r s2::IfOp<'a>,
        disable_nested_fragments: bool,
        inherit_attrs: bool,
    ) -> Result<()> {
        let fact = self.segments[self.pos].fact;
        self.pos += 1;
        let mut conditions = std::vec::Vec::with_capacity(if_op.branches.len());
        for (index, branch) in if_op.branches.iter().enumerate() {
            match (index, &branch.condition) {
                (_, Some(condition)) => {
                    let text = self.expr(condition, TransformContent::Decoded)?;
                    let span =
                        expression_span(condition).map(|span| quoted_value(self.ctx.source, span));
                    conditions.push(Some((text, span)));
                }
                (0, None) => return Err(LegacyReason::Structure.into()),
                (_, None) => conditions.push(None),
            }
        }
        self.ctx.flush_push();
        for (index, (branch, condition)) in if_op.branches.iter().zip(conditions).enumerate() {
            self.close(
                Kind::Branch,
                |source| matches!(source, Source::Branch(open) if core::ptr::eq(*open, branch)),
            )?;
            self.ctx.push_indent();
            // Each branch opens at its authored unit, as the AST walker's does.
            let unit = branch.span.start;
            match (index, condition) {
                (0, Some((condition, span))) => {
                    self.ctx.push_mapped("if (", unit);
                    self.push_expression(&condition, span);
                    self.ctx.push(") {\n");
                }
                (_, Some((condition, span))) => {
                    self.ctx.push_then_mapped("} ", "else if (", unit);
                    self.push_expression(&condition, span);
                    self.ctx.push(") {\n");
                }
                (_, None) => self.ctx.push_then_mapped("} ", "else {\n", unit),
            }
            self.ctx.indent_level += 1;
            let shape = self.scan_region(self.pos)?;
            self.branch_key = self.branch_key_span(fact, index)?;
            let emitted = self.children(Flags {
                as_fragment: !disable_nested_fragments && shape.legacy_children > 1,
                disable_nested_fragments,
                inherit_attrs,
            });
            self.branch_key = None;
            emitted?;
            self.ctx.flush_push();
            self.ctx.indent_level -= 1;
            self.close(
                Kind::CloseBranch,
                |source| matches!(source, Source::Branch(open) if core::ptr::eq(*open, branch)),
            )?;
        }
        if if_op
            .branches
            .iter()
            .all(|branch| branch.condition.is_some())
        {
            self.ctx.push_indent();
            self.ctx.push("} else {\n");
            self.ctx.indent_level += 1;
            self.ctx.push_string_part_static("<!---->");
            self.ctx.flush_push();
            self.ctx.indent_level -= 1;
        }
        self.ctx.push_indent();
        self.ctx.push("}\n");
        self.close(
            Kind::CloseIf,
            |source| matches!(source, Source::If(open) if core::ptr::eq(*open, if_op)),
        )
    }

    /// `process_for`: `_ssrRenderList(source, (aliases) => { ... })` between
    /// fragment markers, with the body inside the aliases' scope.
    pub(super) fn for_loop(
        &mut self,
        open: SsrStringSegment<'r, 'a>,
        for_op: &'r s2::ForOp<'a>,
        disable_nested_fragments: bool,
    ) -> Result<()> {
        self.pos += 1;
        let binding = &for_op.binding;
        let source = self.expr(&binding.source, TransformContent::Decoded)?;
        let value = alias(&binding.value)?;
        let key = binding.key.as_ref().map(alias).transpose()?;
        let index = binding.index.as_ref().map(alias).transpose()?;
        let keyed_template = self.keyed_template(open.fact, &for_op.region.ops)?;

        self.ctx.flush_push();
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderList);
        if !disable_nested_fragments {
            self.ctx.push_indent();
            self.ctx.push("_push(`<!--[-->`)\n");
        }
        self.ctx.push_indent();
        self.ctx.push_mapped("_ssrRenderList(", for_op.span.start);
        self.push_expression(&source, expression_span(&binding.source));
        self.ctx.push(", (");
        // The walker anchors the loop source but not the aliases it binds.
        self.ctx.push(value);
        for alias in [key, index].into_iter().flatten() {
            self.ctx.push(", ");
            self.ctx.push(alias);
        }
        self.ctx.push(") => {\n");
        self.ctx.indent_level += 1;

        let mark = self.exprs.enter_for([Some(value), key, index]);
        let mut params = FxHashSet::default();
        for alias in [Some(value), key, index].into_iter().flatten() {
            extract_destructure_params(alias.trim(), &mut params);
        }
        self.scoped_params.push(params);
        let body = self.scan_region(self.pos).and_then(|shape| {
            self.children(Flags {
                as_fragment: !disable_nested_fragments
                    && (shape.legacy_children > 1 || keyed_template),
                disable_nested_fragments: true,
                inherit_attrs: false,
            })
        });
        self.scoped_params.pop();
        self.exprs.leave(mark);
        body?;

        self.ctx.flush_push();
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("})\n");
        if !disable_nested_fragments {
            self.ctx.push_indent();
            self.ctx.push("_push(`<!--]-->`)\n");
        }
        self.close(
            Kind::CloseFor,
            |source| matches!(source, Source::For(open) if core::ptr::eq(*open, for_op)),
        )
    }

    /// Write an emitted expression, anchored at its authored `span`.
    pub(super) fn push_expression(&mut self, code: &str, span: Option<vize_s0::Span>) {
        match span {
            Some(span) => self.ctx.push_expression_text(code, span),
            None => self.ctx.push(code),
        }
    }

    /// A keyed `<template v-for>` whose content is not one plain element
    /// renders each item as its own fragment.
    fn keyed_template(&self, fact: u32, ops: &[Op<'_>]) -> Result<bool> {
        let id = NodeId::from_index(fact).ok_or(AdmissionFailure::Invalid(
            "string plan fact index is not a node id",
        ))?;
        let Some(wrapper) = self.facts.for_wrappers.get(id) else {
            return Ok(false);
        };
        let single_plain_element = matches!(ops, [Op::Element(_)]);
        Ok(wrapper.key.is_some() && !single_plain_element)
    }
}

/// A callback parameter spelled exactly as authored. Empty or typed
/// positions have no reproducible legacy spelling and keep the legacy lane.
pub(super) fn alias<'a>(expr: &ExprRef<'a>) -> Result<&'a str> {
    let source = match expr {
        ExprRef::Js(js) => js.source,
        ExprRef::Opaque(opaque) => opaque.source,
        _ => return Err(LegacyReason::ExpressionOrEncoding.into()),
    };
    if source.trim().is_empty() || source.trim() != source {
        return Err(LegacyReason::Structure.into());
    }
    Ok(source)
}
