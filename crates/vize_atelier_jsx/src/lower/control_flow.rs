//! Lowering JSX control-flow expression children into Vize structural relief
//! nodes (v-if / v-for) instead of plain text interpolation.
//!
//! Idiomatic JSX control flow is written as an expression child:
//!
//! ```jsx
//! {cond && <X/>}             // -> single-branch IfNode
//! {cond ? <A/> : <B/>}       // -> two-branch IfNode
//! {items.map((i) => <li/>)}  // -> ForNode (v-for)
//! ```
//!
//! Without this pass every such child would fall through to
//! [`InterpolationNode`](vize_relief::InterpolationNode) and be codegen'd as
//! `_toDisplayString(expr)` (TEXT), silently mis-compiling the render output.
//!
//! The core transform consumes **pre-built** [`IfNode`]/[`ForNode`] children
//! directly (no `v-if`/`v-for` directives required), and VDOM/Vapor codegen
//! derive everything they need from `source` + the alias expressions, so the
//! whole transform stays inside this lowering crate.
//!
//! Anything that is not confidently recognized as one of the three patterns
//! returns `None`, so it falls back to today's interpolation behavior — no
//! regressions.

use oxc_ast::ast::{
    ArrowFunctionExpression, CallExpression, ConditionalExpression, Expression, Function,
    JSXElement, JSXExpression, JSXFragment, LogicalOperator, Statement,
};
use oxc_span::{GetSpan, Span};
use vize_carton::{Box, Vec};
use vize_relief::{
    ExpressionNode, ForNode, ForParseResult, IfBranchNode, IfNode, TemplateChildNode,
};

use super::Lowerer;

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Try to lower an expression child as JSX control flow (`&&`, `?:`, or
    /// `.map(...)`). Returns `Some` when a pattern is recognized, otherwise
    /// `None` so the caller falls back to plain interpolation.
    pub(crate) fn lower_control_flow_child(
        &mut self,
        expr: &JSXExpression<'_>,
        container_span: Span,
    ) -> Option<TemplateChildNode<'a>> {
        // A `JSXExpression` inherits every `Expression` variant; the empty case
        // never reaches here.
        let expr = jsx_expression_as_expression(expr)?;
        self.lower_control_flow_expr(expr, container_span)
    }

    /// Lower an arbitrary expression as JSX control flow (`&&`, `?:`, `.map`)
    /// against `container_span`, or `None` when it is not a recognized pattern.
    /// Shared with slot-body lowering ([`super::slot`]) so a `.map`/conditional
    /// rendered inside a slot reuses the same If/For relief lowering.
    pub(crate) fn lower_control_flow_expr(
        &mut self,
        expr: &Expression<'_>,
        container_span: Span,
    ) -> Option<TemplateChildNode<'a>> {
        match unwrap_parens(expr) {
            Expression::LogicalExpression(logical) => {
                // `cond && <X/>`: render `<X/>` when `cond` is truthy.
                if logical.operator != LogicalOperator::And {
                    // `||` / `??` are value coalescing, not conditional
                    // rendering — leave them to interpolation.
                    return None;
                }
                let branch_child = self.lower_jsx_expression(&logical.right)?;

                let mut if_node = IfNode::new(self.bump(), self.mapper().location(container_span));
                let condition = self.dyn_expr(logical.left.span());
                let mut branch = IfBranchNode::new(
                    self.bump(),
                    Some(condition),
                    self.mapper().location(logical.span),
                );
                branch.children.push(branch_child);
                if_node.branches.push(branch);
                Some(TemplateChildNode::If(Box::new_in(if_node, self.bump())))
            }
            Expression::ConditionalExpression(conditional) => {
                self.lower_conditional(conditional, container_span)
            }
            Expression::CallExpression(call) => self.lower_map_call(call, container_span),
            _ => None,
        }
    }

    /// `test ? consequent : alternate` -> a multi-branch `IfNode`, but only if at
    /// least one arm is JSX (otherwise it is an ordinary value expression that
    /// belongs in interpolation).
    ///
    /// A nested ternary in the alternate (`a ? <A/> : b ? <B/> : <C/>`) is the
    /// idiomatic JSX `v-else-if` chain, so it is flattened into the *same*
    /// `IfNode` as additional condition-carrying branches rather than nested as
    /// a child. That matches the `if / else if / else` shape both the SSR and
    /// client codegen already lower, and — critically — avoids re-slicing the
    /// nested ternary's raw source into an interpolation expression, which is
    /// invalid JS (`{a ? <A/> : ...}` embeds JSX in a string) and crashes the
    /// client transform.
    fn lower_conditional(
        &mut self,
        conditional: &ConditionalExpression<'_>,
        container_span: Span,
    ) -> Option<TemplateChildNode<'a>> {
        if !self.conditional_has_jsx_arm(conditional) {
            return None;
        }

        let mut if_node = IfNode::new(self.bump(), self.mapper().location(container_span));
        self.push_conditional_branches(&mut if_node, conditional);
        Some(TemplateChildNode::If(Box::new_in(if_node, self.bump())))
    }

    /// Whether a conditional renders JSX in either arm (recursing through a
    /// nested ternary alternate), i.e. it is conditional *rendering* and not a
    /// plain value expression. A bare-value ternary (`a ? x : y`) stays
    /// interpolation, preserving today's behavior.
    fn conditional_has_jsx_arm(&self, conditional: &ConditionalExpression<'_>) -> bool {
        if is_jsx(unwrap_parens(&conditional.consequent)) {
            return true;
        }
        match unwrap_parens(&conditional.alternate) {
            Expression::ConditionalExpression(nested) => self.conditional_has_jsx_arm(nested),
            other => is_jsx(other),
        }
    }

    /// Append the `if` / `else if` / `else` branches of a (possibly nested)
    /// ternary to `if_node`. The first call contributes the `if` branch; a
    /// nested ternary alternate recurses to contribute `else if` branches; a
    /// non-ternary alternate becomes the final condition-less `else` branch.
    fn push_conditional_branches(
        &mut self,
        if_node: &mut IfNode<'a>,
        conditional: &ConditionalExpression<'_>,
    ) {
        // Condition-carrying branch for `test`.
        let consequent_child = self.lower_branch_child(&conditional.consequent);
        let condition = self.dyn_expr(conditional.test.span());
        let mut branch = IfBranchNode::new(
            self.bump(),
            Some(condition),
            self.mapper().location(conditional.consequent.span()),
        );
        branch.children.push(consequent_child);
        if_node.branches.push(branch);

        // Flatten a nested ternary alternate into further `else if` branches;
        // otherwise emit the terminal `else` branch.
        match unwrap_parens(&conditional.alternate) {
            Expression::ConditionalExpression(nested) => {
                self.push_conditional_branches(if_node, nested);
            }
            _ => {
                let alternate_child = self.lower_branch_child(&conditional.alternate);
                let mut else_branch = IfBranchNode::new(
                    self.bump(),
                    None,
                    self.mapper().location(conditional.alternate.span()),
                );
                else_branch.children.push(alternate_child);
                if_node.branches.push(else_branch);
            }
        }
    }

    /// `<expr>.map((value, index) => <jsx/>)` -> `ForNode`.
    fn lower_map_call(
        &mut self,
        call: &CallExpression<'_>,
        container_span: Span,
    ) -> Option<TemplateChildNode<'a>> {
        // Callee must be `<object>.map` (a static member, not computed/optional).
        let Expression::StaticMemberExpression(member) = unwrap_parens(&call.callee) else {
            return None;
        };
        if member.property.name.as_str() != "map" || member.optional {
            return None;
        }
        // Exactly one argument: the mapping callback.
        if call.arguments.len() != 1 {
            return None;
        }
        let argument = call.arguments.first()?.as_expression()?;

        // The callback's params (value, index) and returned JSX.
        let (params, body_jsx_child) = match unwrap_parens(argument) {
            Expression::ArrowFunctionExpression(arrow) => {
                (&arrow.params, self.arrow_return_child(arrow)?)
            }
            Expression::FunctionExpression(func) => {
                (&func.params, self.function_return_child(func)?)
            }
            _ => return None,
        };

        let source = self.dyn_expr(member.object.span());

        // param0 -> value alias, param1 -> key alias (renderList's 2nd callback
        // param is the index). Build each alias from the binding pattern span.
        let value_alias = params
            .items
            .first()
            .map(|p| self.dyn_expr(p.pattern.span()));
        let key_alias = params.items.get(1).map(|p| self.dyn_expr(p.pattern.span()));

        // `ForParseResult` is not read by codegen, but must be a valid struct.
        let parse_result = ForParseResult {
            source: self.dyn_expr(member.object.span()),
            value: params
                .items
                .first()
                .map(|p| self.dyn_expr(p.pattern.span())),
            key: params.items.get(1).map(|p| self.dyn_expr(p.pattern.span())),
            index: None,
            finalized: false,
        };

        let mut children = Vec::new_in(self.bump());
        children.push(body_jsx_child);

        let for_node = ForNode {
            source,
            value_alias,
            key_alias,
            object_index_alias: None,
            parse_result,
            children,
            loc: self.mapper().location(container_span),
            codegen_node: None,
        };
        Some(TemplateChildNode::For(Box::new_in(for_node, self.bump())))
    }

    /// Returned JSX of an expression-body arrow (`() => <li/>`) or a block-body
    /// arrow with a single `return <li/>`.
    fn arrow_return_child(
        &mut self,
        arrow: &ArrowFunctionExpression<'_>,
    ) -> Option<TemplateChildNode<'a>> {
        if arrow.expression {
            // Expression-body arrow: the body is a single `ExpressionStatement`.
            let stmt = arrow.body.statements.first()?;
            let Statement::ExpressionStatement(expr_stmt) = stmt else {
                return None;
            };
            self.lower_render_expr_child(&expr_stmt.expression)
        } else {
            self.lower_block_return_jsx(&arrow.body.statements)
        }
    }

    /// Returned JSX of a `function (..) { return <li/>; }` callback.
    fn function_return_child(&mut self, func: &Function<'_>) -> Option<TemplateChildNode<'a>> {
        let body = func.body.as_ref()?;
        self.lower_block_return_jsx(&body.statements)
    }

    /// Find the first `return <jsx>` in a block body and lower its JSX.
    fn lower_block_return_jsx(
        &mut self,
        statements: &[Statement<'_>],
    ) -> Option<TemplateChildNode<'a>> {
        for stmt in statements {
            if let Statement::ReturnStatement(ret) = stmt {
                let argument = ret.argument.as_ref()?;
                return self.lower_render_expr_child(argument);
            }
        }
        None
    }

    /// Lower a returned render expression. This accepts JSX directly and JSX
    /// control-flow expressions such as `value ? <A/> : <B/>` inside `.map()`
    /// callbacks, while still rejecting ordinary value-returning callbacks so
    /// they keep the plain interpolation fallback.
    fn lower_render_expr_child(&mut self, expr: &Expression<'_>) -> Option<TemplateChildNode<'a>> {
        let inner = unwrap_parens(expr);
        match inner {
            Expression::JSXElement(_) | Expression::JSXFragment(_) => {
                self.lower_jsx_expression(inner)
            }
            Expression::LogicalExpression(_)
            | Expression::ConditionalExpression(_)
            | Expression::CallExpression(_) => self.lower_control_flow_expr(inner, inner.span()),
            _ => None,
        }
    }

    /// Lower an expression as a JSX element/fragment child, or `None` if it is
    /// not JSX.
    fn lower_jsx_expression(&mut self, expr: &Expression<'_>) -> Option<TemplateChildNode<'a>> {
        match unwrap_parens(expr) {
            Expression::JSXElement(element) => Some(self.element_child(element)),
            Expression::JSXFragment(fragment) => Some(self.fragment_child(fragment)),
            _ => None,
        }
    }

    /// Lower a conditional arm into a single child. JSX becomes an element
    /// child; a `&&` / `.map(...)` arm recurses into nested control flow (so an
    /// arm like `cond && <X/>` or `items.map(...)` renders correctly instead of
    /// being sliced into an invalid interpolation expression); anything else
    /// reuses the interpolation path so non-JSX value arms stay correct text.
    fn lower_branch_child(&mut self, expr: &Expression<'_>) -> TemplateChildNode<'a> {
        let inner = unwrap_parens(expr);
        match inner {
            Expression::JSXElement(element) => self.element_child(element),
            Expression::JSXFragment(fragment) => self.fragment_child(fragment),
            Expression::LogicalExpression(_)
            | Expression::ConditionalExpression(_)
            | Expression::CallExpression(_) => self
                .lower_control_flow_expr(inner, inner.span())
                .unwrap_or_else(|| {
                    let content = self.dyn_expr(inner.span());
                    self.interpolation_child(content, inner.span())
                }),
            other => {
                let content = self.dyn_expr(other.span());
                self.interpolation_child(content, other.span())
            }
        }
    }

    fn element_child(&mut self, element: &JSXElement<'_>) -> TemplateChildNode<'a> {
        TemplateChildNode::Element(Box::new_in(self.lower_element_node(element), self.bump()))
    }

    fn fragment_child(&mut self, fragment: &JSXFragment<'_>) -> TemplateChildNode<'a> {
        TemplateChildNode::Element(Box::new_in(self.lower_fragment_node(fragment), self.bump()))
    }

    fn interpolation_child(
        &self,
        content: ExpressionNode<'a>,
        span: Span,
    ) -> TemplateChildNode<'a> {
        let node = vize_relief::InterpolationNode {
            content,
            loc: self.mapper().location(span),
            #[cfg(feature = "legacy")]
            raw: false,
        };
        TemplateChildNode::Interpolation(Box::new_in(node, self.bump()))
    }
}

/// `true` if the expression is a JSX element or fragment (parens unwrapped).
fn is_jsx(expr: &Expression<'_>) -> bool {
    matches!(
        unwrap_parens(expr),
        Expression::JSXElement(_) | Expression::JSXFragment(_)
    )
}

/// Strip nested `(expr)` parentheses.
fn unwrap_parens<'e, 'a>(mut expr: &'e Expression<'a>) -> &'e Expression<'a> {
    while let Expression::ParenthesizedExpression(inner) = expr {
        expr = &inner.expression;
    }
    expr
}

/// View a non-empty `JSXExpression` as its inherited `Expression`.
///
/// `JSXExpression` is layout-compatible with `Expression` (it only adds the
/// `EmptyExpression` discriminant), so the safe `as_expression` accessor returns
/// the borrowed `Expression` for every non-empty case.
fn jsx_expression_as_expression<'e, 'a>(expr: &'e JSXExpression<'a>) -> Option<&'e Expression<'a>> {
    match expr {
        JSXExpression::EmptyExpression(_) => None,
        other => other.as_expression(),
    }
}
