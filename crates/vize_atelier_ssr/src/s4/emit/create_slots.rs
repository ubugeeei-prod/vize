//! `createSlots` shapes: slot templates under `v-if` / `v-for` and
//! dynamically named `<template #[name]>` carriers, emitted as
//! `_createSlots(base, [entries])` in the push form and the VNode fallback.
//!
//! The legacy lane keeps two quirks this port reproduces byte-for-byte: the
//! push form spells `_renderList` while registering `ssrRenderList`, and the
//! base object always carries `_: 2 /* DYNAMIC */`. The VNode fallback's
//! expression form lives in `vnode_create_slots`.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{FxHashSet, String, ToCompactString, cstr};
use vize_s1_to_s2::TransformContent;
use vize_s2::op::{self as s2, DynamicName};

use super::control::alias;
use super::slots::{ComponentSlots, SlotSpec, slot_pattern};
use super::{Emitter, Result};
use crate::codegen::element::props::{is_valid_js_identifier, quoted_js_string};
use crate::codegen::helpers::extract_destructure_params;
use crate::codegen::scope_prefix::strip_scope_prefixes_for_scoped_params;
use crate::s4::string_plan::{SsrSegmentSource as Source, SsrStringSegmentKind as Kind};
use crate::s4::{AdmissionFailure, LegacyReason};

/// A `createSlots` entry source, by the plan position of its content child.
#[derive(Clone, Copy)]
pub(super) enum Dynamic {
    /// A `v-if` chain with a slot template directly in some branch.
    Conditional(usize),
    /// A `v-for` whose region holds a slot template directly.
    Looped(usize),
    /// A `<template #[name]>` carrier.
    Named(usize),
}

/// One entry object's name expression and slot function.
pub(super) struct Entry {
    pub(super) name: String,
    pub(super) spec: SlotSpec,
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    /// `is_dynamic_slot_source` for the content child at `start`.
    pub(super) fn dynamic_slot_source(&self, start: usize) -> Result<Option<Dynamic>> {
        let segment = self.segments[start];
        match segment.kind {
            Kind::If => {
                for branch in self.branch_starts(start)? {
                    if self.slot_template_in(branch)?.is_some() {
                        return Ok(Some(Dynamic::Conditional(start)));
                    }
                }
                Ok(None)
            }
            Kind::For => Ok(self
                .slot_template_in(start + 1)?
                .map(|_| Dynamic::Looped(start))),
            _ => Ok(match self.template_slot(start) {
                Some((_, content)) if matches!(content.name, Some(DynamicName::Dynamic(_))) => {
                    Some(Dynamic::Named(start))
                }
                _ => None,
            }),
        }
    }

    /// Where each branch's children start in the `v-if` region at `at`.
    fn branch_starts(&self, at: usize) -> Result<std::vec::Vec<usize>> {
        let mut starts = std::vec::Vec::new();
        let mut cursor = at + 1;
        while self.segments.get(cursor).map(|segment| segment.kind) == Some(Kind::Branch) {
            starts.push(cursor + 1);
            cursor = self.child_end(cursor)?;
        }
        match self.segments.get(cursor) {
            Some(close) if close.kind == Kind::CloseIf => Ok(starts),
            _ => Err(AdmissionFailure::Invalid(
                "string plan v-if region has no closing segment",
            )),
        }
    }

    /// `slot_template_in_children`: the first direct `<template v-slot>`.
    pub(super) fn slot_template_in(&self, from: usize) -> Result<Option<usize>> {
        Ok(self
            .direct_children(from)?
            .into_iter()
            .find(|&child| self.template_slot(child).is_some()))
    }

    /// `slot_entry_name` plus the carrier's slot function spec. A dynamic
    /// name is a bare identifier the legacy walker spells `_ctx.<name>`
    /// before the scoped-param strip.
    pub(super) fn entry(&self, template: usize) -> Result<Entry> {
        let (element, content) = self
            .template_slot(template)
            .ok_or(AdmissionFailure::Invalid(
                "createSlots entry is not a slot template",
            ))?;
        let name = match &content.name {
            None => quoted_js_string("default"),
            Some(DynamicName::Static(name)) => quoted_js_string(name),
            Some(DynamicName::Dynamic(expr)) => {
                let source = expr.source();
                if !is_valid_js_identifier(source)
                    || matches!(source, "true" | "false" | "null" | "undefined")
                {
                    return Err(LegacyReason::Operation.into());
                }
                strip_scope_prefixes_for_scoped_params(&self.scoped_params, &cstr!("_ctx.{source}"))
            }
        };
        let pattern = slot_pattern(self.ctx.source, content)?;
        let inner = template + 1 + element.attributes.len() + element.bindings.len();
        let ranges = self
            .direct_children(inner)?
            .into_iter()
            .map(|start| self.child_end(start).map(|end| (start, end)))
            .collect::<Result<_>>()?;
        Ok(Entry {
            name,
            spec: SlotSpec {
                name: String::default(),
                pattern,
                ranges,
            },
        })
    }

    /// The branch operands of the `v-if` at `at`, zipped with their starts.
    pub(super) fn conditional(
        &self,
        at: usize,
    ) -> Result<(&'r s2::IfOp<'a>, std::vec::Vec<usize>)> {
        let Source::If(if_op) = self.segments[at].source else {
            return Err(AdmissionFailure::Invalid(
                "createSlots conditional is not a v-if",
            ));
        };
        let starts = self.branch_starts(at)?;
        if starts.len() != if_op.branches.len() {
            return Err(AdmissionFailure::Invalid(
                "string plan v-if branches do not match the op",
            ));
        }
        Ok((if_op, starts))
    }

    /// Enter the aliases of the `v-for` at `at`; returns the rendered
    /// source, alias list, and the scope mark to leave.
    pub(super) fn enter_loop(
        &mut self,
        at: usize,
    ) -> Result<(String, String, vize_s1_to_s2::TransformScopeMark)> {
        let Source::For(for_op) = self.segments[at].source else {
            return Err(AdmissionFailure::Invalid("createSlots loop is not a v-for"));
        };
        let binding = &for_op.binding;
        let source = self.expr(&binding.source, TransformContent::Decoded)?;
        let value = alias(&binding.value)?;
        let key = binding.key.as_ref().map(alias).transpose()?;
        let index = binding.index.as_ref().map(alias).transpose()?;
        let aliases = [Some(value), key, index];
        let list = aliases.into_iter().flatten().collect::<std::vec::Vec<_>>();
        let mut params = FxHashSet::default();
        for alias in &list {
            extract_destructure_params(alias.trim(), &mut params);
        }
        let mark = self.exprs.enter_for(aliases);
        self.scoped_params.push(params);
        Ok((source, list.join(", ").to_compact_string(), mark))
    }

    pub(super) fn leave_loop(&mut self, mark: vize_s1_to_s2::TransformScopeMark) {
        self.scoped_params.pop();
        self.exprs.leave(mark);
    }

    /// `_createSlots({ default, named, _: 2 }, [entries])`, written in place.
    pub(super) fn emit_create_slots(&mut self, slots: &ComponentSlots) -> Result<()> {
        self.ctx.use_core_helper(RuntimeHelper::CreateSlots);
        self.ctx.use_core_helper(RuntimeHelper::WithCtx);
        self.ctx.push("_createSlots({\n");
        self.ctx.indent_level += 1;
        if !slots.default.is_empty() {
            self.slot_property(&SlotSpec {
                name: "default".to_compact_string(),
                pattern: None,
                ranges: slots.default.clone(),
            })?;
        }
        for named in &slots.named {
            self.slot_property(named)?;
        }
        self.ctx.push_indent();
        self.ctx.push("_: 2 /* DYNAMIC */\n");
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("}, [\n");
        self.ctx.indent_level += 1;
        for (index, dynamic) in slots.dynamic.iter().enumerate() {
            if index > 0 {
                self.ctx.push(",\n");
            }
            self.ctx.push_indent();
            match *dynamic {
                Dynamic::Conditional(at) => self.conditional_entry(at)?,
                Dynamic::Looped(at) => self.looped_entry(at)?,
                Dynamic::Named(at) => {
                    let entry = self.entry(at)?;
                    self.entry_object(&entry, None)?;
                }
            }
        }
        self.ctx.indent_level -= 1;
        if !slots.dynamic.is_empty() {
            self.ctx.push("\n");
            self.ctx.push_indent();
        }
        self.ctx.push("])");
        Ok(())
    }

    /// `(cond) ? { ..., key: "N" } : ... : undefined`.
    fn conditional_entry(&mut self, at: usize) -> Result<()> {
        let (if_op, starts) = self.conditional(at)?;
        for (index, (branch, start)) in if_op.branches.iter().zip(starts).enumerate() {
            if index > 0 {
                self.ctx.push(" : ");
            }
            if let Some(condition) = &branch.condition {
                let condition = self.expr(condition, TransformContent::Decoded)?;
                self.ctx.push("(");
                self.ctx.push(&condition);
                self.ctx.push(") ? ");
            }
            match self.slot_template_in(start)? {
                Some(template) => {
                    let entry = self.entry(template)?;
                    self.entry_object(&entry, Some(index))?;
                }
                None => self.ctx.push("undefined"),
            }
        }
        if if_op
            .branches
            .last()
            .is_none_or(|branch| branch.condition.is_some())
        {
            self.ctx.push(" : undefined");
        }
        Ok(())
    }

    /// `_renderList(source, (aliases) => { return { ... } })`.
    fn looped_entry(&mut self, at: usize) -> Result<()> {
        let template = self
            .slot_template_in(at + 1)?
            .ok_or(AdmissionFailure::Invalid(
                "createSlots loop lost its slot template",
            ))?;
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderList);
        let (source, aliases, mark) = self.enter_loop(at)?;
        self.ctx.push("_renderList(");
        self.ctx.push(&source);
        self.ctx.push(", (");
        self.ctx.push(&aliases);
        self.ctx.push(") => {\n");
        self.ctx.indent_level += 1;
        self.ctx.push_indent();
        self.ctx.push("return ");
        let written = self
            .entry(template)
            .and_then(|entry| self.entry_object(&entry, None));
        self.leave_loop(mark);
        written?;
        self.ctx.push("\n");
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("})");
        Ok(())
    }

    /// `{ name, fn[, key] }` across indented lines.
    fn entry_object(&mut self, entry: &Entry, key: Option<usize>) -> Result<()> {
        self.ctx.push("{\n");
        self.ctx.indent_level += 1;
        self.ctx.push_indent();
        self.ctx.push("name: ");
        self.ctx.push(&entry.name);
        self.ctx.push(",\n");
        self.ctx.push_indent();
        self.ctx.push("fn: ");
        self.slot_fn(&entry.spec)?;
        if let Some(key) = key {
            self.ctx.push(",\n");
            self.ctx.push_indent();
            self.ctx.push(&cstr!("key: \"{key}\""));
        }
        self.ctx.push("\n");
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("}");
        Ok(())
    }
}
