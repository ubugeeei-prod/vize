//! Tags and children: `ui.element`, `ui.component`, `ui.text`, `ui.jsx-child`.
//! Expression children that are structural control flow go to
//! [`super::control_flow`].

use oxc_ast::ast::{JSXChild, JSXElement, JSXElementName, JSXExpression};
use oxc_ast_visit::Visit;
use oxc_span::GetSpan;
use vize_l0::{Box, Vec};
use vize_l2::artifact::OpFamily;
use vize_l2::op::{
    BindOp, BindingOp, ComponentOp, DynamicName, ElementOp, InterpolationOp, Namespace, Op, Region,
    TextOp,
};

use super::attributes::{Owner, Props, props};
use super::control_flow::control_flow;
use super::slots::{slot_children, v_slots_templates};
use super::{Cx, JsxRefusal, span};

/// The tag a dynamic component is rendered through.
const DYNAMIC_COMPONENT: &str = "component";

enum Tag<'a> {
    Element(&'a str),
    /// A resolved-by-name component (`<Foo/>` with no lexical binding).
    Named(&'a str),
    /// A component value: a bound identifier or a member expression,
    /// rendered as `<component is={value}>`.
    Value(oxc_span::Span),
}

pub(super) fn element<'a>(
    cx: &mut Cx<'a, '_>,
    element: &JSXElement<'_>,
) -> Result<Op<'a>, JsxRefusal> {
    cx.mint();
    let opening = &element.opening_element;
    let tag = tag(cx, &opening.name)?;
    let mut lowered = Props::new(cx.allocator);
    if let Tag::Value(value) = tag {
        cx.mint();
        lowered.bindings.push(BindingOp::Bind(Box::new_in(
            BindOp {
                name: Some(DynamicName::Static("is")),
                modifiers: Vec::new_in(&cx.allocator),
                value: Some(cx.expr(value)?),
                span: span(value),
            },
            &cx.allocator,
        )));
    }
    // A lowercase `<component is>` keeps element directives, as in templates.
    let owner = match tag {
        Tag::Element(tag) => Owner::Element {
            tag,
            items: &opening.attributes,
        },
        Tag::Named(_) | Tag::Value(_) => Owner::Component,
    };
    props(cx, &opening.attributes, owner, &mut lowered)?;
    let mut ops = Vec::new_in(&cx.allocator);
    match tag {
        Tag::Element(_) => element_children(cx, &element.children, &mut ops)?,
        // A component's children are slot templates for the slot idioms (a
        // sole function or object child), else its implicit default slot.
        Tag::Named(_) | Tag::Value(_) => {
            if !slot_children(cx, &element.children, &mut ops)? {
                push_children(cx, &element.children, &mut ops)?;
            }
            if lowered.has_slot_templates() {
                v_slots_templates(cx, &opening.attributes, &mut ops)?;
            }
        }
    }
    let children = Region { ops };
    let element_span = span(element.span);
    let name = match tag {
        Tag::Element(name) if !(name == DYNAMIC_COMPONENT && lowered.has_is()) => {
            return Ok(Op::Element(Box::new_in(
                ElementOp {
                    tag: name,
                    namespace: Namespace::Html,
                    attributes: lowered.attributes,
                    bindings: lowered.bindings,
                    children,
                    span: element_span,
                },
                &cx.allocator,
            )));
        }
        Tag::Element(name) | Tag::Named(name) => name,
        Tag::Value(_) => DYNAMIC_COMPONENT,
    };
    cx.observe(OpFamily::SlotCarrier);
    Ok(Op::Component(Box::new_in(
        ComponentOp {
            name,
            attributes: lowered.attributes,
            bindings: lowered.bindings,
            children,
            span: element_span,
        },
        &cx.allocator,
    )))
}

fn tag<'a>(cx: &Cx<'a, '_>, name: &JSXElementName<'_>) -> Result<Tag<'a>, JsxRefusal> {
    match name {
        JSXElementName::Identifier(identifier) => {
            let name = cx.slice(identifier.span)?;
            if name == "style" || !name.starts_with(|c: char| c.is_ascii_lowercase()) {
                return Err(JsxRefusal::Tag);
            }
            Ok(Tag::Element(name))
        }
        JSXElementName::IdentifierReference(reference) => {
            let name = cx.slice(reference.span)?;
            if name.starts_with(|c: char| c.is_ascii_lowercase()) {
                return Ok(Tag::Element(name));
            }
            let bound = name != "Fragment"
                && reference.reference_id.get().is_some_and(|reference| {
                    cx.file
                        .scoping
                        .is_some_and(|scoping| scoping.has_binding(reference))
                });
            Ok(if bound {
                Tag::Value(reference.span)
            } else {
                Tag::Named(name)
            })
        }
        JSXElementName::MemberExpression(member) => Ok(Tag::Value(member.span)),
        JSXElementName::NamespacedName(_) | JSXElementName::ThisExpression(_) => {
            Err(JsxRefusal::Tag)
        }
    }
}

/// An intrinsic element's children. A sole expression child that is not
/// control flow is a JSX child value, which the renderer normalizes.
fn element_children<'a>(
    cx: &mut Cx<'a, '_>,
    children: &[JSXChild<'_>],
    ops: &mut Vec<'a, Op<'a>>,
) -> Result<(), JsxRefusal> {
    let [JSXChild::ExpressionContainer(container)] = children else {
        return push_children(cx, children, ops);
    };
    let expression = match &container.expression {
        JSXExpression::EmptyExpression(_) | JSXExpression::StringLiteral(_) => {
            return push_children(cx, children, ops);
        }
        expression => expression,
    };
    let Some(expression) = expression.as_expression() else {
        return Err(JsxRefusal::Child);
    };
    if let Some(op) = control_flow(cx, expression, container.span)? {
        ops.push(op);
        return Ok(());
    }
    if contains_jsx(expression) {
        return Err(JsxRefusal::Child);
    }
    cx.mint();
    cx.observe(OpFamily::JsxChild);
    ops.push(Op::JsxChildValue(Box::new_in(
        InterpolationOp {
            expression: cx.expr(expression.span())?,
            raw_html: false,
            span: span(container.span),
        },
        &cx.allocator,
    )));
    Ok(())
}

/// Children in document order. A child fragment is spliced: it carries no
/// props and no key, so its children are the parent's at that position.
pub(super) fn push_children<'a>(
    cx: &mut Cx<'a, '_>,
    children: &[JSXChild<'_>],
    ops: &mut Vec<'a, Op<'a>>,
) -> Result<(), JsxRefusal> {
    for child in children {
        match child {
            JSXChild::Fragment(fragment) => push_children(cx, &fragment.children, ops)?,
            JSXChild::Text(text) => {
                let cleaned = clean_text(text.value.as_str());
                if !cleaned.is_empty() {
                    cx.mint();
                    ops.push(text_op(cx, cx.allocator.alloc_str(&cleaned), text.span));
                }
            }
            JSXChild::Element(nested) => ops.push(element(cx, nested)?),
            JSXChild::ExpressionContainer(container) => match &container.expression {
                JSXExpression::EmptyExpression(_) => {}
                JSXExpression::StringLiteral(string) => {
                    cx.mint();
                    let content = cx.allocator.alloc_str(string.value.as_str());
                    ops.push(text_op(cx, content, string.span));
                }
                expression => {
                    let lowered = match expression.as_expression() {
                        Some(expression) => control_flow(cx, expression, container.span)?,
                        None => None,
                    };
                    ops.push(lowered.ok_or(JsxRefusal::Child)?);
                }
            },
            JSXChild::Spread(_) => return Err(JsxRefusal::Child),
        }
    }
    Ok(())
}

fn text_op<'a>(cx: &Cx<'a, '_>, content: &'a str, at: oxc_span::Span) -> Op<'a> {
    Op::Text(Box::new_in(
        TextOp {
            content,
            span: span(at),
        },
        &cx.allocator,
    ))
}

pub(super) fn contains_jsx(expression: &oxc_ast::ast::Expression<'_>) -> bool {
    struct Finder(bool);
    impl<'x> Visit<'x> for Finder {
        fn visit_jsx_element(&mut self, _: &JSXElement<'x>) {
            self.0 = true;
        }
        fn visit_jsx_fragment(&mut self, _: &oxc_ast::ast::JSXFragment<'x>) {
            self.0 = true;
        }
    }
    let mut finder = Finder(false);
    finder.visit_expression(expression);
    finder.0
}

/// JSX text whitespace, the rule `@vue/babel-plugin-jsx` inherits from Babel:
/// lines are split, tabs become spaces, whitespace next to a line break is
/// trimmed, blank lines are dropped and the rest join with one space. The
/// first line's leading and the last line's trailing whitespace stay.
#[must_use]
pub fn clean_text(raw: &str) -> vize_l0::String {
    let lines = || {
        raw.split('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line))
            .enumerate()
    };
    let line_count = raw.split('\n').count();
    // Babel keeps `0` when every line is blank.
    let last_non_blank = lines()
        .filter(|(_, line)| line.bytes().any(|b| b != b' ' && b != b'\t'))
        .last()
        .map_or(0, |(index, _)| index);
    let mut result = vize_l0::String::default();
    for (index, line) in lines() {
        let normalized = line.replace('\t', " ");
        let mut trimmed: &str = &normalized;
        if index != 0 {
            trimmed = trimmed.trim_start_matches(' ');
        }
        if index + 1 != line_count {
            trimmed = trimmed.trim_end_matches(' ');
        }
        if trimmed.is_empty() {
            continue;
        }
        result.push_str(trimmed);
        if index != last_non_blank {
            result.push(' ');
        }
    }
    result
}
