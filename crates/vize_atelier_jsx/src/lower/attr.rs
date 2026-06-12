//! Lowering JSX attributes into Vize props (attributes, binds, directives).
//!
//! The mapping is backend-neutral; richer `v-on`/`v-model` semantics belong to
//! the VDOM/Vapor backends (#1493/#1494). Here we faithfully classify:
//!
//! - `name="str"`      -> static [`AttributeNode`]
//! - `name` (no value) -> boolean [`AttributeNode`]
//! - `name={expr}`     -> `v-bind:name` [`DirectiveNode`]
//! - `{...obj}`        -> `v-bind="obj"` [`DirectiveNode`]
//! - `v-x` / `v-x:arg` -> [`DirectiveNode`] named `x`

use oxc_ast::ast::{
    JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXSpreadAttribute,
};
use oxc_span::{GetSpan, Span};
use vize_carton::{Box, String, Vec};
use vize_relief::ast::core::SourceLocation;
use vize_relief::ast::{AttributeNode, DirectiveNode, PropNode, TextNode};

use vize_relief::ast::SimpleExpressionNode;

use super::Lowerer;
use super::expr::container_expr_span;

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Lower a JSX opening element's attribute list into Vize props.
    pub(crate) fn lower_attributes(
        &mut self,
        items: &[JSXAttributeItem<'_>],
    ) -> Vec<'a, PropNode<'a>> {
        let mut props = Vec::new_in(self.bump());
        for item in items {
            let prop = match item {
                JSXAttributeItem::Attribute(attr) => self.lower_attribute(attr),
                JSXAttributeItem::SpreadAttribute(spread) => self.lower_spread_attribute(spread),
            };
            props.push(prop);
        }
        props
    }

    /// `{...obj}` -> `v-bind="obj"`.
    fn lower_spread_attribute(&mut self, spread: &JSXSpreadAttribute<'_>) -> PropNode<'a> {
        let loc = self.mapper().location(spread.span);
        let mut directive = DirectiveNode::new(self.bump(), "bind", loc);
        directive.exp = Some(self.dyn_expr(spread.argument.span()));
        PropNode::Directive(Box::new_in(directive, self.bump()))
    }

    fn lower_attribute(&mut self, attr: &JSXAttribute<'_>) -> PropNode<'a> {
        let loc = self.mapper().location(attr.span);

        // Directive forms: `v-model`, `v-show`, `v-on:click`, custom `v-foo:arg`.
        if let Some(directive) = self.try_directive_attribute(attr, &loc) {
            return directive;
        }

        let name = attr_full_name(&attr.name);
        let name_loc = self.mapper().location(attr.name.span());
        match attr.value.as_ref() {
            None => self.boolean_attr(name, name_loc, loc),
            Some(JSXAttributeValue::StringLiteral(string)) => {
                let value =
                    TextNode::new(string.value.as_str(), self.mapper().location(string.span));
                PropNode::Attribute(Box::new_in(
                    AttributeNode {
                        name,
                        name_loc,
                        value: Some(value),
                        loc,
                    },
                    self.bump(),
                ))
            }
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                match container_expr_span(container) {
                    // `name={}` behaves like a boolean attribute.
                    None => self.boolean_attr(name, name_loc, loc),
                    Some(span) => {
                        // `onClickCapture={h}` (event name + option modifiers) ->
                        // a `v-on` directive so core codegen emits the suffixed
                        // listener key. Plain `onClick={h}` has no recognized
                        // suffix and stays a `v-bind` like before.
                        if let Some((event, mods)) = split_on_event_modifiers(&name) {
                            return self.von_modifier_prop(
                                &event,
                                attr.name.span(),
                                span,
                                &mods,
                                loc,
                            );
                        }
                        self.bind_prop(&name, attr.name.span(), span, loc)
                    }
                }
            }
            Some(JSXAttributeValue::Element(element)) => {
                self.bind_prop(&name, attr.name.span(), element.span(), loc)
            }
            Some(JSXAttributeValue::Fragment(fragment)) => {
                self.bind_prop(&name, attr.name.span(), fragment.span(), loc)
            }
        }
    }

    fn boolean_attr(
        &self,
        name: String,
        name_loc: SourceLocation,
        loc: SourceLocation,
    ) -> PropNode<'a> {
        PropNode::Attribute(Box::new_in(
            AttributeNode {
                name,
                name_loc,
                value: None,
                loc,
            },
            self.bump(),
        ))
    }

    /// `name={expr}` -> `v-bind:name="expr"`.
    fn bind_prop(
        &self,
        name: &str,
        name_span: Span,
        value_span: Span,
        loc: SourceLocation,
    ) -> PropNode<'a> {
        let mut directive = DirectiveNode::new(self.bump(), "bind", loc);
        directive.arg = Some(self.static_expr(name, name_span));
        directive.exp = Some(self.dyn_expr(value_span));
        PropNode::Directive(Box::new_in(directive, self.bump()))
    }

    /// `onClickCapture={expr}` -> `v-on:click.capture="expr"`.
    fn von_modifier_prop(
        &self,
        event: &str,
        name_span: Span,
        value_span: Span,
        mods: &[&str],
        loc: SourceLocation,
    ) -> PropNode<'a> {
        let mut directive = DirectiveNode::new(self.bump(), "on", loc);
        directive.arg = Some(self.static_expr(event, name_span));
        directive.exp = Some(self.dyn_expr(value_span));
        for modifier in mods {
            directive.modifiers.push(SimpleExpressionNode::new(
                *modifier,
                false,
                self.mapper().location(name_span),
            ));
        }
        PropNode::Directive(Box::new_in(directive, self.bump()))
    }

    fn try_directive_attribute(
        &self,
        attr: &JSXAttribute<'_>,
        loc: &SourceLocation,
    ) -> Option<PropNode<'a>> {
        let (directive_name, arg) = match &attr.name {
            JSXAttributeName::NamespacedName(named) => {
                let directive_name = named.namespace.name.as_str().strip_prefix("v-")?;
                (
                    directive_name,
                    Some((named.name.name.as_str(), named.name.span())),
                )
            }
            JSXAttributeName::Identifier(id) => {
                let directive_name = id.name.as_str().strip_prefix("v-")?;
                (directive_name, None)
            }
        };

        let mut directive = DirectiveNode::new(self.bump(), directive_name, loc.clone());
        if let Some((arg_name, arg_span)) = arg {
            directive.arg = Some(self.static_expr(arg_name, arg_span));
        }
        directive.exp = self.directive_value_expr(attr.value.as_ref());
        Some(PropNode::Directive(Box::new_in(directive, self.bump())))
    }

    fn directive_value_expr(
        &self,
        value: Option<&JSXAttributeValue<'_>>,
    ) -> Option<vize_relief::ast::ExpressionNode<'a>> {
        match value? {
            JSXAttributeValue::StringLiteral(string) => {
                Some(self.static_expr(string.value.as_str(), string.span))
            }
            JSXAttributeValue::ExpressionContainer(container) => {
                container_expr_span(container).map(|span| self.dyn_expr(span))
            }
            JSXAttributeValue::Element(element) => Some(self.dyn_expr(element.span())),
            JSXAttributeValue::Fragment(fragment) => Some(self.dyn_expr(fragment.span())),
        }
    }
}

/// Split a babel-plugin-jsx event attribute name into its event name and
/// trailing option modifiers, e.g. `onClickCapture` -> `("click", ["capture"])`
/// and `onInputPassiveCapture` -> `("input", ["passive", "capture"])`.
///
/// Returns `None` for names without an `on<Event>` shape, without any
/// recognized trailing modifier, or whose only content is modifiers (so bare
/// `onCapture` / `onOnce` keep their plain-bind behavior).
fn split_on_event_modifiers(name: &str) -> Option<(String, std::vec::Vec<&str>)> {
    // Require an `on` prefix immediately followed by an uppercase event char.
    let rest = name.strip_prefix("on")?;
    if !rest.chars().next()?.is_ascii_uppercase() {
        return None;
    }

    // Peel recognized option modifiers off the END, preserving source order.
    let mut event = rest;
    let mut mods = std::vec::Vec::new();
    loop {
        let modifier = if let Some(head) = event.strip_suffix("Capture") {
            event = head;
            "capture"
        } else if let Some(head) = event.strip_suffix("Once") {
            event = head;
            "once"
        } else if let Some(head) = event.strip_suffix("Passive") {
            event = head;
            "passive"
        } else {
            break;
        };
        mods.push(modifier);
    }
    mods.reverse();

    // Require at least one modifier and a non-empty event tail.
    if mods.is_empty() || event.is_empty() {
        return None;
    }

    // Lowercase the first char of the remaining event name.
    let mut chars = event.chars();
    let first = chars.next()?;
    let mut lowered = String::new("");
    lowered.push(first.to_ascii_lowercase());
    lowered.push_str(chars.as_str());
    Some((lowered, mods))
}

fn attr_full_name(name: &JSXAttributeName<'_>) -> String {
    match name {
        JSXAttributeName::Identifier(id) => String::from(id.name.as_str()),
        JSXAttributeName::NamespacedName(named) => {
            let mut full = String::from(named.namespace.name.as_str());
            full.push(':');
            full.push_str(named.name.name.as_str());
            full
        }
    }
}
