//! The first authored expression that the inline transform wraps in `_unref`.
//!
//! Slot objects print named templates ahead of earlier default content, so
//! the emit walk meets that wrapper later than the transform does. The
//! preference walk is source order, and its visit numbers are what
//! `prefer_at_visit` sorts by.

use alloc::vec::Vec as StdVec;

use oxc_ast::ast as js;
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{walk_arrow_function_expression, walk_function};
use oxc_syntax::scope::ScopeFlags;
use vize_s2::expr::ExprRef;
use vize_s2::op::{BindingOp, DynamicName, Op};

use super::super::options::{BindingKind, BindingTable};
use super::PreferCx;

pub(super) fn note_op(cx: &PreferCx<'_>, op: &Op<'_>, visit: u32) {
    if cx.authored_unref.get() != u32::MAX || !cx.inline {
        return;
    }
    let Some(table) = cx.bindings else {
        return;
    };
    if op_needs_unref(op, table) {
        cx.authored_unref.set(visit);
    }
}

fn op_needs_unref(op: &Op<'_>, table: &BindingTable) -> bool {
    match op {
        Op::Interpolation(text) => expression_needs_unref(&text.expression, table),
        Op::If(if_op) => if_op.branches.iter().any(|branch| {
            branch
                .condition
                .as_ref()
                .is_some_and(|condition| expression_needs_unref(condition, table))
        }),
        Op::For(for_op) => expression_needs_unref(&for_op.binding.source, table),
        Op::Element(element) => bindings_need_unref(&element.bindings, table),
        Op::Component(component) => bindings_need_unref(&component.bindings, table),
        Op::Slot(slot) => bindings_need_unref(&slot.bindings, table),
        Op::Text(_) | Op::Comment(_) => false,
    }
}

fn bindings_need_unref(bindings: &[BindingOp<'_>], table: &BindingTable) -> bool {
    bindings.iter().any(|binding| match binding {
        BindingOp::Bind(bind) => {
            dynamic_needs_unref(bind.name.as_ref(), table)
                || bind
                    .value
                    .as_ref()
                    .is_some_and(|value| expression_needs_unref(value, table))
        }
        BindingOp::On(on) => {
            dynamic_needs_unref(on.name.as_ref(), table)
                || on
                    .handler
                    .as_ref()
                    .is_some_and(|handler| expression_needs_unref(handler, table))
        }
        BindingOp::VueDirective(directive) => {
            dynamic_needs_unref(directive.argument.as_ref(), table)
                || directive
                    .value
                    .as_ref()
                    .is_some_and(|value| expression_needs_unref(value, table))
        }
        BindingOp::VueShow(show) => expression_needs_unref(&show.value, table),
        BindingOp::VueHtml(html) => html
            .value
            .as_ref()
            .is_some_and(|value| expression_needs_unref(value, table)),
        BindingOp::VueText(text) => text
            .value
            .as_ref()
            .is_some_and(|value| expression_needs_unref(value, table)),
        BindingOp::VueMemo(memo) => expression_needs_unref(&memo.value, table),
        BindingOp::Model(_)
        | BindingOp::SlotContent(_)
        | BindingOp::VueCssBind(_)
        | BindingOp::VueSync(_)
        | BindingOp::VueSlotScope(_)
        | BindingOp::VueOnce(_)
        | BindingOp::VueCloak(_) => false,
    })
}

fn dynamic_needs_unref(name: Option<&DynamicName<'_>>, table: &BindingTable) -> bool {
    match name {
        Some(DynamicName::Dynamic(expr)) => expression_needs_unref(expr, table),
        Some(DynamicName::Static(_)) | None => false,
    }
}

fn expression_needs_unref(expr: &ExprRef<'_>, table: &BindingTable) -> bool {
    match expr {
        ExprRef::Js(js) => {
            let mut walk = UnrefWalk {
                table,
                locals: StdVec::new(),
                needs: false,
            };
            walk.visit_expression(js.ast);
            walk.needs
        }
        ExprRef::Opaque(opaque) => name_needs_unref(table, opaque.source.trim()),
        ExprRef::Foreign(_) | ExprRef::Filter(_) => false,
    }
}

fn name_needs_unref(table: &BindingTable, name: &str) -> bool {
    matches!(
        table.kind(name),
        Some(BindingKind::SetupLet | BindingKind::SetupMaybeRef)
    ) && table.reactive_read(name).is_none()
}

struct UnrefWalk<'a> {
    table: &'a BindingTable,
    locals: StdVec<StdVec<vize_s0::String>>,
    needs: bool,
}

impl UnrefWalk<'_> {
    fn is_local(&self, name: &str) -> bool {
        self.locals
            .iter()
            .any(|frame| frame.iter().any(|bound| bound.as_str() == name))
    }
}

impl<'a> Visit<'a> for UnrefWalk<'_> {
    fn visit_identifier_reference(&mut self, ident: &js::IdentifierReference<'a>) {
        if self.needs {
            return;
        }
        let name = ident.name.as_str();
        if !self.is_local(name) && name_needs_unref(self.table, name) {
            self.needs = true;
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &js::ArrowFunctionExpression<'a>) {
        self.locals.push(StdVec::new());
        for param in &arrow.params.items {
            if let js::BindingPattern::BindingIdentifier(ident) = &param.pattern
                && let Some(frame) = self.locals.last_mut()
            {
                frame.push(vize_s0::String::from(ident.name.as_str()));
            }
        }
        walk_arrow_function_expression(self, arrow);
        self.locals.pop();
    }

    fn visit_function(&mut self, function: &js::Function<'a>, flags: ScopeFlags) {
        self.locals.push(StdVec::new());
        for param in &function.params.items {
            if let js::BindingPattern::BindingIdentifier(ident) = &param.pattern
                && let Some(frame) = self.locals.last_mut()
            {
                frame.push(vize_s0::String::from(ident.name.as_str()));
            }
        }
        walk_function(self, function, flags);
        self.locals.pop();
    }
}
