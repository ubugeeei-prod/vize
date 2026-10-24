//! vue/no-non-component-keep-alive-child
//!
//! Disallow plain element wrappers directly below `<KeepAlive>`.
//!
//! Vue's `<KeepAlive>` caches component VNodes. A native wrapper such as
//! `<div v-if>` between `<KeepAlive>` and the component leaves the wrapped
//! component uncached while still looking intentional in review.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_allocator::{CloneIn, GetAllocator};
use oxc_ast::{
    ast::{Expression, IdentifierReference},
    builder::{AstBuilder, GetAstBuilder},
};
use oxc_ast_visit::VisitMut;
use oxc_ecmascript::{
    GlobalContext, ToBoolean,
    constant_evaluation::{ConstantEvaluation, ConstantEvaluationCtx, IsLiteralValue},
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext, PropertyReadSideEffects},
};
use vize_l0::is_native_tag;
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-non-component-keep-alive-child",
    description: "Disallow plain element wrappers directly below `<KeepAlive>`",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Detect native wrappers that make `<KeepAlive>` a no-op.
pub struct NoNonComponentKeepAliveChild;

impl Rule for NoNonComponentKeepAliveChild {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if !is_keep_alive_tag(element.tag) {
            return;
        }

        check_children(ctx, &element.children);
    }
}

fn check_children<'a>(ctx: &mut LintContext<'a>, children: &[TemplateChildNode<'a>]) {
    for child in children {
        check_child(ctx, child);
    }
}

fn check_child<'a>(ctx: &mut LintContext<'a>, child: &TemplateChildNode<'a>) {
    match child {
        TemplateChildNode::Comment(_) => {}
        TemplateChildNode::Text(text) if text.content.trim().is_empty() => {}
        TemplateChildNode::Element(element) => check_element_child(ctx, element),
        TemplateChildNode::If(if_node) => {
            for branch in &if_node.branches {
                check_children(ctx, &branch.children);
            }
        }
        TemplateChildNode::For(for_node) => check_children(ctx, &for_node.children),
        _ => {}
    }
}

fn check_element_child<'a>(ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
    if is_allowed_component_child(element) {
        return;
    }

    if element.tag_type == ElementType::Template {
        check_children(ctx, &element.children);
        return;
    }

    if is_keep_alive_transparent_wrapper(element.tag) {
        check_children(ctx, &element.children);
        return;
    }

    // The lint parse leaves kebab-case components such as `<n-layout>` as
    // elements; only a native tag is known to be a plain wrapper.
    if element.tag_type != ElementType::Element
        || !is_native_tag(element.tag)
        || (has_v_show_only(element)
            && !contains_changing_component(&element.children, ctx.allocator().as_oxc()))
    {
        return;
    }

    ctx.warn_with_help(
        ctx.t("vue/no-non-component-keep-alive-child.message"),
        &element.loc,
        ctx.t("vue/no-non-component-keep-alive-child.help"),
    );
}

fn is_allowed_component_child(element: &ElementNode<'_>) -> bool {
    if is_dynamic_component(element) {
        return true;
    }

    if element.tag_type == ElementType::Component || element.tag_type == ElementType::Slot {
        return !is_keep_alive_transparent_wrapper(element.tag);
    }

    is_allowed_builtin_component_child(element.tag)
}

fn is_dynamic_component(element: &ElementNode<'_>) -> bool {
    element.tag == "component"
        && element.props.iter().any(|prop| match prop {
            PropNode::Attribute(attr) => attr.name == "is",
            PropNode::Directive(dir) => dir.name == "bind" && expression_arg_is(dir, "is"),
        })
}

fn is_keep_alive_tag(tag: &str) -> bool {
    matches!(tag, "KeepAlive" | "keep-alive")
}

fn is_keep_alive_transparent_wrapper(tag: &str) -> bool {
    matches!(
        tag,
        "Transition" | "transition" | "BaseTransition" | "base-transition"
    )
}

fn is_allowed_builtin_component_child(tag: &str) -> bool {
    matches!(
        tag,
        "Suspense" | "suspense" | "Teleport" | "teleport" | "TransitionGroup" | "transition-group"
    )
}

fn has_v_show_only(element: &ElementNode<'_>) -> bool {
    let mut has_show = false;
    for prop in &element.props {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        match dir.name {
            "show" => has_show = true,
            "if" | "else-if" | "else" | "for" => return false,
            "bind" if is_key_binding(dir) => return false,
            _ => {}
        }
    }
    has_show
}

// #5966 intentionally permits static visibility-only wrappers. A changing
// component below that native VNode still remounts: KeepAlive never owns it.
// Carry structural/key ownership through native and transparent ancestors,
// stopping at components rather than guessing how their slots are rendered.
fn contains_changing_component<'a>(
    children: &[TemplateChildNode<'a>],
    allocator: &'a oxc_allocator::Allocator,
) -> bool {
    let constants = ConstantContext(AstBuilder::new(allocator));
    let mut pending = vec![(children, false)];
    while let Some((children, inherited)) = pending.pop() {
        let mut conditional = false;
        let mut chain_exhausted = false;
        for child in children {
            match child {
                TemplateChildNode::Element(element) => {
                    let mut changing = inherited;
                    let mut unreachable = false;
                    if !element.props.iter().any(|prop| {
                        matches!(prop,
                        PropNode::Directive(dir) if matches!(dir.name, "if" | "else-if" | "else"))
                    }) {
                        conditional = false;
                        chain_exhausted = false;
                    }
                    for prop in &element.props {
                        let PropNode::Directive(dir) = prop else {
                            continue;
                        };
                        match dir.name {
                            "if" | "else-if" => {
                                if dir.name == "if" {
                                    conditional = false;
                                    chain_exhausted = false;
                                }
                                let truth = dir
                                    .exp
                                    .as_ref()
                                    .and_then(|expr| constant_truthiness(expr, &constants));
                                unreachable = chain_exhausted || truth == Some(false);
                                chain_exhausted |= truth == Some(true);
                                conditional |= truth.is_none() && binding_can_change(dir);
                            }
                            "else" => {
                                unreachable = chain_exhausted;
                                chain_exhausted = true;
                            }
                            "for" => changing = true,
                            "bind" if is_key_binding(dir) => changing |= binding_can_change(dir),
                            _ => {}
                        }
                    }
                    if unreachable {
                        continue;
                    }
                    changing |= conditional;
                    let component = is_allowed_component_child(element)
                        || (element.tag_type == ElementType::Element
                            && !is_native_tag(element.tag)
                            && element.tag != "component"
                            && !is_keep_alive_transparent_wrapper(element.tag));
                    if component {
                        if element.tag_type != ElementType::Slot
                            && (changing
                                || (is_dynamic_component(element)
                                    && element.props.iter().any(|prop| {
                                        matches!(prop, PropNode::Directive(dir)
                                            if dir.name == "bind" && expression_arg_is(dir, "is")
                                                && binding_can_change(dir))
                                    })))
                        {
                            return true;
                        }
                    } else {
                        pending.push((&element.children, changing));
                    }
                }
                TemplateChildNode::If(node) => {
                    let mut changing = inherited;
                    for branch in &node.branches {
                        let truth = branch
                            .condition
                            .as_ref()
                            .and_then(|expr| constant_truthiness(expr, &constants));
                        if truth == Some(false) {
                            continue;
                        }
                        changing |= truth.is_none()
                            && branch.condition.as_ref().is_some_and(expression_can_change);
                        pending.push((
                            &branch.children,
                            changing || branch.user_key.as_ref().is_some_and(key_can_change),
                        ));
                        if branch.condition.is_none() || truth == Some(true) {
                            break;
                        }
                    }
                }
                TemplateChildNode::IfBranch(branch) => {
                    let truth = branch
                        .condition
                        .as_ref()
                        .and_then(|expr| constant_truthiness(expr, &constants));
                    if truth != Some(false) {
                        pending.push((
                            &branch.children,
                            inherited
                                || (truth.is_none()
                                    && branch
                                        .condition
                                        .as_ref()
                                        .is_some_and(expression_can_change))
                                || branch.user_key.as_ref().is_some_and(key_can_change),
                        ));
                    }
                }
                TemplateChildNode::For(node) => {
                    pending.push((&node.children, inherited || !node.parse_result.match_scope))
                }
                _ => {}
            }
        }
    }
    false
}

// The template has no script scope/intrinsic provenance. OXC must not
// assume identifier values, globals, pure calls/annotations, or property reads.
struct ConstantContext<'a>(AstBuilder<'a>);

impl<'a> GlobalContext<'a> for ConstantContext<'a> {
    fn is_global_reference(&self, _: &IdentifierReference<'a>) -> bool {
        false
    }
}

impl<'a> MayHaveSideEffectsContext<'a> for ConstantContext<'a> {
    fn annotations(&self) -> bool {
        false
    }
    fn manual_pure_functions(&self, _: &Expression<'_>) -> bool {
        false
    }
    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        PropertyReadSideEffects::All
    }
    fn unknown_global_side_effects(&self) -> bool {
        true
    }
}

impl<'a> GetAllocator<'a> for ConstantContext<'a> {
    fn allocator(&self) -> &'a oxc_allocator::Allocator {
        self.0.allocator()
    }
}

impl<'a> GetAstBuilder<'a> for ConstantContext<'a> {
    type Builder = AstBuilder<'a>;
    fn builder(&self) -> &Self::Builder {
        &self.0
    }
}

impl<'a> ConstantEvaluationCtx<'a> for ConstantContext<'a> {}

struct OpaqueReferences;

impl<'a> VisitMut<'a> for OpaqueReferences {
    fn visit_identifier_reference(&mut self, reference: &mut IdentifierReference<'a>) {
        reference.name = "__vize_unknown_reference".into();
    }
}

fn constant_truthiness<'a>(
    expression: &ExpressionNode<'a>,
    context: &ConstantContext<'a>,
) -> Option<bool> {
    let ExpressionNode::Simple(simple) = expression else {
        return None;
    };
    let retained = simple
        .js_ast
        .filter(|ast| ast.raw == simple.content)?
        .as_expression()?;
    let ast = retained.ast.get_inner_expression();
    if ast.may_have_side_effects(context) {
        return None;
    }
    // Pinned OXC's equality fallback recognizes NaN by name without consulting
    // scope. Isolate evaluation from every authored reference name; preserve the
    // retained AST and let OXC prove short circuits such as true || visible.
    // Literal values need no clone or reference normalization.
    // TODO: Remove normalization once pinned OXC equality honors GlobalContext;
    // retain the local-shadowing controls.
    let mut opaque;
    let ast = if ast.is_literal_value(false, context) {
        ast
    } else {
        opaque = ast.clone_in(context.allocator());
        OpaqueReferences.visit_expression(&mut opaque);
        &opaque
    };
    ast.evaluate_value_to_boolean(context).or_else(|| {
        // Pinned OXC deliberately skips BinaryExpression's boolean target.
        ast.evaluate_value(context)?.to_boolean(context)
    })
}

fn key_can_change(prop: &PropNode<'_>) -> bool {
    matches!(prop, PropNode::Directive(dir)
        if dir.name == "bind" && is_key_binding(dir) && binding_can_change(dir))
}

fn binding_can_change(dir: &vize_relief::DirectiveNode<'_>) -> bool {
    dir.exp.as_ref().is_none_or(expression_can_change)
}

fn expression_can_change(expression: &ExpressionNode<'_>) -> bool {
    let ExpressionNode::Simple(simple) = expression else {
        return true;
    };
    if simple.is_static {
        return false;
    }
    // Only consume the retained whole original expression. Never reparse or
    // trust stale payloads copied by a transform that changed the text.
    let Some(retained) = simple
        .js_ast
        .filter(|ast| ast.raw == simple.content)
        .and_then(|ast| ast.as_expression())
    else {
        return true;
    };
    let mut pending = vec![retained.ast];
    while let Some(expression) = pending.pop() {
        match expression.get_inner_expression() {
            Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_) => {}
            Expression::TemplateLiteral(node) => pending.extend(node.expressions.iter()),
            Expression::UnaryExpression(node) => pending.push(&node.argument),
            Expression::BinaryExpression(node) => pending.extend([&node.left, &node.right]),
            Expression::LogicalExpression(node) => pending.extend([&node.left, &node.right]),
            Expression::ConditionalExpression(node) => {
                pending.extend([&node.test, &node.consequent, &node.alternate])
            }
            _ => return true,
        }
    }
    false
}

fn is_key_binding(dir: &vize_relief::DirectiveNode<'_>) -> bool {
    expression_arg_is(dir, "key")
}

fn expression_arg_is(dir: &vize_relief::DirectiveNode<'_>, name: &str) -> bool {
    matches!(
        dir.arg.as_ref(),
        Some(ExpressionNode::Simple(arg)) if arg.content == name
    )
}

#[cfg(test)]
mod tests;
