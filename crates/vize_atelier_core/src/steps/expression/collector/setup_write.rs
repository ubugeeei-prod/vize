//! Original-AST mutable setup writes within the existing identifier walk.

use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, IdentifierReference, SimpleAssignmentTarget,
    UpdateExpression,
};
use oxc_span::GetSpan;
use vize_l0::{String, cstr};

use super::IdentifierCollector;

impl IdentifierCollector<'_, '_> {
    fn guards_setup_write(&self, name: &str) -> bool {
        !self.is_local(name)
            && !self.ctx.is_in_scope(name)
            && self.ctx.options.inline
            && !self.ctx.options.ssr
            && !self.ctx.options.vapor
            && self
                .ctx
                .options
                .binding_metadata
                .as_ref()
                .is_some_and(|table| {
                    table.is_script_setup
                        && table.bindings.get(name) == Some(&crate::options::BindingType::SetupLet)
                })
    }

    pub(super) fn guard_setup_assignment(&mut self, expr: &AssignmentExpression<'_>) {
        let AssignmentTarget::AssignmentTargetIdentifier(ident) = &expr.left else {
            return;
        };
        if !self.guards_setup_write(ident.name.as_str()) {
            return;
        }
        let start = expr.span.start as usize;
        let right = expr.right.span();
        let right_start = right.start as usize;
        let end = expr.span.end as usize;
        let Some(head) = self.source.get(start..right_start) else {
            return;
        };
        let Some(source) = self.source.get(right_start..end) else {
            return;
        };
        // Child visits already produced every RHS edit. Reuse those exact
        // original coordinates; no second AST walk or parse is introduced.
        let prefixes = self
            .rewrites
            .iter()
            .filter(|(at, _)| *at >= right_start && *at <= end)
            .map(|(at, text)| (*at - right_start, text.clone()))
            .collect();
        let suffixes = self
            .suffix_rewrites
            .iter()
            .filter(|(at, _)| *at >= right_start && *at <= end)
            .map(|(at, text)| (*at - right_start, text.clone()))
            .collect();
        let right = super::super::splice::splice_insertions(source, prefixes, suffixes, 0);
        let plain = cstr!("{head}{right}");
        self.finish_setup_write(expr.span.start, expr.span.end, ident, plain);
    }

    pub(super) fn guard_setup_update(&mut self, expr: &UpdateExpression<'_>) {
        let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = &expr.argument else {
            return;
        };
        if !self.guards_setup_write(ident.name.as_str()) {
            return;
        }
        let Some(plain) = self
            .source
            .get(expr.span.start as usize..expr.span.end as usize)
        else {
            return;
        };
        let plain = String::from(plain);
        self.finish_setup_write(expr.span.start, expr.span.end, ident, plain);
    }

    fn finish_setup_write(
        &mut self,
        start: u32,
        end: u32,
        ident: &IdentifierReference<'_>,
        plain: String,
    ) {
        let target_end = ident.span.end as usize;
        // Replace only this original target's legacy suffix. Putting .value
        // inside any authored target parentheses preserves both update forms.
        let scanned = target_end
            + self
                .source
                .as_bytes()
                .get(target_end..)
                .unwrap_or_default()
                .iter()
                .take_while(|byte| **byte == b')')
                .count();
        self.suffix_rewrites
            .retain(|(at, text)| *at != scanned || text.as_str() != ".value");
        self.suffix_rewrites
            .push((target_end, String::from(".value")));
        self.rewrites
            .insert((start as usize, cstr!("(_isRef({}) ? ", ident.name.as_str())));
        let end = end as usize;
        // The splicer reverses construction order at one position. An outer
        // tail must therefore precede existing child tails in this list.
        let position = self
            .suffix_rewrites
            .iter()
            .position(|(at, _)| *at == end)
            .unwrap_or(self.suffix_rewrites.len());
        self.suffix_rewrites
            .insert(position, (end, cstr!(" : {plain})")));
        self.used_is_ref = true;
    }
}

#[cfg(test)]
mod tests;
