use super::{MarkupDocumentVisitor, MarkupElement, MarkupRule};
use oxc_ast::ast::{
    Expression, JSXAttributeItem, JSXAttributeValue, JSXElement, JSXExpressionContainer,
    JSXFragment, Program,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_expression, walk_jsx_element, walk_jsx_expression_container, walk_jsx_fragment,
        walk_program,
    },
};
use std::marker::PhantomData;

struct NestedDriver<'visitor, 'a, F: ?Sized> {
    visitor: &'visitor mut F,
    offset: u32,
    _marker: PhantomData<&'a ()>,
}

impl<'a, F: ?Sized> NestedDriver<'_, 'a, F>
where
    F: FnMut(MarkupElement<'a>),
{
    fn visit_element(&mut self, element: MarkupElement<'a>) {
        (self.visitor)(element);
    }
}

impl<'a, F: ?Sized> Visit<'a> for NestedDriver<'_, 'a, F>
where
    F: FnMut(MarkupElement<'a>),
{
    fn visit_jsx_element(&mut self, it: &JSXElement<'a>) {
        self.visit_element(MarkupElement::from_jsx_element(it as *const _, self.offset));
    }

    fn visit_jsx_fragment(&mut self, it: &JSXFragment<'a>) {
        self.visit_element(MarkupElement::from_jsx_fragment(
            it as *const _,
            self.offset,
        ));
    }
}

pub(super) fn walk_jsx_program<'a>(
    program: &'a Program<'a>,
    offset: u32,
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    struct JsxMarkupWalker<'enter, 'exit, FEnter, FExit> {
        offset: u32,
        enter: &'enter mut FEnter,
        exit: &'exit mut FExit,
    }

    impl<'ast, FEnter, FExit> Visit<'ast> for JsxMarkupWalker<'_, '_, FEnter, FExit>
    where
        FEnter: FnMut(MarkupElement<'ast>),
        FExit: FnMut(MarkupElement<'ast>),
    {
        fn visit_jsx_element(&mut self, it: &JSXElement<'ast>) {
            let element = MarkupElement::from_jsx_element(it as *const _, self.offset);
            (self.enter)(element);
            walk_jsx_element(self, it);
            (self.exit)(element);
        }

        fn visit_jsx_fragment(&mut self, it: &JSXFragment<'ast>) {
            let element = MarkupElement::from_jsx_fragment(it as *const _, self.offset);
            (self.enter)(element);
            walk_jsx_fragment(self, it);
            (self.exit)(element);
        }
    }

    let mut walker = JsxMarkupWalker {
        offset,
        enter,
        exit,
    };
    walk_program(&mut walker, program);
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

fn walk_expression_container_roots<'a>(
    container: &'a JSXExpressionContainer<'a>,
    offset: u32,
    visitor: &mut impl FnMut(MarkupElement<'a>),
) {
    let mut driver = NestedDriver {
        visitor,
        offset,
        _marker: PhantomData,
    };
    walk_jsx_expression_container(&mut driver, container);
}

fn walk_expression_roots<'a>(
    expression: &'a Expression<'a>,
    offset: u32,
    visitor: &mut impl FnMut(MarkupElement<'a>),
) {
    let mut driver = NestedDriver {
        visitor,
        offset,
        _marker: PhantomData,
    };
    walk_expression(&mut driver, expression);
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
            visitor(MarkupElement::from_jsx_element(
                &**element as *const _,
                offset,
            ));
        }
        JSXAttributeValue::Fragment(fragment) => {
            visitor(MarkupElement::from_jsx_fragment(
                &**fragment as *const _,
                offset,
            ));
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
