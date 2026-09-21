//! The VNode fallback's `createSlots` expression: the same entries as the
//! push form, with `_withCtx((params) => [children])` slot functions.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, cstr};
use vize_s1_to_s2::TransformContent;

use super::create_slots::{Dynamic, Entry};
use super::slots::ComponentSlots;
use super::{Emitter, Result};
use crate::s4::AdmissionFailure;

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    /// The VNode fallback's `_createSlots({ ... }, [ ... ])` expression.
    pub(super) fn vnode_create_slots(&mut self, slots: &ComponentSlots) -> Result<String> {
        self.ctx.use_core_helper(RuntimeHelper::CreateSlots);
        self.ctx.use_core_helper(RuntimeHelper::WithCtx);
        let mut out = String::from("_createSlots({ ");
        if !slots.default.is_empty() {
            out.push_str("default: _withCtx(() => ");
            out.push_str(&self.vnode_list(&slots.default)?);
            out.push_str("), ");
        }
        for named in &slots.named {
            out.push_str(&self.vnode_slot_entry(named)?);
            out.push_str(", ");
        }
        out.push_str("_: 2 /* DYNAMIC */ }, [");
        for (index, dynamic) in slots.dynamic.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let entry = match *dynamic {
                Dynamic::Conditional(at) => self.vnode_conditional_entry(at)?,
                Dynamic::Looped(at) => self.vnode_looped_entry(at)?,
                Dynamic::Named(at) => {
                    let entry = self.entry(at)?;
                    self.vnode_entry_object(&entry, None)?
                }
            };
            out.push_str(&entry);
        }
        out.push_str("])");
        Ok(out)
    }

    fn vnode_conditional_entry(&mut self, at: usize) -> Result<String> {
        let (if_op, starts) = self.conditional(at)?;
        let mut out = String::default();
        for (index, (branch, start)) in if_op.branches.iter().zip(starts).enumerate() {
            if index > 0 {
                out.push_str(" : ");
            }
            if let Some(condition) = &branch.condition {
                out.push('(');
                out.push_str(&self.expr(condition, TransformContent::Decoded)?);
                out.push_str(") ? ");
            }
            match self.slot_template_in(start)? {
                Some(template) => {
                    let entry = self.entry(template)?;
                    out.push_str(&self.vnode_entry_object(&entry, Some(index))?);
                }
                None => out.push_str("undefined"),
            }
        }
        if if_op
            .branches
            .last()
            .is_none_or(|branch| branch.condition.is_some())
        {
            out.push_str(" : undefined");
        }
        Ok(out)
    }

    fn vnode_looped_entry(&mut self, at: usize) -> Result<String> {
        let template = self
            .slot_template_in(at + 1)?
            .ok_or(AdmissionFailure::Invalid(
                "createSlots loop lost its slot template",
            ))?;
        self.ctx.use_core_helper(RuntimeHelper::RenderList);
        let (source, aliases, mark) = self.enter_loop(at)?;
        let entry = self
            .entry(template)
            .and_then(|entry| self.vnode_entry_object(&entry, None));
        self.leave_loop(mark);
        Ok(cstr!(
            "_renderList({source}, ({aliases}) => {{ return {} }})",
            entry?
        ))
    }

    /// `{ name: X, fn: _withCtx((params) => [children])[, key: "N"] }`.
    fn vnode_entry_object(&mut self, entry: &Entry, key: Option<usize>) -> Result<String> {
        let mut out = cstr!(
            "{{ name: {}, fn: {}",
            entry.name,
            self.vnode_slot_fn(&entry.spec)?
        );
        if let Some(key) = key {
            out.push_str(&cstr!(", key: \"{key}\""));
        }
        out.push_str(" }");
        Ok(out)
    }
}
