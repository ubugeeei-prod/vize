//! Original-Program JSX references through the same bounded resolver traversal.

use super::{ReferenceSink, ResolutionError, ResolutionErrorKind, Resolver, Usage};
use oxc_ast::ast::{
    JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement, JSXElementName,
    JSXExpression, JSXExpressionContainer, JSXFragment, JSXMemberExpression,
    JSXMemberExpressionObject,
};
use oxc_span::GetSpan;

impl<'a, S: ReferenceSink<'a>> Resolver<'a, '_, S> {
    pub(super) fn jsx_element(
        &mut self,
        element: &JSXElement<'a>,
        depth: usize,
    ) -> Result<(), ResolutionError> {
        if !self.source.allows_jsx() {
            return Err(self.fail(element.span, ResolutionErrorKind::UnsupportedSyntax));
        }
        self.visit(element.span, depth)?;
        let next = depth + 1;
        let opening = &element.opening_element;
        self.visit(opening.span, next)?;
        if opening.type_arguments.is_some() {
            return Err(self.fail(opening.span, ResolutionErrorKind::UnsupportedSyntax));
        }
        self.jsx_tag_name(&opening.name, next, true)?;
        for attribute in &opening.attributes {
            self.jsx_attribute(attribute, next)?;
        }
        self.jsx_children(&element.children, next)?;
        if let Some(closing) = &element.closing_element {
            self.visit(closing.span, next)?;
            self.jsx_tag_name(&closing.name, next, false)?;
        }
        Ok(())
    }

    pub(super) fn jsx_fragment(
        &mut self,
        fragment: &JSXFragment<'a>,
        depth: usize,
    ) -> Result<(), ResolutionError> {
        if !self.source.allows_jsx() {
            return Err(self.fail(fragment.span, ResolutionErrorKind::UnsupportedSyntax));
        }
        self.visit(fragment.span, depth)?;
        let next = depth + 1;
        self.visit(fragment.opening_fragment.span, next)?;
        self.jsx_children(&fragment.children, next)?;
        self.visit(fragment.closing_fragment.span, next)?;
        Ok(())
    }

    fn jsx_name_span(&mut self, span: oxc_span::Span, depth: usize) -> Result<(), ResolutionError> {
        let checked = self.visit(span, depth)?;
        if checked.start == checked.end {
            return Err(self.fail(span, ResolutionErrorKind::InvalidSpan));
        }
        Ok(())
    }

    fn jsx_tag_name(
        &mut self,
        name: &JSXElementName<'a>,
        depth: usize,
        opening: bool,
    ) -> Result<(), ResolutionError> {
        match name {
            JSXElementName::Identifier(name) => self.jsx_name_span(name.span, depth),
            JSXElementName::IdentifierReference(name) if opening => {
                self.identifier(name, Usage::Read, false)
            }
            JSXElementName::IdentifierReference(name) => self.jsx_name_span(name.span, depth),
            JSXElementName::MemberExpression(member) => self.jsx_member(member, depth, opening),
            name => Err(self.fail(name.span(), ResolutionErrorKind::UnsupportedSyntax)),
        }
    }

    fn jsx_member(
        &mut self,
        member: &JSXMemberExpression<'a>,
        depth: usize,
        opening: bool,
    ) -> Result<(), ResolutionError> {
        self.visit(member.span, depth)?;
        let next = depth + 1;
        match &member.object {
            JSXMemberExpressionObject::IdentifierReference(name) if opening => {
                self.identifier(name, Usage::Read, false)?;
            }
            JSXMemberExpressionObject::IdentifierReference(name) => {
                self.jsx_name_span(name.span, next)?;
            }
            JSXMemberExpressionObject::MemberExpression(object) => {
                self.jsx_member(object, next, opening)?;
            }
            object => {
                return Err(self.fail(object.span(), ResolutionErrorKind::UnsupportedSyntax));
            }
        }
        self.jsx_name_span(member.property.span, next)
    }

    fn jsx_attribute(
        &mut self,
        attribute: &JSXAttributeItem<'a>,
        depth: usize,
    ) -> Result<(), ResolutionError> {
        self.visit(attribute.span(), depth)?;
        let next = depth + 1;
        match attribute {
            JSXAttributeItem::SpreadAttribute(value) => self.expression(&value.argument, next),
            JSXAttributeItem::Attribute(value) => {
                match &value.name {
                    JSXAttributeName::Identifier(name) => self.jsx_name_span(name.span, next)?,
                    name => {
                        return Err(self.fail(name.span(), ResolutionErrorKind::UnsupportedSyntax));
                    }
                }
                match &value.value {
                    None => Ok(()),
                    Some(JSXAttributeValue::StringLiteral(value)) => {
                        self.visit(value.span, next)?;
                        Ok(())
                    }
                    Some(JSXAttributeValue::ExpressionContainer(value)) => {
                        self.jsx_container(value, next)
                    }
                    Some(JSXAttributeValue::Element(value)) => self.jsx_element(value, next),
                    Some(JSXAttributeValue::Fragment(value)) => self.jsx_fragment(value, next),
                }
            }
        }
    }

    fn jsx_container(
        &mut self,
        container: &JSXExpressionContainer<'a>,
        depth: usize,
    ) -> Result<(), ResolutionError> {
        self.visit(container.span, depth)?;
        match &container.expression {
            JSXExpression::EmptyExpression(value) => {
                self.visit(value.span, depth + 1)?;
                Ok(())
            }
            value => match value.as_expression() {
                Some(expression) => self.expression(expression, depth + 1),
                None => Err(self.fail(value.span(), ResolutionErrorKind::UnsupportedSyntax)),
            },
        }
    }

    fn jsx_children(
        &mut self,
        children: &[JSXChild<'a>],
        depth: usize,
    ) -> Result<(), ResolutionError> {
        for child in children {
            self.visit(child.span(), depth)?;
            match child {
                JSXChild::Text(_) => {}
                JSXChild::Element(value) => self.jsx_element(value, depth + 1)?,
                JSXChild::Fragment(value) => self.jsx_fragment(value, depth + 1)?,
                JSXChild::ExpressionContainer(value) => self.jsx_container(value, depth + 1)?,
                JSXChild::Spread(value) => self.expression(&value.expression, depth + 1)?,
            }
        }
        Ok(())
    }
}
