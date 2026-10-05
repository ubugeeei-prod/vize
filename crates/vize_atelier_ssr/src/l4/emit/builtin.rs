//! Builtin writers over the original SSR string-plan regions.

use vize_atelier_core::RuntimeHelper;
use vize_l1_to_l2::TransformContent;
use vize_l2::op::{self as l2, DynamicName};

use crate::codegen::element::props::{is_simple_identifier, quoted_js_string};
use crate::l4::string_plan::{SsrStringPayloadKind, SsrStringSegmentKind as Kind};

use super::spans::expression_span;
use super::{Emitter, Flags, Result, plan_source};

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    pub(super) fn suspense(&mut self, flags: Flags) -> Result<()> {
        let children = self.direct_children(self.pos)?;
        let separate = children.iter().any(|&start| {
            self.template_slot(start).is_some_and(|(_, slot)| {
                !matches!(slot.name, None | Some(DynamicName::Static("default")))
            })
        });
        if !separate {
            return self.suspense_default(flags);
        }
        let end = self.region_end(self.pos)?;
        let mut implicit = std::vec::Vec::new();
        let mut dynamic = false;
        self.ctx.flush_push();
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderSuspense);
        self.ctx.push_indent();
        self.ctx.push("_ssrRenderSuspense(_push, {\n");
        self.ctx.indent_level += 1;
        for start in children {
            let Some((element, slot)) = self.template_slot(start) else {
                if self.segment(start)?.kind != Kind::Comment {
                    implicit.push(start);
                }
                continue;
            };
            self.ctx.push_indent();
            self.suspense_name(slot)?;
            dynamic |= matches!(slot.name, Some(DynamicName::Dynamic(_)));
            let inner = start + 1 + element.attributes.len() + element.bindings.len();
            let ranges = self.direct_children(inner)?;
            self.suspense_body(&ranges, flags)?;
        }
        if implicit.iter().all(|&start| {
            self.segments.get(start).is_some_and(|segment| {
                segment.kind == Kind::Text
                    && plan_source(segment, SsrStringPayloadKind::Text)
                        .is_ok_and(|text| text.trim().is_empty())
            })
        }) {
            implicit.clear();
        }
        if !implicit.is_empty() {
            self.ctx.push_indent();
            self.ctx.push("default");
            self.suspense_body(&implicit, flags)?;
        }
        self.ctx.push_indent();
        self.ctx.push(if dynamic { "_: 2\n" } else { "_: 1\n" });
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("})\n");
        self.pos = end;
        Ok(())
    }

    fn suspense_name(&mut self, slot: &l2::SlotContentOp<'_>) -> Result<()> {
        match &slot.name {
            None => self.ctx.push("default"),
            Some(DynamicName::Static(name)) => {
                let start = super::slots::slot_name_start(self.ctx.source, slot);
                if is_simple_identifier(name) {
                    self.ctx.push_optionally_mapped(name, start);
                } else {
                    let key = quoted_js_string(name);
                    let inner = key
                        .strip_prefix('"')
                        .and_then(|key| key.strip_suffix('"'))
                        .unwrap_or_default();
                    self.ctx.push("\"");
                    self.ctx.push_optionally_mapped(inner, start);
                    self.ctx.push("\"");
                }
            }
            Some(DynamicName::Dynamic(expr)) => {
                let key = self.expr(expr, TransformContent::Decoded)?;
                self.ctx.push("[");
                self.push_expression(&key, expression_span(expr));
                self.ctx.push("]");
            }
        }
        Ok(())
    }

    fn suspense_body(&mut self, children: &[usize], flags: Flags) -> Result<()> {
        self.ctx.push(": () => {\n");
        self.ctx.indent_level += 1;
        let saved = core::mem::take(&mut self.ctx.current_template_parts);
        let result = children.iter().try_for_each(|&start| {
            let end = self.child_end(start)?;
            self.pos = start;
            self.child(self.segment(start)?, flags, false)?;
            if self.pos != end {
                return Err(crate::l4::AdmissionFailure::Invalid(
                    "Suspense child emission left its original plan range",
                ));
            }
            Ok(())
        });
        self.ctx.flush_push();
        self.ctx.current_template_parts = saved;
        self.ctx.indent_level -= 1;
        result?;
        self.ctx.push_indent();
        self.ctx.push("},\n");
        Ok(())
    }

    /// `_ssrRenderSuspense(_push, { default: () => { ... }, _: 1 })`.
    fn suspense_default(&mut self, flags: Flags) -> Result<()> {
        self.ctx.flush_push();
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderSuspense);
        self.ctx.push_indent();
        self.ctx.push("_ssrRenderSuspense(_push, {\n");
        self.ctx.indent_level += 1;
        self.ctx.push_indent();
        self.ctx.push("default: () => {\n");
        self.ctx.indent_level += 1;
        self.children(flags)?;
        self.ctx.flush_push();
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("},\n");
        self.ctx.push_indent();
        self.ctx.push("_: 1\n");
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("})\n");
        Ok(())
    }
}
