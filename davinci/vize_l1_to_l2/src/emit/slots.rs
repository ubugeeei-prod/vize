//! Slot objects (`withCtx` / `_: 1|2`) from [`SlotFacts`].
//!
//! Implicit default, named `<template>` groups, and component-root `v-slot`.
//! Conditional / looped slot templates go through [`super::create_slots`].
//! Outlets live in [`super::outlet`]. A `v-slots` spread is the children
//! argument when it is the only slot source, or `...expr` closes a slot object.

mod capture;
mod group;
mod params;

use alloc::vec::Vec as StdVec;

use oxc_ast::ast as js;
use oxc_ast_visit::{
    Visit,
    walk::{walk_arrow_function_expression, walk_function},
};
use oxc_syntax::scope::ScopeFlags;
use vize_l0::{String, ensure_sufficient_stack};
use vize_l2::expr::ExprRef;
use vize_l2::op::{BindingOp, DynamicName, Op, Region};
use vize_l2::scope::ScopeOrigin;

pub(super) use self::capture::{
    SlotPiece, capture, capture_child, emit_template_pieces, is_slot_template, is_whitespace_text,
};
use self::params::{emit_slot_key, emit_slot_params, group_slot_content};
use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::js::{RawJs, expr_source};
use super::prefix::PrefixScope;
use crate::pass::{SlotCarrier, SlotFacts, SlotName};

/// First `v-slots="expr"` (no argument) on the component, matching
/// `codegen/slots/detect.rs`. An argument spelling is a different
/// construct and stays unsupported.
pub(super) fn slots_spread<'a>(
    bindings: &'a [BindingOp<'a>],
) -> Result<Option<RawJs<'a>>, EmitError> {
    for binding in bindings.iter() {
        if let BindingOp::SlotsObject(object) = binding {
            return expr_source(&object.value, false).map(Some).ok_or_else(|| {
                EmitError::unsupported_at(Reason::SlotsSpreadValueNotJs, object.value.span())
            });
        }
        let BindingOp::VueDirective(directive) = binding else {
            continue;
        };
        if directive.name != "slots" {
            continue;
        }
        if directive.argument.is_some() || !directive.modifiers.is_empty() {
            return Err(EmitError::unsupported_at(
                Reason::SlotsSpreadShape,
                directive.span,
            ));
        }
        return match directive.value {
            Some(expr) => expr_source(&expr, false).map(Some).ok_or_else(|| {
                EmitError::unsupported_at(Reason::SlotsSpreadValueNotJs, expr.span())
            }),
            None => Err(EmitError::unsupported_at(
                Reason::SlotsSpreadValueNotJs,
                directive.span,
            )),
        };
    }
    Ok(None)
}

pub(super) fn is_slots_spread(binding: &BindingOp<'_>) -> bool {
    matches!(binding, BindingOp::SlotsObject(_))
        || matches!(binding, BindingOp::VueDirective(directive) if directive.name == "slots")
}

pub(super) fn has_implicit_default(children: &Region<'_>) -> bool {
    children.ops.iter().any(|op| !is_whitespace_text(op))
}

pub(super) fn filler_default_needs_props_placeholder(children: &Region<'_>) -> bool {
    let mut needs_placeholder = false;
    for op in children.ops.iter() {
        let Op::Text(text) = op else {
            return false;
        };
        if !crate::lower::legacy_slot_filler_text(text.content) {
            return false;
        }
        needs_placeholder |= crate::lower::legacy_slot_filler_needs_props_placeholder(text.content);
    }
    needs_placeholder
}

pub(super) fn has_text_only_implicit_default(children: &Region<'_>) -> bool {
    let mut has_content = false;
    for op in children.ops.iter() {
        if is_whitespace_text(op) {
            continue;
        }
        has_content = true;
        if !matches!(op, Op::Text(_) | Op::Interpolation(_)) {
            return false;
        }
    }
    has_content
}

pub(super) fn has_dynamic_names(facts: &SlotFacts) -> bool {
    facts
        .groups
        .iter()
        .any(|g| matches!(g.name, SlotName::Dynamic { .. }))
}

/// Whether a component's slot functions read an active template-scope binding.
/// Such closures must be replaced when their parent slot is invoked again even
/// when the component's ordinary props compare equal (#10169).
pub(super) fn references_current_scope(cx: &EmitCx<'_>, children: &Region<'_>) -> bool {
    cx.scope.has_codegen_scope_params() && region_references_scope(&cx.scope, children)
}

fn region_references_scope(scope: &PrefixScope<'_>, region: &Region<'_>) -> bool {
    region.ops.iter().any(|op| {
        ensure_sufficient_stack(|| match op {
            Op::Element(element) => {
                bindings_reference_scope(scope, &element.bindings)
                    || region_references_scope(scope, &element.children)
            }
            Op::Component(component) => {
                bindings_reference_scope(scope, &component.bindings)
                    || region_references_scope(scope, &component.children)
            }
            Op::Interpolation(interpolation) => {
                expression_references_scope(scope, interpolation.expression)
            }
            Op::If(if_op) => if_op.branches.iter().any(|branch| {
                branch
                    .condition
                    .is_some_and(|condition| expression_references_scope(scope, condition))
                    || region_references_scope(scope, &branch.region)
            }),
            Op::For(for_op) => {
                expression_references_scope(scope, for_op.binding.source)
                    || region_references_scope(scope, &for_op.region)
            }
            Op::OriginalFor(for_op) => region_references_scope(scope, &for_op.region),
            Op::Slot(slot) => {
                dynamic_name_references_scope(scope, &slot.name)
                    || bindings_reference_scope(scope, &slot.bindings)
                    || region_references_scope(scope, &slot.fallback)
            }
            Op::Text(_) | Op::Comment(_) => false,
        })
    })
}

fn bindings_reference_scope(scope: &PrefixScope<'_>, bindings: &[BindingOp<'_>]) -> bool {
    bindings.iter().any(|binding| match binding {
        BindingOp::Bind(bind) => {
            bind.name
                .as_ref()
                .is_some_and(|name| dynamic_name_references_scope(scope, name))
                || bind
                    .value
                    .is_some_and(|value| expression_references_scope(scope, value))
        }
        BindingOp::On(on) => {
            on.name
                .as_ref()
                .is_some_and(|name| dynamic_name_references_scope(scope, name))
                || on
                    .expression()
                    .is_some_and(|expression| expression_references_scope(scope, expression))
        }
        BindingOp::Model(model) => {
            expression_references_scope(scope, model.contract.read)
                || expression_references_scope(scope, model.contract.write)
                || model
                    .argument
                    .as_ref()
                    .is_some_and(|name| dynamic_name_references_scope(scope, name))
        }
        BindingOp::SlotContent(slot) => slot
            .name
            .as_ref()
            .is_some_and(|name| dynamic_name_references_scope(scope, name)),
        BindingOp::SlotsObject(object) => expression_references_scope(scope, object.value),
        BindingOp::VueDirective(directive) => {
            directive
                .argument
                .as_ref()
                .is_some_and(|name| dynamic_name_references_scope(scope, name))
                || directive
                    .value
                    .is_some_and(|value| expression_references_scope(scope, value))
        }
        BindingOp::VueCssBind(bind) => expression_references_scope(scope, bind.value),
        BindingOp::VueSync(sync) => expression_references_scope(scope, sync.value),
        // Both spellings declare their value rather than read it.
        BindingOp::VueSlotScope(_) => false,
        BindingOp::VueMemo(memo) => expression_references_scope(scope, memo.value),
        BindingOp::VueShow(show) => expression_references_scope(scope, show.value),
        BindingOp::VueHtml(html) => html
            .value
            .is_some_and(|value| expression_references_scope(scope, value)),
        BindingOp::VueText(text) => text
            .value
            .is_some_and(|value| expression_references_scope(scope, value)),
        BindingOp::VueOnce(_) | BindingOp::VueCloak(_) => false,
    })
}

fn dynamic_name_references_scope(scope: &PrefixScope<'_>, name: &DynamicName<'_>) -> bool {
    match name {
        DynamicName::Static(_) => false,
        DynamicName::Dynamic(expression) => expression_references_scope(scope, *expression),
    }
}

fn expression_references_scope(scope: &PrefixScope<'_>, expression: ExprRef<'_>) -> bool {
    let ExprRef::Js(expression) = expression else {
        // Opaque and foreign expressions cannot prove their binding graph. A
        // token-bounded hit is the safe answer: extra updates preserve output.
        return contains_codegen_scope_identifier(scope, expression.source());
    };
    let mut visitor = ScopeReferenceVisitor::new(scope);
    visitor.visit_expression(expression.ast);
    visitor.found
}

fn contains_codegen_scope_identifier(scope: &PrefixScope<'_>, content: &str) -> bool {
    content
        .split(|character: char| {
            !character.is_ascii_alphanumeric() && character != '_' && character != '$'
        })
        .any(|name| !name.is_empty() && scope.is_codegen_scope_param(name))
}

struct ScopeReferenceVisitor<'s> {
    scope: &'s PrefixScope<'s>,
    locals: StdVec<StdVec<String>>,
    found: bool,
}

impl<'s> ScopeReferenceVisitor<'s> {
    fn new(scope: &'s PrefixScope<'s>) -> Self {
        Self {
            scope,
            locals: StdVec::new(),
            found: false,
        }
    }

    fn is_local(&self, name: &str) -> bool {
        self.locals
            .iter()
            .rev()
            .any(|frame| frame.iter().any(|bound| bound.as_str() == name))
    }

    fn bind_pattern(&mut self, pattern: &js::BindingPattern<'_>) {
        match pattern {
            js::BindingPattern::BindingIdentifier(ident) => {
                if let Some(frame) = self.locals.last_mut() {
                    frame.push(String::from(ident.name.as_str()));
                }
            }
            js::BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.bind_pattern(&property.value);
                }
                if let Some(rest) = &object.rest {
                    self.bind_pattern(&rest.argument);
                }
            }
            js::BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.bind_pattern(element);
                }
                if let Some(rest) = &array.rest {
                    self.bind_pattern(&rest.argument);
                }
            }
            js::BindingPattern::AssignmentPattern(assignment) => {
                self.bind_pattern(&assignment.left);
            }
        }
    }
}

impl<'a> Visit<'a> for ScopeReferenceVisitor<'_> {
    fn visit_identifier_reference(&mut self, ident: &js::IdentifierReference<'a>) {
        let name = ident.name.as_str();
        if !self.is_local(name) && self.scope.is_codegen_scope_param(name) {
            self.found = true;
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &js::ArrowFunctionExpression<'a>) {
        self.locals.push(StdVec::new());
        for param in &arrow.params.items {
            self.bind_pattern(&param.pattern);
        }
        walk_arrow_function_expression(self, arrow);
        self.locals.pop();
    }

    fn visit_function(&mut self, function: &js::Function<'a>, flags: ScopeFlags) {
        self.locals.push(StdVec::new());
        for param in &function.params.items {
            self.bind_pattern(&param.pattern);
        }
        walk_function(self, function, flags);
        self.locals.pop();
    }

    fn visit_variable_declarator(&mut self, declarator: &js::VariableDeclarator<'a>) {
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
        self.bind_pattern(&declarator.id);
    }
}

pub(super) fn admit_default(children: &Region<'_>) -> Result<(), EmitError> {
    walk_admit(children)
}

fn walk_admit(region: &Region<'_>) -> Result<(), EmitError> {
    for op in region.ops.iter() {
        match op {
            Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => {}
            Op::Element(element) if is_slot_template(element) => {
                if element.bindings.iter().any(|b| !is_inert_slot_binding(b)) {
                    return Err(EmitError::unsupported_at(
                        Reason::SlotDefaultShape,
                        element.span,
                    ));
                }
                walk_admit(&element.children)?;
            }
            Op::Element(element) => walk_admit(&element.children)?,
            Op::Component(component) => walk_admit(&component.children)?,
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    walk_admit(&branch.region)?;
                }
            }
            Op::For(for_op) => walk_admit(&for_op.region)?,
            Op::OriginalFor(for_op) => walk_admit(&for_op.region)?,
            Op::Slot(slot) => walk_admit(&slot.fallback)?,
        }
    }
    Ok(())
}

fn is_inert_slot_binding(binding: &BindingOp<'_>) -> bool {
    match binding {
        BindingOp::SlotContent(_)
        | BindingOp::VueOnce(_)
        | BindingOp::VueMemo(_)
        | BindingOp::VueCloak(_) => true,
        BindingOp::Bind(bind)
            if matches!(bind.name, Some(DynamicName::Static("key")))
                && bind.modifiers.is_empty()
                && matches!(bind.value, Some(ExprRef::Js(_))) =>
        {
            true
        }
        _ => false,
    }
}

pub(super) fn emit_slots(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    facts: &SlotFacts,
    spread: Option<&RawJs<'_>>,
    component_bindings: &[BindingOp<'_>],
    captures_scope: bool,
) -> Result<(), EmitError> {
    if let Some(group) = facts.groups.iter().find(|group| group.name.text() == "_") {
        if let SlotName::Static {
            origin: ScopeOrigin::Authored { span },
            ..
        } = &group.name
        {
            return Err(EmitError::unsupported_at(Reason::SlotNameUnderscore, *span));
        }
        return Err(EmitError::unsupported(Reason::SlotNameUnderscore));
    }
    cx.buf.use_with_ctx();
    cx.buf.indent();
    cx.buf.indent();
    let mut buckets: StdVec<StdVec<SlotPiece>> =
        facts.groups.iter().map(|_| StdVec::new()).collect();
    let start_walk = cx.walk.clone();
    group::collect_pieces(cx, children, facts, &mut buckets)?;
    cx.buf.deindent();
    cx.buf.deindent();
    cx.buf.push("{");
    cx.buf.indent();
    for (group, bucket) in facts.groups.iter().zip(&buckets) {
        if bucket.is_empty() && matches!(group.carrier, SlotCarrier::Implicit) {
            continue;
        }
        cx.buf.newline();
        emit_slot_key(cx, &group.name);
        cx.buf.push(": ");
        cx.buf.push(Buf::with_ctx_alias());
        cx.buf.push("(");
        let content = group_slot_content(&start_walk, children, component_bindings, group.carrier);
        emit_slot_params(cx, &group.params, content)?;
        cx.buf.push(" => [");
        cx.buf.indent();
        for (j, piece) in bucket.iter().enumerate() {
            if j > 0 {
                cx.buf.push(",");
            }
            cx.buf.newline();
            cx.push_captured(piece);
        }
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("]),");
    }
    cx.buf.newline();
    if let Some(spread) = spread {
        cx.buf.push("...");
        cx.buf.push(spread.as_str());
    } else {
        let forwarded = super::outlet::has_forwarded_outlet(children);
        if forwarded && cx.slot_param_depth == 0 && !cx.in_v_for {
            cx.buf.push("_: 3 /* FORWARDED */");
        } else if cx.in_v_for
            || has_dynamic_names(facts)
            || captures_scope
            || (forwarded && cx.slot_param_depth > 0)
        {
            cx.buf.push("_: 2 /* DYNAMIC */");
        } else {
            cx.buf.push("_: 1 /* STABLE */");
        }
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("}");
    Ok(())
}
