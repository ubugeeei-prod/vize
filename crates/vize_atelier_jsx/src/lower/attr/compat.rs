//! Attribute lowering specific to Babel JSX compatibility mode.

use oxc_ast::ast::{JSXAttribute, JSXAttributeName, JSXAttributeValue};
use oxc_span::{GetSpan, Span};
use vize_relief::{DirectiveNode, PropNode, SourceLocation};
use vize_s0::String;

use crate::lower::Lowerer;

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Babel's `v-text` transform passes the authored value directly to the
    /// DOM `textContent` prop. Native Vize keeps its established template-style
    /// `_toDisplayString` normalization, so this rewrite is compatibility-only.
    pub(super) fn compat_v_text_prop(
        &self,
        attr: &JSXAttribute<'_>,
        loc: SourceLocation,
    ) -> Option<PropNode<'a>> {
        if !self.uses_babel_compat() {
            return None;
        }

        let mut directive = DirectiveNode::new(self.bump(), "bind", loc);
        directive.arg = Some(self.static_expr("textContent", attr.name.span()));
        directive.exp = self.directive_value_expr(attr.value.as_ref());
        Some(PropNode::Directive(self.boxed(directive)))
    }

    /// Apply Babel's `transformOn: true` option to the two exact prop names the
    /// real plugin recognizes. The generated no-argument `v-bind` keeps this
    /// listener object in authored merge order while the helper performs the
    /// runtime key rewrite (`click` -> `onClick`).
    pub(super) fn transform_on_attribute(
        &self,
        attr: &JSXAttribute<'_>,
        loc: SourceLocation,
    ) -> Option<PropNode<'a>> {
        let helper = self.transform_on_helper()?;
        let JSXAttributeName::Identifier(name) = &attr.name else {
            return None;
        };
        if name.name != "on" && name.name != "nativeOn" {
            return None;
        }

        let value = match attr.value.as_ref() {
            None => "true",
            Some(JSXAttributeValue::StringLiteral(string)) => self.mapper().slice(string.span),
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                super::container_expr_span(container)
                    .map(|span| self.mapper().slice(span))
                    .unwrap_or("true")
            }
            Some(JSXAttributeValue::Element(element)) => self.mapper().slice(element.span()),
            Some(JSXAttributeValue::Fragment(fragment)) => self.mapper().slice(fragment.span()),
        };

        let mut expression = String::from(helper);
        expression.push('(');
        expression.push_str(value);
        expression.push(')');

        let mut directive = DirectiveNode::new(self.bump(), "bind", loc);
        directive.exp = Some(self.constant_expr(self.bump().alloc_str(&expression), attr.span));
        Some(PropNode::Directive(self.boxed(directive)))
    }

    /// Normalize the camel-cased SVG prop exactly as Babel does (#3391).
    pub(super) fn compat_attribute_name(&self, name_span: Span) -> String {
        let authored = self.mapper().slice(name_span);
        String::from(if self.uses_babel_compat() && authored == "xlinkHref" {
            "xlink:href"
        } else {
            authored
        })
    }

    /// A valueless JSX attribute is a boolean `true` in Babel's JSX
    /// semantics. Native Vize lowering deliberately keeps its established
    /// template-style empty-string value.
    /// Freeze the compat-rewritten attribute name into the compile arena.
    pub(super) fn intern_attr_name(&self, span: oxc_span::Span) -> &'a str {
        self.bump().alloc_str(&self.compat_attribute_name(span))
    }

    pub(super) fn valueless_attr(
        &self,
        name: &'a str,
        name_span: Span,
        name_loc: SourceLocation,
        loc: SourceLocation,
    ) -> PropNode<'a> {
        if !self.uses_babel_compat() {
            return self.boolean_attr(name, name_loc, loc);
        }

        let mut directive = DirectiveNode::new(self.bump(), "bind", loc);
        directive.arg = Some(self.static_expr(name, name_span));
        directive.exp = Some(self.constant_expr("true", name_span));
        PropNode::Directive(self.boxed(directive))
    }
}

/// Split a babel-plugin-jsx event attribute name into its event name and
/// trailing option modifiers, e.g. `onClickCapture` -> `("click", ["capture"])`
/// and `onInputPassiveCapture` -> `("input", ["passive", "capture"])`.
///
/// Returns `None` for names without an `on<Event>` shape, without any
/// recognized trailing modifier, or whose only content is modifiers (so bare
/// `onCapture` / `onOnce` keep their plain-bind behavior).
pub(super) fn split_on_event_modifiers(name: &str) -> Option<(String, std::vec::Vec<&str>)> {
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
