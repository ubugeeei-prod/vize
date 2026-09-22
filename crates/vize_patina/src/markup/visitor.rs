//! [`MarkupDocumentVisitor`]: drives markup rules from any backend.

use super::dispatch::MarkupRules;
use super::element::{MarkupElement, MarkupElementInner};
use super::hooks::MarkupHooks;
use super::jsx_names::{jsx_element_ref, jsx_fragment_ref};
use super::loc_to_range;
use super::node::{MarkupNode, MarkupText};
use super::relief_scopes::{BranchKind, ReliefChain, branch_of, list_of};
use super::s2::walk::{S2Step, scope_region};
use super::s2::{S2ElementOp, S2Markup, children};
use super::scope::{MarkupConditional, MarkupList};
use super::{MarkupContext, MarkupDocument, MarkupDocumentInner, jsx_roots};
use oxc_ast::ast::{JSXChild, JSXElement, JSXFragment, Program};
use oxc_ast_visit::{Visit, walk::walk_program};
use vize_relief::{ElementNode, TemplateChildNode};
use vize_s0::profile;
use vize_s2::op::Op;

/// Projection visitor that drives markup rules — one rule, or a fused set
/// ([`MarkupRules`]) — from any backend.
///
/// Walks the backing tree in source order, firing the rules' hooks. It never
/// materializes a synthetic template AST: the hooks receive borrow-based
/// facades over the live nodes. Profiling spans (`patina.markup.*`) mirror the
/// template visitor so the adapter's overhead stays visible in benchmarks.
pub struct MarkupDocumentVisitor<'rule, 'ctx, 'mc, 'a, R: ?Sized> {
    rules: &'rule R,
    ctx: &'ctx mut MarkupContext<'mc, 'a>,
    /// Whether a rule listens to scopes or text: only then does a Relief
    /// sibling list group its raw `v-if` chains (the element order is the
    /// same either way).
    scopes: bool,
    /// Whether a rule listens to text.
    text: bool,
}

impl<'rule, 'ctx, 'mc, 'a, R: MarkupRules + ?Sized> MarkupDocumentVisitor<'rule, 'ctx, 'mc, 'a, R> {
    pub(super) fn new(rules: &'rule R, ctx: &'ctx mut MarkupContext<'mc, 'a>) -> Self {
        let scopes =
            rules.subscribes(MarkupHooks::CONDITIONAL) || rules.subscribes(MarkupHooks::TEXT);
        let text = rules.subscribes(MarkupHooks::TEXT);
        Self {
            rules,
            ctx,
            scopes,
            text,
        }
    }

    pub(super) fn run(&mut self, document: &MarkupDocument<'a>) {
        self.rules.enter_document(self.ctx, document);

        profile!("patina.markup.visit", {
            match document.inner {
                MarkupDocumentInner::Relief(root) => self.visit_relief_children(&root.children),
                MarkupDocumentInner::Jsx { program, offset } => {
                    self.visit_jsx_program(program, offset)
                }
                MarkupDocumentInner::S2(markup) => self.visit_s2_region(markup, &markup.root.ops),
            }
        });
    }

    fn text(&mut self, text: MarkupText<'a>) {
        self.rules.enter_text(self.ctx, &text);
    }

    fn interpolation(&mut self, range: crate::ir::ByteRange) {
        self.rules.enter_interpolation(self.ctx, range);
    }

    fn conditional(&mut self, conditional: MarkupConditional<'a>) {
        self.rules.enter_conditional(self.ctx, &conditional);
    }

    fn list(&mut self, list: MarkupList<'a>) {
        self.rules.enter_list(self.ctx, &list);
    }

    /// A Relief sibling list. Raw directive chains are grouped into scopes
    /// ([`super::relief_scopes`]); lowered `IfNode` / `ForNode` pass through.
    fn visit_relief_children(&mut self, children: &'a [TemplateChildNode<'a>]) {
        let mut index = 0;
        while let Some(child) = children.get(index) {
            match child {
                TemplateChildNode::Element(element)
                    if self.scopes && matches!(branch_of(element), Some((BranchKind::If, _))) =>
                {
                    let chain = ReliefChain::scan(children, index);
                    self.conditional(MarkupConditional::from_chain(chain));
                    chain.walk_branches(&mut |branch| self.visit_relief_listed(branch));
                    chain.walk_kept_gaps(&mut |text| self.text(text));
                    index = chain.end;
                    continue;
                }
                TemplateChildNode::Element(element) => self.visit_relief_listed(element),
                TemplateChildNode::Text(text) if self.text => {
                    self.text(MarkupText::from_relief(text));
                }
                TemplateChildNode::Interpolation(interpolation) => {
                    self.interpolation(loc_to_range(&interpolation.loc));
                }
                TemplateChildNode::If(if_node) => {
                    self.conditional(MarkupConditional::from_relief(if_node));
                    for branch in if_node.branches.iter() {
                        self.visit_relief_children(&branch.children);
                    }
                }
                TemplateChildNode::For(for_node) => {
                    self.list(MarkupList::from_relief(for_node));
                    self.visit_relief_children(&for_node.children);
                }
                _ => {}
            }
            index += 1;
        }
    }

    /// A raw element, inside its `v-for` list scope when it carries one.
    fn visit_relief_listed(&mut self, element: &'a ElementNode<'a>) {
        if self.rules.subscribes(MarkupHooks::LIST)
            && let Some(directive) = list_of(element)
        {
            self.list(MarkupList::from_relief_directive(element, directive));
        }
        self.visit_element(MarkupElement::new(element));
    }

    /// An S2 region, in page order.
    fn visit_s2_region(&mut self, doc: &'a S2Markup<'a>, ops: &'a [Op<'a>]) {
        for op in ops {
            match op {
                Op::Element(_) | Op::Component(_) | Op::Slot(_) => {
                    let Some(element) = S2ElementOp::from_op(op) else {
                        continue;
                    };
                    self.visit_element(MarkupElement::from_s2(element, doc));
                }
                Op::Text(text) => self.text(children::text_node(doc, text)),
                Op::Interpolation(interpolation) => {
                    children::walk_interpolation(doc, interpolation, &mut |node| match node {
                        MarkupNode::Text(text) => self.text(text),
                        MarkupNode::Interpolation(range) => self.interpolation(range),
                        _ => {}
                    });
                }
                Op::Comment(_) => {}
                Op::If(if_op) => {
                    self.conditional(MarkupConditional::from_s2(if_op, doc));
                    for branch in if_op.branches.iter() {
                        self.visit_s2_step(doc, scope_region(doc, branch.span, &branch.region.ops));
                    }
                }
                Op::For(for_op) => {
                    self.list(MarkupList::from_s2(for_op, doc));
                    self.visit_s2_step(doc, scope_region(doc, for_op.span, &for_op.region.ops));
                }
            }
        }
    }

    fn visit_s2_step(&mut self, doc: &'a S2Markup<'a>, step: S2Step<'a>) {
        match step {
            S2Step::Element(element, _) => self.visit_element(element),
            S2Step::Region(region) => self.visit_s2_region(doc, region),
        }
    }

    pub(super) fn visit_element(&mut self, element: MarkupElement<'a>) {
        self.ctx.push_element(element);

        self.rules.enter_element(self.ctx, &element);

        if self.rules.subscribes(MarkupHooks::BINDING) {
            element.walk_bindings(&mut |binding| {
                self.rules.enter_binding(self.ctx, &element, &binding);
            });
        }
        if self.rules.subscribes(MarkupHooks::DIRECTIVE) {
            element.walk_directives(&mut |directive| {
                self.rules.enter_directive(self.ctx, &element, &directive);
            });
        }

        match element.inner {
            MarkupElementInner::Relief(node) => self.visit_relief_children(&node.children),
            MarkupElementInner::JsxElement { node, offset } => {
                self.ctx.push_jsx_attribute_value();
                jsx_roots::visit_attribute_roots(self, jsx_element_ref(node), offset);
                self.ctx.pop_jsx_attribute_value();
                self.visit_jsx_children(&jsx_element_ref(node).children, offset)
            }
            MarkupElementInner::JsxFragment { node, offset } => {
                self.visit_jsx_children(&jsx_fragment_ref(node).children, offset)
            }
            MarkupElementInner::S2 { op, doc, .. } => self.visit_s2_region(doc, op.children()),
            MarkupElementInner::S2Carrier { doc, region, .. } => {
                self.visit_s2_region(doc, region);
            }
        }

        self.rules.exit_element(self.ctx, &element);

        let popped = self.ctx.pop_element();
        debug_assert!(
            popped.is_some(),
            "markup visitor must pop the element it just visited"
        );
    }

    fn visit_jsx_program(&mut self, program: &'a Program<'a>, offset: u32) {
        // Stream each outermost JSX element/fragment into `visit_element` in
        // source order, with no root container allocated. Nested JSX roots are
        // handled by our own element/attribute/expression recursion.
        struct RootDriver<'visitor, 'rule, 'ctx, 'mc, 'a, R: ?Sized> {
            visitor: &'visitor mut MarkupDocumentVisitor<'rule, 'ctx, 'mc, 'a, R>,
            offset: u32,
        }

        impl<'a, R: MarkupRules + ?Sized> Visit<'a> for RootDriver<'_, '_, '_, '_, 'a, R> {
            fn visit_jsx_element(&mut self, it: &JSXElement<'a>) {
                self.visitor
                    .visit_element(MarkupElement::from_jsx_element(it as *const _, self.offset));
            }

            fn visit_jsx_fragment(&mut self, it: &JSXFragment<'a>) {
                self.visitor.visit_element(MarkupElement::from_jsx_fragment(
                    it as *const _,
                    self.offset,
                ));
            }
        }

        let mut driver = RootDriver {
            visitor: self,
            offset,
        };
        walk_program(&mut driver, program);
    }

    fn visit_jsx_children(&mut self, children: &'a [JSXChild<'a>], offset: u32) {
        for child in children {
            match MarkupNode::from_jsx_child(child, offset) {
                MarkupNode::Element(element) => self.visit_element(element),
                MarkupNode::Text(text) => self.text(text),
                MarkupNode::Interpolation(range) => {
                    self.interpolation(range);
                    if let JSXChild::ExpressionContainer(container) = child {
                        jsx_roots::visit_expression_container_roots(self, container, offset);
                    }
                }
                MarkupNode::If(_) | MarkupNode::For(_) => {
                    if let JSXChild::ExpressionContainer(container) = child {
                        jsx_roots::visit_expression_container_roots(self, container, offset);
                    }
                }
                _ => {}
            }
        }
    }
}
