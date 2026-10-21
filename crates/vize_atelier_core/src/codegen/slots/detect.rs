//! Slot detection predicates (which children form slots, dynamic/forwarded checks).

use crate::codegen::context::CodegenContext;
use crate::codegen::expression::scope_prefix::strip_scope_prefixes_for_slot_params;
use crate::steps::v_slot::{collect_slots, has_v_slot};
use crate::{
    CompoundExpressionChild, ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode,
    TextCallContent,
};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_l0::{FxHashSet, String, ensure_sufficient_stack};
use vize_relief::ExpressionScope;

/// The expression a component spreads into its slots object, if it carries a
/// `v-slots` directive (#3467).
///
/// `v-slots` is the `@vue/babel-plugin-jsx` spelling for slot forwarding:
/// `<B v-slots={slots}/>` hands `B` a slots object the compiler cannot see
/// inside. The JSX lowering keeps the object-literal form as synthetic
/// `<template v-slot:name>` children (#3418) and only reaches here for a value
/// that stays opaque at compile time.
pub(super) fn slots_spread<'a, 'b>(el: &'b ElementNode<'a>) -> Option<&'b ExpressionNode<'a>> {
    el.props.iter().find_map(|prop| {
        let PropNode::Directive(dir) = prop else {
            return None;
        };
        // `v-slots` takes no argument -- the slot names are the forwarded
        // object's own keys -- so an argument spelling is not this construct.
        if dir.name != "slots" || dir.arg.is_some() {
            return None;
        }
        dir.exp.as_ref()
    })
}

/// Whether the component's slots object is built **only** from a forwarded
/// `v-slots` value: no `v-slot` on the component root, no slot templates, and
/// no children to become the default slot.
///
/// `@vue/babel-plugin-jsx` passes the forwarded value straight through as the
/// children argument in that shape (`createVNode(B, null, slots)`) rather than
/// wrapping it in an object literal, and Vize matches it.
pub(super) fn slots_are_only_forwarded(el: &ElementNode<'_>) -> bool {
    slots_spread(el).is_some() && !has_authored_slots(el)
}

/// Whether the element contributes slots of its own: a `v-slot` on the
/// component root, or any child that is not whitespace filler.
fn has_authored_slots(el: &ElementNode<'_>) -> bool {
    if el
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "slot"))
    {
        return true;
    }
    el.children.iter().any(|child| match child {
        TemplateChildNode::Text(t) => !t.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => true,
        _ => true,
    })
}

/// Check if component has slot children that need to be generated as slots object
pub fn has_slot_children(el: &ElementNode<'_>) -> bool {
    // Teleport and KeepAlive consume raw children rather than a slot object.
    // KeepAlive still gets DYNAMIC_SLOTS at the vnode patch-flag layer.
    if matches!(el.tag, "Teleport" | "teleport" | "KeepAlive" | "keep-alive") {
        return false;
    }

    // A forwarded `v-slots` value is the component's slots even when it has no
    // children of its own (`<B v-slots={slots}/>`).
    if slots_spread(el).is_some() {
        return true;
    }

    if el.children.is_empty() {
        return false;
    }

    // Check for v-slot on component root
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop
            && dir.name == "slot"
        {
            return true;
        }
    }

    // Whitespace-only children do not create a slot. A comment-only child does
    // create a default slot in development builds, as it does in Vue.
    let has_meaningful_child = el.children.iter().any(|child| match child {
        TemplateChildNode::Text(t) => !t.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => true,
        _ => true,
    });
    if !has_meaningful_child {
        return false;
    }

    // Check for any children (default slot) or template slots
    true
}

/// KeepAlive always needs `DYNAMIC_SLOTS`. Other components need it when the
/// slot object itself can change (`v-for` parent, dynamic names, forwarded
/// outlets).
pub fn needs_dynamic_slots_patch(ctx: &CodegenContext, el: &ElementNode<'_>) -> bool {
    el.tag == "KeepAlive"
        || el.tag == "keep-alive"
        || ((ctx.in_v_for || ctx.in_match_scope) && has_slot_children(el))
        || has_dynamic_slots_flag(el, &ctx.source)
        || slots_reference_current_scope(ctx, el)
        || (ctx.has_slot_params() && has_forwarded_slot_outlet(el))
}

/// Whether a component's slot functions close over the current template scope.
///
/// A component nested in a scoped slot can otherwise look unchanged when its
/// ordinary props compare equal. Vue marks the slots object dynamic whenever a
/// slot expression references an active `v-slot`/`v-for` binding; this keeps the
/// child from retaining the previous render's closure (#10169).
#[cold]
#[inline(never)]
pub(super) fn slots_reference_current_scope(ctx: &CodegenContext, el: &ElementNode<'_>) -> bool {
    if !ctx.has_slot_params() || !has_slot_children(el) {
        return false;
    }

    let detector = ScopeReferenceDetector::new(ctx);
    el.props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(dir)
                if dir.name == "slot"
                    && (dir.arg.as_ref().is_some_and(|arg| detector.expression(arg))
                        || dir.exp.as_ref().is_some_and(|exp| detector.expression(exp)))
        )
    }) || el
        .children
        .iter()
        .any(|child| ensure_sufficient_stack(|| detector.child(child)))
}

struct ScopeReferenceDetector<'ctx> {
    ctx: &'ctx CodegenContext,
    allocator: oxc_allocator::Allocator,
}

impl<'ctx> ScopeReferenceDetector<'ctx> {
    fn new(ctx: &'ctx CodegenContext) -> Self {
        Self {
            ctx,
            allocator: oxc_allocator::Allocator::default(),
        }
    }

    fn expression(&self, expression: &ExpressionNode<'_>) -> bool {
        match expression {
            ExpressionNode::Simple(simple) => self.simple(simple.content, simple.is_static),
            ExpressionNode::Compound(compound) => self.compound(&compound.children),
        }
    }

    fn compound(&self, children: &[CompoundExpressionChild<'_>]) -> bool {
        children.iter().any(|child| match child {
            CompoundExpressionChild::Simple(simple) => {
                self.simple(simple.content, simple.is_static)
            }
            CompoundExpressionChild::Compound(compound) => self.compound(&compound.children),
            CompoundExpressionChild::Interpolation(interpolation) => {
                self.expression(&interpolation.content)
            }
            CompoundExpressionChild::Text(_)
            | CompoundExpressionChild::String(_)
            | CompoundExpressionChild::Symbol(_) => false,
        })
    }

    fn simple(&self, content: &str, is_static: bool) -> bool {
        if is_static
            || !self
                .ctx
                .slot_params
                .keys()
                .any(|name| contains_identifier(content, name))
        {
            return false;
        }

        let normalized = strip_scope_prefixes_for_slot_params(self.ctx, content);
        let mut wrapped = String::with_capacity(normalized.len() + 2);
        wrapped.push('(');
        wrapped.push_str(&normalized);
        wrapped.push(')');

        if let Ok(expression) = Parser::new(
            &self.allocator,
            &wrapped,
            SourceType::ts().with_module(true),
        )
        .parse_expression()
        {
            let mut visitor = ScopeReferenceVisitor::new(self.ctx);
            visitor.visit_expression(&expression);
            return visitor.found;
        }

        let result = Parser::new(
            &self.allocator,
            &normalized,
            SourceType::ts().with_module(true),
        )
        .parse();
        if !result.diagnostics.is_empty() {
            return false;
        }
        let mut visitor = ScopeReferenceVisitor::new(self.ctx);
        visitor.visit_program(&result.program);
        visitor.found
    }

    fn child(&self, child: &TemplateChildNode<'_>) -> bool {
        match child {
            TemplateChildNode::Element(element) => {
                element.props.iter().any(|prop| {
                    matches!(
                        prop,
                        PropNode::Directive(dir)
                            if dir.arg.as_ref().is_some_and(|arg| self.expression(arg))
                                || dir.exp.as_ref().is_some_and(|exp| self.expression(exp))
                    )
                }) || element
                    .children
                    .iter()
                    .any(|child| ensure_sufficient_stack(|| self.child(child)))
            }
            TemplateChildNode::Interpolation(interpolation) => {
                self.expression(&interpolation.content)
            }
            TemplateChildNode::If(if_node) => if_node.branches.iter().any(|branch| {
                branch
                    .condition
                    .as_ref()
                    .is_some_and(|condition| self.expression(condition))
                    || branch
                        .children
                        .iter()
                        .any(|child| ensure_sufficient_stack(|| self.child(child)))
            }),
            TemplateChildNode::IfBranch(branch) => {
                branch
                    .condition
                    .as_ref()
                    .is_some_and(|condition| self.expression(condition))
                    || branch
                        .children
                        .iter()
                        .any(|child| ensure_sufficient_stack(|| self.child(child)))
            }
            TemplateChildNode::For(for_node) => {
                self.expression(&for_node.source)
                    || for_node
                        .children
                        .iter()
                        .any(|child| ensure_sufficient_stack(|| self.child(child)))
            }
            TemplateChildNode::TextCall(call) => match &call.content {
                TextCallContent::Interpolation(interpolation) => {
                    self.expression(&interpolation.content)
                }
                TextCallContent::Compound(compound) => self.compound(&compound.children),
                TextCallContent::Text(_) => false,
            },
            TemplateChildNode::CompoundExpression(compound) => self.compound(&compound.children),
            TemplateChildNode::Text(_)
            | TemplateChildNode::Comment(_)
            | TemplateChildNode::Hoisted(_) => false,
        }
    }
}

fn contains_identifier(content: &str, name: &str) -> bool {
    let bytes = content.as_bytes();
    content.match_indices(name).any(|(start, _)| {
        let end = start + name.len();
        let boundary = |byte: u8| !byte.is_ascii_alphanumeric() && byte != b'_' && byte != b'$';
        start
            .checked_sub(1)
            .and_then(|index| bytes.get(index))
            .is_none_or(|byte| boundary(*byte))
            && bytes.get(end).is_none_or(|byte| boundary(*byte))
    })
}

struct ScopeReferenceVisitor<'ctx> {
    ctx: &'ctx CodegenContext,
    local_scopes: Vec<FxHashSet<String>>,
    found: bool,
}

impl<'ctx> ScopeReferenceVisitor<'ctx> {
    fn new(ctx: &'ctx CodegenContext) -> Self {
        Self {
            ctx,
            local_scopes: vec![FxHashSet::default()],
            found: false,
        }
    }

    fn is_local(&self, name: &str) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }
}

impl<'a> ExpressionScope<'a> for ScopeReferenceVisitor<'_> {
    fn push_scope(&mut self) {
        self.local_scopes.push(FxHashSet::default());
    }

    fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    fn add_local(&mut self, name: &str) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(name.into());
        }
    }
}

impl<'a> Visit<'a> for ScopeReferenceVisitor<'_> {
    fn visit_identifier_reference(&mut self, identifier: &oxc_ast::ast::IdentifierReference<'a>) {
        let name = identifier.name.as_str();
        if !self.is_local(name) && self.ctx.is_slot_param(name) {
            self.found = true;
        }
    }

    fn visit_program(&mut self, node: &oxc_ast::ast::Program<'a>) {
        self.scoped_program(node);
    }

    fn visit_arrow_function_expression(
        &mut self,
        node: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.scoped_arrow(node);
    }

    fn visit_function_body(&mut self, node: &oxc_ast::ast::FunctionBody<'a>) {
        self.scoped_function_body(node);
    }

    fn visit_block_statement(&mut self, node: &oxc_ast::ast::BlockStatement<'a>) {
        self.scoped_block(node);
    }

    fn visit_catch_clause(&mut self, node: &oxc_ast::ast::CatchClause<'a>) {
        self.scoped_catch(node);
    }

    fn visit_for_statement(&mut self, node: &oxc_ast::ast::ForStatement<'a>) {
        self.scoped_for(node);
    }

    fn visit_for_in_statement(&mut self, node: &oxc_ast::ast::ForInStatement<'a>) {
        self.scoped_for_in(node);
    }

    fn visit_for_of_statement(&mut self, node: &oxc_ast::ast::ForOfStatement<'a>) {
        self.scoped_for_of(node);
    }

    fn visit_switch_statement(&mut self, node: &oxc_ast::ast::SwitchStatement<'a>) {
        self.scoped_switch(node);
    }

    fn visit_class(&mut self, node: &oxc_ast::ast::Class<'a>) {
        self.scoped_class(node);
    }

    fn visit_static_block(&mut self, node: &oxc_ast::ast::StaticBlock<'a>) {
        self.scoped_static_block(node);
    }

    fn visit_function(&mut self, node: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
        self.scoped_function(node, flags);
    }
}

/// Check if component has dynamic slots (requires DYNAMIC_SLOTS patch flag)
pub fn has_dynamic_slots_flag(el: &ElementNode<'_>, source: &str) -> bool {
    // A forwarded slots object can change without anything on this vnode
    // changing, and the emitted slots object carries no `_` stability flag (see
    // `generate_slots`), so the child is only re-rendered if the parent forces
    // it through DYNAMIC_SLOTS. `@vue/babel-plugin-jsx` gets the same forced
    // update for free by emitting no patch flags at all: `shouldUpdateComponent`
    // falls back to "any children means update" for unoptimized vnodes.
    if slots_spread(el).is_some() {
        return true;
    }
    let collected_slots = collect_slots(el, source);
    if collected_slots.iter().any(|s| s.is_dynamic) {
        return true;
    }
    // Also check for v-if/v-for on slot templates (they become IfNode/ForNode children)
    has_conditional_or_loop_slots(el)
}

/// Check whether this component forwards an incoming slot to another component,
/// e.g. `<Inner><slot /></Inner>`.
pub fn has_forwarded_slot_outlet(el: &ElementNode<'_>) -> bool {
    el.children.iter().any(child_contains_slot_outlet)
}

fn child_contains_slot_outlet(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Element(el) => {
            if el.tag_type == ElementType::Slot || el.tag == "slot" {
                return true;
            }
            ensure_sufficient_stack(|| el.children.iter().any(child_contains_slot_outlet))
        }
        TemplateChildNode::If(if_node) => if_node
            .branches
            .iter()
            .flat_map(|branch| branch.children.iter())
            .any(|child| ensure_sufficient_stack(|| child_contains_slot_outlet(child))),
        TemplateChildNode::For(for_node) => for_node
            .children
            .iter()
            .any(|child| ensure_sufficient_stack(|| child_contains_slot_outlet(child))),
        _ => false,
    }
}

/// Check if children have conditional (v-if) or looped (v-for) slot templates.
/// Only returns true when the IfNode/ForNode wraps a `<template v-slot>` element.
pub(super) fn has_conditional_or_loop_slots(el: &ElementNode<'_>) -> bool {
    el.children.iter().any(|child| match child {
        TemplateChildNode::If(if_node) => if_node.branches.iter().any(|branch| {
            branch.children.iter().any(|c| {
                if let TemplateChildNode::Element(el) = c {
                    el.tag == "template" && has_v_slot(el)
                } else {
                    false
                }
            })
        }),
        TemplateChildNode::For(for_node) => for_node.children.iter().any(|c| {
            if let TemplateChildNode::Element(el) = c {
                el.tag == "template" && has_v_slot(el)
            } else {
                false
            }
        }),
        _ => false,
    })
}

pub(super) fn has_authored_slot_template(el: &ElementNode<'_>) -> bool {
    el.children.iter().any(child_is_slot_template)
}

pub(super) fn child_is_implicit_default(
    child: &TemplateChildNode<'_>,
    has_slot_template: bool,
) -> bool {
    if has_slot_template && matches!(child, TemplateChildNode::Comment(_)) {
        return false;
    }
    !matches!(child, TemplateChildNode::Element(el) if el.tag == "template" && has_v_slot(el))
}

pub(super) fn child_is_slot_template(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Element(el) => el.tag == "template" && has_v_slot(el),
        TemplateChildNode::If(if_node) => if_node.branches.iter().any(|branch| {
            branch.children.iter().any(|child| {
                matches!(
                    child,
                    TemplateChildNode::Element(el)
                        if el.tag == "template" && has_v_slot(el)
                )
            })
        }),
        TemplateChildNode::For(for_node) => for_node.children.iter().any(|child| {
            matches!(
                child,
                TemplateChildNode::Element(el)
                    if el.tag == "template" && has_v_slot(el)
            )
        }),
        _ => false,
    }
}

pub(super) fn slot_children_have_meaningful_content(children: &[&TemplateChildNode<'_>]) -> bool {
    children.iter().any(|child| match child {
        TemplateChildNode::Text(text) => !text.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => false,
        _ => true,
    })
}
