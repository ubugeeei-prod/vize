use super::{MarkupDocumentVisitor, MarkupElement, MarkupRule};
use crate::lent_walk::{self, LentNodes};
use oxc_ast::ast::{
    Expression, JSXAttributeItem, JSXAttributeValue, JSXElement, JSXExpressionContainer,
    JSXFragment, Program,
};

/// Hands each outermost JSX element or fragment to `visitor` without walking
/// into it; the markup recursion visits what it contains.
struct Roots<'visitor, F> {
    visitor: &'visitor mut F,
    offset: u32,
}

impl<'a, F: FnMut(MarkupElement<'a>)> LentNodes<'a> for Roots<'_, F> {
    #[inline]
    fn enter_jsx_element(&mut self, element: &'a JSXElement<'a>) -> bool {
        (self.visitor)(MarkupElement::from_jsx_element(element, self.offset));
        false
    }

    #[inline]
    fn enter_jsx_fragment(&mut self, fragment: &'a JSXFragment<'a>) -> bool {
        (self.visitor)(MarkupElement::from_jsx_fragment(fragment, self.offset));
        false
    }
}

/// Hands every JSX element and fragment to `enter` before its contents and to
/// `exit` after them.
struct Tree<'enter, 'exit, FEnter, FExit> {
    offset: u32,
    enter: &'enter mut FEnter,
    exit: &'exit mut FExit,
}

impl<'a, FEnter, FExit> LentNodes<'a> for Tree<'_, '_, FEnter, FExit>
where
    FEnter: FnMut(MarkupElement<'a>),
    FExit: FnMut(MarkupElement<'a>),
{
    #[inline]
    fn enter_jsx_element(&mut self, element: &'a JSXElement<'a>) -> bool {
        (self.enter)(MarkupElement::from_jsx_element(element, self.offset));
        true
    }

    #[inline]
    fn leave_jsx_element(&mut self, element: &'a JSXElement<'a>) {
        (self.exit)(MarkupElement::from_jsx_element(element, self.offset));
    }

    #[inline]
    fn enter_jsx_fragment(&mut self, fragment: &'a JSXFragment<'a>) -> bool {
        (self.enter)(MarkupElement::from_jsx_fragment(fragment, self.offset));
        true
    }

    #[inline]
    fn leave_jsx_fragment(&mut self, fragment: &'a JSXFragment<'a>) {
        (self.exit)(MarkupElement::from_jsx_fragment(fragment, self.offset));
    }
}

pub(super) fn walk_jsx_program<'a>(
    program: &'a Program<'a>,
    offset: u32,
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    lent_walk::walk_program(
        &mut Tree {
            offset,
            enter,
            exit,
        },
        program,
    );
}

/// Each outermost JSX element or fragment of `program`, in source order.
pub(super) fn walk_program_roots<'a>(
    program: &'a Program<'a>,
    offset: u32,
    visitor: &mut impl FnMut(MarkupElement<'a>),
) {
    lent_walk::walk_program(&mut Roots { visitor, offset }, program);
}

pub(super) fn visit_expression_container_roots<'rule, 'ctx, 'mc, 'a, R: MarkupRule + ?Sized>(
    visitor: &mut MarkupDocumentVisitor<'rule, 'ctx, 'mc, 'a, R>,
    container: &'a JSXExpressionContainer<'a>,
    offset: u32,
) {
    walk_expression_container_roots(container, offset, &mut |element| {
        visitor.visit_element(element);
    });
}

pub(super) fn walk_expression_container_roots<'a>(
    container: &'a JSXExpressionContainer<'a>,
    offset: u32,
    visitor: &mut impl FnMut(MarkupElement<'a>),
) {
    lent_walk::walk_jsx_expression_container(&mut Roots { visitor, offset }, container);
}

fn walk_expression_roots<'a>(
    expression: &'a Expression<'a>,
    offset: u32,
    visitor: &mut impl FnMut(MarkupElement<'a>),
) {
    lent_walk::walk_expression(&mut Roots { visitor, offset }, expression);
}

fn walk_attribute_value_roots<'a>(
    value: &'a JSXAttributeValue<'a>,
    offset: u32,
    visitor: &mut impl FnMut(MarkupElement<'a>),
) {
    match value {
        JSXAttributeValue::StringLiteral(_) => {}
        JSXAttributeValue::ExpressionContainer(container) => {
            walk_expression_container_roots(container, offset, visitor);
        }
        JSXAttributeValue::Element(element) => {
            visitor(MarkupElement::from_jsx_element(element, offset));
        }
        JSXAttributeValue::Fragment(fragment) => {
            visitor(MarkupElement::from_jsx_fragment(fragment, offset));
        }
    }
}

pub(super) fn visit_attribute_roots<'rule, 'ctx, 'mc, 'a, R: MarkupRule + ?Sized>(
    visitor: &mut MarkupDocumentVisitor<'rule, 'ctx, 'mc, 'a, R>,
    element: &'a JSXElement<'a>,
    offset: u32,
) {
    walk_attribute_roots(element, offset, &mut |element| {
        visitor.visit_element(element);
    });
}

fn walk_attribute_roots<'a>(
    element: &'a JSXElement<'a>,
    offset: u32,
    visitor: &mut impl FnMut(MarkupElement<'a>),
) {
    for item in &element.opening_element.attributes {
        match item {
            JSXAttributeItem::Attribute(attribute) => {
                if let Some(value) = attribute.value.as_ref() {
                    walk_attribute_value_roots(value, offset, visitor);
                }
            }
            JSXAttributeItem::SpreadAttribute(spread) => {
                walk_expression_roots(&spread.argument, offset, visitor);
            }
        }
    }
}
