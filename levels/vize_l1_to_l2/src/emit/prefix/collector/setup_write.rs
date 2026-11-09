//! Original-AST mutable setup writes within the existing identifier walk.

use alloc::vec::Vec as StdVec;
use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, IdentifierReference, SimpleAssignmentTarget,
    UpdateExpression,
};
use oxc_span::GetSpan;
use vize_l0::{String, cstr};

use super::IdentifierCollector;
use crate::emit::options::BindingKind;

impl IdentifierCollector<'_, '_> {
    fn guarded_setup_write_kind(&self, name: &str) -> Option<BindingKind> {
        if self.is_local(name) {
            return None;
        }
        self.scope.guarded_setup_write_kind(name)
    }

    pub(super) fn guard_setup_assignment(&mut self, expr: &AssignmentExpression<'_>) {
        let AssignmentTarget::AssignmentTargetIdentifier(ident) = &expr.left else {
            return;
        };
        let Some(kind) = self.guarded_setup_write_kind(ident.name.as_str()) else {
            return;
        };
        let start = self.at(expr.span.start);
        let right = expr.right.span();
        let right_start = self.at(right.start);
        let end = self.at(expr.span.end);
        let Some(head) = self.source.get(start..right_start) else {
            return;
        };
        let Some(source) = self.source.get(right_start..end) else {
            return;
        };
        // Child visits already produced every RHS edit. Reuse those exact
        // original coordinates; no second AST walk or parse is introduced.
        let prefixes: StdVec<_> = self
            .rewrites
            .iter()
            .filter(|(at, _)| *at >= right_start && *at <= end)
            .map(|(at, text)| (*at - right_start, text.clone()))
            .collect();
        let suffixes: StdVec<_> = self
            .suffix_rewrites
            .iter()
            .filter(|(at, _)| *at >= right_start && *at <= end)
            .map(|(at, text)| (*at - right_start, text.clone()))
            .collect();
        let right = super::super::splice::splice_insertions(source, prefixes, suffixes, 0);
        let plain = cstr!("{head}{right}");
        self.finish_setup_write(expr.span.start, expr.span.end, ident, kind, plain);
    }

    pub(super) fn guard_setup_update(&mut self, expr: &UpdateExpression<'_>) {
        let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = &expr.argument else {
            return;
        };
        let Some(kind) = self.guarded_setup_write_kind(ident.name.as_str()) else {
            return;
        };
        let Some(plain) = self
            .source
            .get(self.at(expr.span.start)..self.at(expr.span.end))
        else {
            return;
        };
        let plain = String::from(plain);
        self.finish_setup_write(expr.span.start, expr.span.end, ident, kind, plain);
    }

    fn finish_setup_write(
        &mut self,
        start: u32,
        end: u32,
        ident: &IdentifierReference<'_>,
        kind: BindingKind,
        plain: String,
    ) {
        let target_end = self.at(ident.span.end);
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
            .push((self.at(start), cstr!("(_isRef({}) ? ", ident.name.as_str())));
        let end = self.at(end);
        // The splicer reverses construction order at one position. An outer
        // tail must therefore precede existing child tails in this list.
        let position = self
            .suffix_rewrites
            .iter()
            .position(|(at, _)| *at == end)
            .unwrap_or(self.suffix_rewrites.len());
        let fallback = match kind {
            BindingKind::SetupLet => plain,
            BindingKind::SetupMaybeRef => String::from("null"),
            _ => return,
        };
        self.suffix_rewrites
            .insert(position, (end, cstr!(" : {fallback})")));
        self.used_is_ref = true;
    }
}
