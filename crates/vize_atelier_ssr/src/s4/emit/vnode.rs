//! The VNode fallback a slot function returns when the parent renders on
//! the client: `_createElementVNode` / `_createVNode` / `_createTextVNode`
//! trees built from the same plan ranges as the push form.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, cstr};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::op as s2;

use super::slots::Ranges;
use super::{Emitter, Result, plan_source, text};
use crate::codegen::element::props::quoted_js_string;
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringSegmentKind as Kind,
};
use crate::s4::{AdmissionFailure, LegacyReason};

/// `[a, b]` over already rendered children.
pub(super) fn array(expressions: &[String]) -> String {
    cstr!("[{}]", expressions.join(", "))
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    /// `vnode_children_expression` over plan ranges.
    pub(super) fn vnode_list(&mut self, ranges: &Ranges) -> Result<String> {
        let mut expressions = std::vec::Vec::new();
        for &(start, end) in ranges {
            self.pos = start;
            self.vnode_child(&mut expressions)?;
            if self.pos != end {
                return Err(AdmissionFailure::Invalid(
                    "vnode emission left its plan range",
                ));
            }
        }
        Ok(array(&expressions))
    }

    /// The VNode expressions of the region at the cursor, up to its close.
    pub(super) fn vnode_region(&mut self) -> Result<std::vec::Vec<String>> {
        let mut expressions = std::vec::Vec::new();
        while let Some(segment) = self.segments.get(self.pos) {
            if super::region::is_close(segment.kind) {
                break;
            }
            self.vnode_child(&mut expressions)?;
        }
        Ok(expressions)
    }

    /// `vnode_child_expression` for the child at the cursor; a merged S2 text
    /// run yields one expression per legacy child.
    fn vnode_child(&mut self, out: &mut std::vec::Vec<String>) -> Result<()> {
        let segment = self.segments[self.pos];
        match (segment.kind, segment.source) {
            (Kind::Text, Source::Text(_)) => {
                self.pos += 1;
                let content = plan_source(&segment, SsrStringPayloadKind::Text)?;
                if let Some(expression) = self.vnode_text(content)? {
                    out.push(expression);
                }
            }
            (Kind::DynamicText, Source::Interpolation(interpolation)) => {
                self.pos += 1;
                match text::compound_parts(self.facts.texts, &segment, interpolation)? {
                    Some(parts) => {
                        for part in parts {
                            if part.dynamic {
                                let exp = self.text_expr(part.text.as_str())?;
                                out.push(self.vnode_display(&exp));
                            } else if let Some(expression) = self.vnode_text(&part.text)? {
                                out.push(expression);
                            }
                        }
                    }
                    None => {
                        let exp = self.expr(&interpolation.expression, TransformContent::Padded)?;
                        out.push(self.vnode_display(&exp));
                    }
                }
            }
            (Kind::OpenElement, Source::Element(element)) => {
                let expression =
                    vize_s0::ensure_sufficient_stack(|| self.vnode_element(element, None))?;
                out.push(expression);
            }
            (Kind::Component, Source::Component(component)) => {
                let expression =
                    vize_s0::ensure_sufficient_stack(|| self.vnode_component(component))?;
                out.push(expression);
            }
            (Kind::SlotOutlet, Source::Slot(slot)) => {
                let expression = vize_s0::ensure_sufficient_stack(|| self.vnode_outlet(slot))?;
                out.push(expression);
            }
            (Kind::If, Source::If(if_op)) => {
                let expression = vize_s0::ensure_sufficient_stack(|| self.vnode_if(if_op))?;
                out.push(expression);
            }
            (Kind::For, Source::For(for_op)) => {
                let expression = vize_s0::ensure_sufficient_stack(|| self.vnode_for(for_op))?;
                out.push(expression);
            }
            _ => return Err(LegacyReason::Operation.into()),
        }
        Ok(())
    }

    /// `_createTextVNode("...")`; empty text renders nothing.
    fn vnode_text(&mut self, content: &str) -> Result<Option<String>> {
        let decoded = decode_template_entities(content);
        text::admit_decoded(content, &decoded)?;
        if decoded.is_empty() {
            return Ok(None);
        }
        self.ctx.use_core_helper(RuntimeHelper::CreateText);
        let quoted = quoted_js_string(&decoded);
        Ok(Some(cstr!("_createTextVNode({quoted})")))
    }

    fn vnode_display(&mut self, exp: &str) -> String {
        self.ctx.use_core_helper(RuntimeHelper::CreateText);
        self.ctx.use_core_helper(RuntimeHelper::ToDisplayString);
        cstr!("_createTextVNode(_toDisplayString({exp}))")
    }

    /// `_createElementVNode("tag", props, children)`.
    pub(super) fn vnode_element(
        &mut self,
        element: &'r s2::ElementOp<'a>,
        key: Option<&str>,
    ) -> Result<String> {
        let open = self.segments[self.pos];
        let tag = plan_source(&open, SsrStringPayloadKind::TagName)?;
        self.pos += 1;
        let attached = self.take_attached(element.attributes.len() + element.bindings.len())?;
        let attached = attached.as_slice();
        self.ctx.use_core_helper(RuntimeHelper::CreateElementVNode);
        let props = self.vnode_element_props(attached, key)?;
        let children = match self.direct_children(self.pos)?.as_slice() {
            [] => "null".to_compact_string(),
            [only] if self.segments[*only].kind == Kind::Text => {
                let segment = self.segments[*only];
                self.pos += 1;
                let content = plan_source(&segment, SsrStringPayloadKind::Text)?;
                let decoded = decode_template_entities(content);
                text::admit_decoded(content, &decoded)?;
                quoted_js_string(&decoded)
            }
            _ => array(&self.vnode_region()?),
        };
        self.close(
            Kind::CloseElement,
            |source| matches!(source, Source::Element(closed) if core::ptr::eq(*closed, element)),
        )?;
        let tag = quoted_js_string(tag);
        Ok(cstr!("_createElementVNode({tag}, {props}, {children})"))
    }

    /// `_createVNode(callee, props, slots)`.
    fn vnode_component(&mut self, component: &'r s2::ComponentOp<'a>) -> Result<String> {
        let open = self.segments[self.pos];
        let name = plan_source(&open, SsrStringPayloadKind::ComponentName)?;
        if matches!(name, "component" | "Component") {
            return Err(LegacyReason::Operation.into());
        }
        self.pos += 1;
        let attached = self.take_attached(component.attributes.len() + component.bindings.len())?;
        let attached = attached.as_slice();
        self.ctx.use_core_helper(RuntimeHelper::CreateVNode);
        let callee = match self.ctx.resolve_component_binding_expr(name) {
            Some(binding) => binding,
            None => self.ctx.resolved_component_callee(name),
        };
        let props = self.component_props(attached)?;
        let slots = self.vnode_slots(component)?;
        self.close(Kind::CloseComponent, |source| {
            matches!(source, Source::Component(closed) if core::ptr::eq(*closed, component))
        })?;
        Ok(cstr!("_createVNode({callee}, {props}, {slots})"))
    }

    /// The VNode slots object of the component content at the cursor.
    fn vnode_slots(&mut self, component: &'r s2::ComponentOp<'a>) -> Result<String> {
        let start = self.pos;
        let slots = self.component_slots(component, start)?;
        let has_content =
            slots.own.is_some() || !slots.default.is_empty() || !slots.named.is_empty();
        if !has_content {
            return Ok("null".to_compact_string());
        }
        self.ctx.use_core_helper(RuntimeHelper::WithCtx);
        let mut out = String::from("{ ");
        if let Some(own) = &slots.own {
            out.push_str(&self.vnode_slot_entry(own)?);
            out.push_str(", ");
        } else {
            for named in &slots.named {
                out.push_str(&self.vnode_slot_entry(named)?);
                out.push_str(", ");
            }
            if !slots.default.is_empty() {
                out.push_str("default: _withCtx(() => ");
                out.push_str(&self.vnode_list(&slots.default)?);
                out.push_str("), ");
            }
        }
        out.push_str(self.slot_flag(&slots));
        out.push_str(" }");
        self.pos = self.region_end(start)?;
        Ok(out)
    }

    /// `name: _withCtx((params) => [children])`.
    fn vnode_slot_entry(&mut self, spec: &super::slots::SlotSpec) -> Result<String> {
        let key = if is_identifier(&spec.name) {
            spec.name.clone()
        } else {
            quoted_js_string(&spec.name)
        };
        let mut params = vize_s0::FxHashSet::default();
        if let Some(pattern) = spec.pattern.as_deref() {
            crate::codegen::helpers::extract_destructure_params(pattern.trim(), &mut params);
        }
        let mark = self.exprs.enter_slot(spec.pattern.as_deref().unwrap_or(""));
        let scoped = !params.is_empty();
        if scoped {
            self.scoped_params.push(params);
        }
        let children = self.vnode_list(&spec.ranges);
        if scoped {
            self.scoped_params.pop();
        }
        self.exprs.leave(mark);
        let pattern = spec.pattern.as_deref().unwrap_or("_");
        Ok(cstr!("{key}: _withCtx(({pattern}) => {})", children?))
    }

    /// The position of the closing segment of the region starting at `start`.
    pub(super) fn region_end(&self, start: usize) -> Result<usize> {
        let children = self.direct_children(start)?;
        match children.last() {
            Some(last) => self.child_end(*last),
            None => Ok(start),
        }
    }
}

fn is_identifier(name: &str) -> bool {
    crate::codegen::element::props::is_valid_js_identifier(name)
}
