//! Lowering JSX attributes into backend-neutral Vize props leaves richer `v-on`/`v-model` semantics to the
//! VDOM/Vapor backends (#1493/#1494):
//!
//! - `name="str"`      -> static [`AttributeNode`]
//! - `name` (no value) -> boolean [`AttributeNode`]
//! - `name={expr}`     -> `v-bind:name` [`DirectiveNode`]
//! - `{...obj}`        -> `v-bind="obj"` [`DirectiveNode`]
//! - `v-x` / `v-x:arg` -> [`DirectiveNode`] named `x`

mod compat;

use compat::split_on_event_modifiers;

use oxc_ast::ast::{
    JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXSpreadAttribute,
};
use oxc_span::{GetSpan, Span};
use vize_relief::{
    AttributeNode, DirectiveNode, PropNode, SimpleExpressionNode, SourceLocation, TextNode,
};
use vize_s0::{Box, Vec, is_builtin_directive};

use super::Lowerer;
use super::expr::container_expr_span;
use super::v_model::{
    ModelArrayLowering, split_model_arg_modifiers, split_underscore_model_modifiers,
};

enum DirectiveAttributeLowering<'a> {
    NotDirective,
    Lowered(PropNode<'a>),
    Rejected,
}

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Lower a JSX opening element's attribute list into Vize props.
    ///
    /// `on_component` is whether the owning tag renders as a component; the
    /// component-only `v-models` built-in is rejected when it is `false`.
    pub(crate) fn lower_attributes(
        &mut self,
        items: &[JSXAttributeItem<'_>],
        on_component: bool,
    ) -> Vec<'a, PropNode<'a>> {
        let mut props = self.vec();
        for item in items {
            let prop = match item {
                JSXAttributeItem::Attribute(attr) => {
                    // `v-models` expands into one model binding per entry, so it
                    // yields several props (or none, plus a diagnostic) and
                    // cannot use the one-prop-per-attribute path below.
                    if self.try_lower_v_models(attr, on_component, &mut props) {
                        continue;
                    }
                    self.lower_attribute(attr, on_component)
                }
                JSXAttributeItem::SpreadAttribute(spread) => {
                    Some(self.lower_spread_attribute(spread))
                }
            };
            // `None` means the attribute was rejected with a diagnostic; it
            // contributes no prop so no code is generated from it.
            if let Some(prop) = prop {
                props.push(prop);
            }
        }
        props
    }

    /// `{...obj}` -> `v-bind="obj"`.
    fn lower_spread_attribute(&mut self, spread: &JSXSpreadAttribute<'_>) -> PropNode<'a> {
        let loc = self.mapper().location(spread.span);
        let mut directive = DirectiveNode::new(self.bump(), "bind", loc);
        directive.exp = Some(self.dyn_expr(spread.argument.span()));
        PropNode::Directive(self.boxed(directive))
    }

    /// Lower one attribute, or `None` when it was rejected with a diagnostic.
    fn lower_attribute(
        &mut self,
        attr: &JSXAttribute<'_>,
        on_component: bool,
    ) -> Option<PropNode<'a>> {
        let loc = self.mapper().location(attr.span);

        // `v-model` writes back through an assignment, so its target must be
        // assignable. Reject early: lowering it anyway produces an assignment to
        // a non-place expression, i.e. emitted code that does not parse (#3420).
        if self.reject_unassignable_model_target(attr) {
            return None;
        }

        // `v-slots` supplies the component's slots, not a prop. It is applied to
        // the children by `lower_element_node`, which is where the tag kind and
        // the already-lowered children are both available (#3418).
        if self.is_v_slots_attribute(attr) {
            return None;
        }

        // Directive forms: `v-model`, `v-show`, `v-on:click`, custom `v-foo:arg`.
        match self.try_directive_attribute(attr, &loc, on_component) {
            DirectiveAttributeLowering::Lowered(directive) => return Some(directive),
            DirectiveAttributeLowering::Rejected => return None,
            DirectiveAttributeLowering::NotDirective => {}
        }

        if let Some(prop) = self.transform_on_attribute(attr, loc.clone()) {
            return Some(prop);
        }

        let name: &'a str = self.intern_attr_name(attr.name.span());
        let name_loc = self.mapper().location(attr.name.span());
        let prop = match attr.value.as_ref() {
            None => self.valueless_attr(name, attr.name.span(), name_loc, loc),
            Some(JSXAttributeValue::StringLiteral(string)) => {
                let txt = self.bump().alloc_str(string.value.as_str());
                let value = TextNode::new(txt, self.mapper().location(string.span));
                PropNode::Attribute(Box::new_in(
                    AttributeNode {
                        name,
                        name_loc,
                        value: Some(value),
                        loc,
                    },
                    &self.bump(),
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
                        if let Some((ev, mods)) = split_on_event_modifiers(name) {
                            self.von_modifier_prop(&ev, attr.name.span(), span, &mods, loc)
                        } else {
                            self.bind_prop(name, attr.name.span(), span, loc)
                        }
                    }
                }
            }
            Some(JSXAttributeValue::Element(el)) => {
                self.bind_prop(name, attr.name.span(), el.span(), loc)
            }
            Some(JSXAttributeValue::Fragment(f)) => {
                self.bind_prop(name, attr.name.span(), f.span(), loc)
            }
        };
        Some(prop)
    }

    fn boolean_attr(
        &self,
        name: &'a str,
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
            &self.bump(),
        ))
    }

    /// `name={expr}` -> `v-bind:name="expr"`.
    fn bind_prop(
        &self,
        name: &'a str,
        name_span: Span,
        value_span: Span,
        loc: SourceLocation,
    ) -> PropNode<'a> {
        let mut directive = DirectiveNode::new(self.bump(), "bind", loc);
        directive.arg = Some(self.static_expr(name, name_span));
        directive.exp = Some(self.dyn_expr(value_span));
        PropNode::Directive(self.boxed(directive))
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
        directive.arg = Some(self.static_expr(self.bump().alloc_str(event), name_span));
        directive.exp = Some(self.dyn_expr(value_span));
        for modifier in mods {
            directive.modifiers.push(SimpleExpressionNode::new(
                self.bump().alloc_str(modifier),
                false,
                self.mapper().location(name_span),
            ));
        }
        PropNode::Directive(self.boxed(directive))
    }

    fn try_directive_attribute(
        &mut self,
        attr: &JSXAttribute<'_>,
        loc: &SourceLocation,
        on_component: bool,
    ) -> DirectiveAttributeLowering<'a> {
        let (raw_name, arg) = match &attr.name {
            JSXAttributeName::NamespacedName(named) => {
                let Some(raw_name) = self
                    .mapper()
                    .slice(named.namespace.span())
                    .strip_prefix("v-")
                else {
                    return DirectiveAttributeLowering::NotDirective;
                };
                let arg_name = self.mapper().slice(named.name.span());
                (raw_name, Some((arg_name, named.name.span())))
            }
            JSXAttributeName::Identifier(id) => {
                let Some(raw_name) = self.mapper().slice(id.span()).strip_prefix("v-") else {
                    return DirectiveAttributeLowering::NotDirective;
                };
                (raw_name, None)
            }
        };

        // Babel encodes `v-model:foo.trim` as `v-model:foo_trim` because JSX
        // attribute names cannot contain dots. This spelling is special only
        // for Babel-compatible plain elements; component arguments retain the
        // authored underscore name and their established lowering.
        if self.uses_babel_vdom_compat()
            && !on_component
            && raw_name == "model"
            && let Some((arg_name, arg_span)) = arg
            && let Some((model_arg, suffix_mods)) = split_model_arg_modifiers(arg_name)
        {
            let mut directive = DirectiveNode::new(self.bump(), "model", loc.clone());
            directive.arg = Some(self.static_expr(model_arg, arg_span));
            directive.exp = self.directive_value_expr(attr.value.as_ref());
            for modifier in suffix_mods {
                directive
                    .modifiers
                    .push(SimpleExpressionNode::new(modifier, false, loc.clone()));
            }
            return DirectiveAttributeLowering::Lowered(PropNode::Directive(Box::new_in(
                directive,
                &self.bump(),
            )));
        }

        // `v-model_lazy` / `v-model_number_lazy` — babel-plugin-jsx encodes
        // v-model modifiers as `_<mod>` name suffixes (JSX attribute names cannot
        // contain `.`). Strip the suffixes and lower as a `model` directive with
        // those modifiers, NOT a `model_lazy` custom directive.
        if let Some((directive_name, suffix_mods)) = split_underscore_model_modifiers(raw_name) {
            let mut directive = DirectiveNode::new(self.bump(), directive_name, loc.clone());
            if let Some((arg_name, arg_span)) = arg {
                directive.arg = Some(self.static_expr(arg_name, arg_span));
            }
            directive.exp = self.directive_value_expr(attr.value.as_ref());
            for modifier in suffix_mods {
                directive
                    .modifiers
                    .push(SimpleExpressionNode::new(modifier, false, loc.clone()));
            }
            return DirectiveAttributeLowering::Lowered(PropNode::Directive(Box::new_in(
                directive,
                &self.bump(),
            )));
        }

        // `v-model={[value, ['trim']]}` — babel-plugin-jsx encodes the model
        // expression, an optional string arg (component prop name), and a
        // trailing modifiers array as an array literal. Destructure it instead of
        // treating the whole array as the bound expression (which produced a
        // malformed `$event => ($event => (...))` chain).
        if raw_name == "model"
            && arg.is_none()
            && let Some(array) = self.array_literal_value(attr.value.as_ref())
        {
            match self.lower_model_array(array, loc, self.uses_babel_vdom_compat() && on_component)
            {
                ModelArrayLowering::Lowered(prop) => {
                    return DirectiveAttributeLowering::Lowered(prop);
                }
                ModelArrayLowering::Rejected => return DirectiveAttributeLowering::Rejected,
                ModelArrayLowering::Unrecognized => {}
            }
        }

        // Babel lowers `v-text` to a raw `textContent` prop. Keep Vize's
        // established `_toDisplayString` behavior outside the explicit
        // compatibility mode.
        if raw_name == "text"
            && arg.is_none()
            && let Some(prop) = self.compat_v_text_prop(attr, loc.clone())
        {
            return DirectiveAttributeLowering::Lowered(prop);
        }

        // `v-custom={[value, 'arg', ['a','b']]}` — babel-plugin-jsx's array
        // encoding for a custom directive's value, argument and modifiers.
        // Restricted to custom directives with no JSX-namespace argument: the
        // built-ins carry their own array meanings (`v-model` just above,
        // `v-models` before `lower_attribute` even runs), and when the argument
        // is already spelled `v-custom:arg` there is no positional slot for the
        // array to fill.
        if arg.is_none()
            && !is_builtin_directive(raw_name)
            && raw_name != "models"
            && let Some(array) = self.array_literal_value(attr.value.as_ref())
            && let Some(prop) = self.lower_custom_directive_array(array, raw_name, loc)
        {
            return DirectiveAttributeLowering::Lowered(prop);
        }

        let mut directive = DirectiveNode::new(self.bump(), raw_name, loc.clone());
        if let Some((arg_name, arg_span)) = arg {
            directive.arg = Some(self.static_expr(arg_name, arg_span));
        }
        directive.exp = self.directive_value_expr(attr.value.as_ref());
        DirectiveAttributeLowering::Lowered(PropNode::Directive(Box::new_in(
            directive,
            &self.bump(),
        )))
    }

    fn directive_value_expr(
        &self,
        value: Option<&JSXAttributeValue<'_>>,
    ) -> Option<vize_relief::ExpressionNode<'a>> {
        match value? {
            JSXAttributeValue::StringLiteral(string) => {
                Some(self.static_expr(self.bump().alloc_str(string.value.as_str()), string.span))
            }
            JSXAttributeValue::ExpressionContainer(container) => {
                container_expr_span(container).map(|span| self.dyn_expr(span))
            }
            JSXAttributeValue::Element(element) => Some(self.dyn_expr(element.span())),
            JSXAttributeValue::Fragment(fragment) => Some(self.dyn_expr(fragment.span())),
        }
    }
}
