//! Component slot content: the static slots object whose `_withCtx` slot
//! functions render the push form and return the VNode fallback.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{FxHashSet, String, ToCompactString};
use vize_s2::op::{self as s2, DynamicName};

use super::{Emitter, PLAIN, Result};
use crate::codegen::element::props::{is_valid_js_identifier, quoted_js_string};
use crate::codegen::helpers::extract_destructure_params;
use crate::s4::string_plan::{SsrSegmentSource as Source, SsrStringSegmentKind as Kind};
use crate::s4::{AdmissionFailure, LegacyReason};

/// Plan ranges `[start, end)` of slot children.
pub(super) type Ranges = std::vec::Vec<(usize, usize)>;

/// One slot function: its name, props pattern, and children.
pub(super) struct SlotSpec {
    pub(super) name: String,
    pub(super) pattern: Option<String>,
    pub(super) ranges: Ranges,
    /// Authored starts of the slot name token and of the element carrying
    /// the slot, as the AST walker anchors them (P3-9 source maps).
    pub(super) anchor: SlotAnchor,
}

/// `(slot name start, carrying element start)`; both absent for the
/// implicit default slot.
pub(super) type SlotAnchor = (Option<u32>, Option<u32>);

/// Start of the static slot name in the `#name` / `v-slot:name` directive.
fn slot_name_start(file: &str, content: &s2::SlotContentOp<'_>) -> Option<u32> {
    let Some(DynamicName::Static(name)) = content.name else {
        return None;
    };
    let span = content.span;
    let raw = file.get(span.start as usize..span.end as usize)?;
    let rest = ["v-slot:", "#"]
        .iter()
        .find_map(|prefix| raw.strip_prefix(prefix))?;
    rest.starts_with(name)
        .then(|| span.start + (raw.len() - rest.len()) as u32)
}

/// A component's slot content in the legacy static-object shape.
pub(super) struct ComponentSlots {
    /// `v-slot` on the component: every child belongs to that slot.
    pub(super) own: Option<SlotSpec>,
    pub(super) default: Ranges,
    pub(super) named: std::vec::Vec<SlotSpec>,
    /// A `<slot>` outlet anywhere in the content forwards slots.
    pub(super) forwards: bool,
    /// `createSlots` entries, in content order.
    pub(super) dynamic: std::vec::Vec<super::create_slots::Dynamic>,
}

/// The `v-slot` binding of a slot carrier.
fn slot_content<'r, 'a>(bindings: &'r [s2::BindingOp<'a>]) -> Option<&'r s2::SlotContentOp<'a>> {
    bindings.iter().find_map(|binding| match binding {
        s2::BindingOp::SlotContent(content) => Some(&**content),
        _ => None,
    })
}

/// The authored value between the attribute quotes when only whitespace pads
/// `source` (the legacy lane reads the directive value's whole span).
fn quote_padded<'f>(file: &'f str, source: &'f str, span: vize_s0::Span) -> &'f str {
    let (start, end) = (span.start as usize, span.end as usize);
    if file.get(start..end) != Some(source) {
        return source;
    }
    let before = &file[..start];
    let after = &file[end..];
    let open = before.trim_end_matches(|c: char| c.is_ascii_whitespace());
    let close = after.trim_start_matches(|c: char| c.is_ascii_whitespace());
    match (open.chars().last(), close.chars().next()) {
        (Some(quote @ ('"' | '\'')), Some(closing)) if quote == closing => {
            &file[open.len()..file.len() - close.len()]
        }
        _ => source,
    }
}

/// The slot name and props pattern of a `v-slot` spelling.
fn slot_head(file: &str, content: &s2::SlotContentOp<'_>) -> Result<(String, Option<String>)> {
    let name = match content.name {
        None => "default".to_compact_string(),
        Some(DynamicName::Static(name)) => name.to_compact_string(),
        Some(DynamicName::Dynamic(_)) => return Err(LegacyReason::Operation.into()),
    };
    Ok((name, slot_pattern(file, content)?))
}

/// The props pattern of a `v-slot` spelling, TypeScript erased.
pub(super) fn slot_pattern(file: &str, content: &s2::SlotContentOp<'_>) -> Result<Option<String>> {
    if !content.modifiers.is_empty() {
        return Err(LegacyReason::Operation.into());
    }
    Ok(content.params.map(|params| {
        let authored = quote_padded(file, params.source(), params.span());
        vize_atelier_core::steps::strip_typescript_from_expression(authored)
    }))
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    /// Classify the component content region starting at `start`.
    pub(super) fn component_slots(
        &self,
        component: &'r s2::ComponentOp<'a>,
        start: usize,
    ) -> Result<ComponentSlots> {
        let children = self.direct_children(start)?;
        let end = match children.last() {
            Some(last) => self.child_end(*last)?,
            None => start,
        };
        let forwards = self.segments[start..end]
            .iter()
            .any(|segment| segment.kind == Kind::SlotOutlet);
        let mut slots = ComponentSlots {
            own: None,
            default: std::vec::Vec::new(),
            named: std::vec::Vec::new(),
            forwards,
            dynamic: std::vec::Vec::new(),
        };
        if let Some(content) = slot_content(&component.bindings) {
            let (name, pattern) = slot_head(self.ctx.source, content)?;
            // The walker puts every child in this one slot. A nested
            // `<template v-slot>` is transparent there, not a second slot.
            let mut ranges = std::vec::Vec::new();
            for child in children {
                ranges.push((child, self.child_end(child)?));
            }
            slots.own = Some(SlotSpec {
                name,
                pattern,
                ranges,
                anchor: (
                    slot_name_start(self.ctx.source, content),
                    Some(component.span.start),
                ),
            });
            return Ok(slots);
        }
        for child in children {
            let child_end = self.child_end(child)?;
            if let Some(dynamic) = self.dynamic_slot_source(child)? {
                slots.dynamic.push(dynamic);
            } else if let Some((element, content)) = self.template_slot(child) {
                let (name, pattern) = slot_head(self.ctx.source, content)?;
                let inner = child + 1 + element.attributes.len() + element.bindings.len();
                let ranges = self
                    .direct_children(inner)?
                    .into_iter()
                    .map(|start| self.child_end(start).map(|end| (start, end)))
                    .collect::<Result<Ranges>>()?;
                slots.named.push(SlotSpec {
                    name,
                    pattern,
                    ranges,
                    anchor: (
                        slot_name_start(self.ctx.source, content),
                        Some(element.span.start),
                    ),
                });
            } else if self.nested_carrier(child, child_end)? {
                // A slot carrier nested below plain content has no legacy
                // `createSlots` entry shape the plan emitter reproduces.
                return Err(LegacyReason::Operation.into());
            } else {
                slots.default.push((child, child_end));
            }
        }
        Ok(slots)
    }

    /// Whether `[start, end)` holds a slot carrier outside the content of
    /// nested components (which own their own carriers).
    fn nested_carrier(&self, start: usize, end: usize) -> Result<bool> {
        let mut pos = start;
        while pos < end {
            let segment = self.segments[pos];
            match (segment.kind, segment.source) {
                (Kind::Component, _) => pos = self.child_end(pos)?,
                (_, Source::Element(element)) if slot_content(&element.bindings).is_some() => {
                    return Ok(true);
                }
                _ => pos += 1,
            }
        }
        Ok(false)
    }

    /// The `<template v-slot>` carrier at `start`, if it is one.
    pub(super) fn template_slot(
        &self,
        start: usize,
    ) -> Option<(&'r s2::ElementOp<'a>, &'r s2::SlotContentOp<'a>)> {
        let segment = self.segments.get(start)?;
        match (segment.kind, segment.source) {
            (Kind::OpenElement, Source::Element(element)) if element.tag == "template" => {
                slot_content(&element.bindings).map(|content| (element, content))
            }
            _ => None,
        }
    }

    /// `_: 3` for forwarded slots at the top scope, `_: 2` inside scoped
    /// params, `_: 1` otherwise.
    pub(super) fn slot_flag(&self, slots: &ComponentSlots) -> &'static str {
        match (slots.forwards, self.scoped_params.is_empty()) {
            (true, true) => "_: 3 /* FORWARDED */",
            (true, false) => "_: 2 /* DYNAMIC */",
            (false, _) => "_: 1",
        }
    }

    /// The push-form slots object, written in place.
    pub(super) fn emit_slots_object(&mut self, slots: &ComponentSlots) -> Result<()> {
        if !slots.dynamic.is_empty() {
            return self.emit_create_slots(slots);
        }
        self.ctx.use_core_helper(RuntimeHelper::WithCtx);
        self.ctx.push("{\n");
        self.ctx.indent_level += 1;
        if let Some(own) = &slots.own {
            self.slot_property(own)?;
        } else {
            if !slots.default.is_empty() {
                let default = SlotSpec {
                    name: "default".to_compact_string(),
                    pattern: None,
                    ranges: slots.default.clone(),
                    anchor: (None, None),
                };
                self.slot_property(&default)?;
            }
            for named in &slots.named {
                self.slot_property(named)?;
            }
        }
        self.ctx.push_indent();
        let flag = self.slot_flag(slots);
        self.ctx.push(flag);
        self.ctx.push("\n");
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("}");
        Ok(())
    }

    pub(super) fn slot_property(&mut self, spec: &SlotSpec) -> Result<()> {
        self.ctx.push_indent();
        // The name maps to its authored `v-slot` argument, as the walker's does.
        if is_valid_js_identifier(&spec.name) {
            self.ctx.push_optionally_mapped(&spec.name, spec.anchor.0);
        } else {
            let quoted = quoted_js_string(&spec.name);
            self.ctx.push("\"");
            self.ctx
                .push_optionally_mapped(&quoted[1..quoted.len() - 1], spec.anchor.0);
            self.ctx.push("\"");
        }
        self.ctx.push(": ");
        self.slot_fn(spec)?;
        self.ctx.push(",\n");
        Ok(())
    }

    /// `_withCtx((params, _push, _parent, _scopeId) => { if (_push) { ... }
    /// else { return [...] } })`.
    pub(super) fn slot_fn(&mut self, spec: &SlotSpec) -> Result<()> {
        self.ctx.use_core_helper(RuntimeHelper::WithCtx);
        self.ctx.push_optionally_mapped("_withCtx((", spec.anchor.1);
        self.ctx.push(spec.pattern.as_deref().unwrap_or("_"));
        self.ctx.push(", _push, _parent, _scopeId) => {\n");
        self.ctx.indent_level += 1;
        self.ctx.push_indent();
        self.ctx.push("if (_push) {\n");
        self.ctx.indent_level += 1;

        let mut params = FxHashSet::default();
        if let Some(pattern) = spec.pattern.as_deref() {
            extract_destructure_params(pattern.trim(), &mut params);
        }
        let mark = self.exprs.enter_slot(spec.pattern.as_deref().unwrap_or(""));
        let saved_parts = core::mem::take(&mut self.ctx.current_template_parts);
        let saved_scope = self.ctx.with_slot_scope_id;
        self.ctx.with_slot_scope_id = true;
        let scoped = !params.is_empty();
        if scoped {
            self.scoped_params.push(params.clone());
        }
        let pushed = self.slot_children(&spec.ranges);
        self.ctx.flush_push();
        if scoped {
            self.scoped_params.pop();
        }
        self.ctx.with_slot_scope_id = saved_scope;
        self.ctx.current_template_parts = saved_parts;
        let fallback = pushed.and_then(|()| {
            if scoped {
                self.scoped_params.push(params);
            }
            let fallback = self.vnode_list(&spec.ranges);
            if scoped {
                self.scoped_params.pop();
            }
            fallback
        });
        self.exprs.leave(mark);
        let fallback = fallback?;

        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("} else {\n");
        self.ctx.indent_level += 1;
        self.ctx.push_indent();
        self.ctx.push("return ");
        self.ctx.push(&fallback);
        self.ctx.push("\n");
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("}\n");
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("})");
        Ok(())
    }

    /// The push form of slot children, each child in its own range.
    fn slot_children(&mut self, ranges: &Ranges) -> Result<()> {
        for &(start, end) in ranges {
            self.pos = start;
            let segment = self.segments[start];
            self.child(segment, PLAIN, false)?;
            if self.pos != end {
                return Err(AdmissionFailure::Invalid(
                    "slot child emission left its plan range",
                ));
            }
        }
        Ok(())
    }
}
