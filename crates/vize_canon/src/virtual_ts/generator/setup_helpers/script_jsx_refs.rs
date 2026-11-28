//! Static `ref="name"` attributes on native elements of a script's JSX.
//!
//! A component without a `<template>` renders the JSX of its script, so those
//! attributes are the refs `useTemplateRef` reads. Vize lowers an
//! `items.map((item) => <li ref="row" />)` JSX child to the same `v-for` a
//! template writes, with `ref_for`, so the ref holds an array of its elements.
//!
//! Only native tags are typed. A name any other target registers (a component,
//! a member tag, a custom element) is left out of the registry entirely, so its
//! `useTemplateRef` keeps Vue's own untyped result rather than a partial union.

use oxc_ast::ast::{
    Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement,
    JSXElementName, JSXExpression, LogicalOperator, Program,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{String, is_native_tag};

/// One `ref="name"` on a native JSX element.
pub(in crate::virtual_ts::generator) struct ScriptJsxRef {
    pub(super) name: String,
    pub(super) tag: String,
    pub(super) is_svg: bool,
    /// Rendered by a `.map()` callback that lowers to `v-for`.
    pub(super) in_list: bool,
}

/// The typed refs of `program`, in source order.
pub(super) fn collect(program: &Program<'_>) -> Vec<ScriptJsxRef> {
    let mut collector = Collector::default();
    collector.visit_program(program);
    let Collector { refs, untyped, .. } = collector;
    refs.into_iter()
        .filter(|entry| !untyped.contains(&entry.name))
        .collect()
}

#[derive(Default)]
struct Collector {
    refs: Vec<ScriptJsxRef>,
    /// Names some target registers that is not a native element.
    untyped: Vec<String>,
    /// Starts of the `.map()` callbacks that lower to `v-for`.
    list_callbacks: Vec<u32>,
    list_depth: usize,
    /// Whether each enclosing element puts its children in the SVG namespace.
    svg: Vec<bool>,
}

impl Collector {
    /// Mark the `.map()` callbacks of a JSX child expression, through the
    /// `&&` and `?:` arms the compiler lowers as control flow.
    fn mark_list_callbacks(&mut self, expression: &Expression<'_>) {
        match expression.without_parentheses() {
            Expression::LogicalExpression(logical) if logical.operator == LogicalOperator::And => {
                self.mark_list_callbacks(&logical.right);
            }
            Expression::ConditionalExpression(conditional) => {
                self.mark_list_callbacks(&conditional.consequent);
                self.mark_list_callbacks(&conditional.alternate);
            }
            Expression::CallExpression(call) => {
                let Expression::StaticMemberExpression(member) = call.callee.without_parentheses()
                else {
                    return;
                };
                if member.property.name != "map" || member.optional {
                    return;
                }
                let [argument] = call.arguments.as_slice() else {
                    return;
                };
                let Some(callback) = argument
                    .as_expression()
                    .map(Expression::without_parentheses)
                else {
                    return;
                };
                if matches!(
                    callback,
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                ) {
                    self.list_callbacks.push(callback.span().start);
                }
            }
            _ => {}
        }
    }

    fn record(&mut self, element: &JSXElement<'_>, is_svg: bool) {
        for attribute in &element.opening_element.attributes {
            let JSXAttributeItem::Attribute(attribute) = attribute else {
                continue;
            };
            let JSXAttributeName::Identifier(name) = &attribute.name else {
                continue;
            };
            if name.name != "ref" {
                continue;
            }
            let key = match &attribute.value {
                Some(JSXAttributeValue::StringLiteral(key)) => key.value.as_str(),
                Some(JSXAttributeValue::ExpressionContainer(container)) => {
                    match &container.expression {
                        JSXExpression::StringLiteral(key) => key.value.as_str(),
                        _ => continue,
                    }
                }
                _ => continue,
            };
            if key.is_empty() {
                continue;
            }
            match &element.opening_element.name {
                JSXElementName::Identifier(tag) if is_native_tag(tag.name.as_str()) => {
                    self.refs.push(ScriptJsxRef {
                        name: String::from(key),
                        tag: String::from(tag.name.as_str()),
                        is_svg,
                        in_list: self.list_depth > 0,
                    });
                }
                _ => self.untyped.push(String::from(key)),
            }
        }
    }

    fn enter_function(&mut self, start: u32) -> bool {
        let list = self.list_callbacks.contains(&start);
        self.list_depth += usize::from(list);
        list
    }
}

impl<'a> Visit<'a> for Collector {
    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        let parent_svg = self.svg.last().copied().unwrap_or(false);
        let tag = match &element.opening_element.name {
            JSXElementName::Identifier(tag) => tag.name.as_str(),
            _ => "",
        };
        // The parser places `<svg>` itself in the SVG namespace, and the
        // children of `<foreignObject>` back in HTML.
        let is_svg = parent_svg || tag == "svg";
        self.record(element, is_svg);
        self.svg.push(is_svg && tag != "foreignObject");
        walk::walk_jsx_element(self, element);
        self.svg.pop();
    }

    fn visit_jsx_child(&mut self, child: &JSXChild<'a>) {
        if let JSXChild::ExpressionContainer(container) = child
            && let Some(expression) = container.expression.as_expression()
        {
            self.mark_list_callbacks(expression);
        }
        walk::walk_jsx_child(self, child);
    }

    fn visit_arrow_function_expression(
        &mut self,
        function: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        let list = self.enter_function(function.span.start);
        walk::walk_arrow_function_expression(self, function);
        self.list_depth -= usize::from(list);
    }

    fn visit_function(&mut self, function: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
        let list = self.enter_function(function.span.start);
        walk::walk_function(self, function, flags);
        self.list_depth -= usize::from(list);
    }
}

#[cfg(test)]
mod tests;
