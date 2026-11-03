//! Original-Program JSX references through the same bounded resolver traversal.

use super::{ReferenceSink, ResolutionError, ResolutionErrorKind, Resolver, Usage};
use crate::resolution::{SyntaxEdge, SyntaxKind};
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
        self.syntax(SyntaxKind::Element, SyntaxEdge::Enter, element.span)?;
        let next = depth + 1;
        let opening = &element.opening_element;
        self.visit(opening.span, next)?;
        if opening.type_arguments.is_some() {
            return Err(self.fail(opening.span, ResolutionErrorKind::UnsupportedSyntax));
        }
        let header = SyntaxKind::Opening {
            self_closing: element.closing_element.is_none(),
        };
        self.syntax(header, SyntaxEdge::Enter, opening.span)?;
        self.jsx_tag_name(&opening.name, next, true)?;
        for attribute in &opening.attributes {
            self.jsx_attribute(attribute, next)?;
        }
        self.syntax(header, SyntaxEdge::Leave, opening.span)?;
        self.jsx_children(&element.children, next)?;
        if let Some(closing) = &element.closing_element {
            self.visit(closing.span, next)?;
            self.syntax(SyntaxKind::Closing, SyntaxEdge::Enter, closing.span)?;
            self.jsx_tag_name(&closing.name, next, false)?;
            self.syntax(SyntaxKind::Closing, SyntaxEdge::Leave, closing.span)?;
        }
        self.syntax(SyntaxKind::Element, SyntaxEdge::Leave, element.span)?;
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
        self.syntax(SyntaxKind::Fragment, SyntaxEdge::Enter, fragment.span)?;
        let next = depth + 1;
        self.visit(fragment.opening_fragment.span, next)?;
        self.syntax(
            SyntaxKind::FragmentOpening,
            SyntaxEdge::Leaf,
            fragment.opening_fragment.span,
        )?;
        self.jsx_children(&fragment.children, next)?;
        self.visit(fragment.closing_fragment.span, next)?;
        self.syntax(
            SyntaxKind::FragmentClosing,
            SyntaxEdge::Leaf,
            fragment.closing_fragment.span,
        )?;
        self.syntax(SyntaxKind::Fragment, SyntaxEdge::Leave, fragment.span)?;
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
            JSXElementName::Identifier(name) => {
                self.jsx_name_span(name.span, depth)?;
                self.syntax(
                    SyntaxKind::Intrinsic(name.name.as_str()),
                    SyntaxEdge::Leaf,
                    name.span,
                )
            }
            JSXElementName::IdentifierReference(name) if opening => {
                let kind = SyntaxKind::Component(name.name.as_str());
                self.syntax(kind, SyntaxEdge::Enter, name.span)?;
                self.identifier(name, Usage::Read, false)?;
                self.syntax(kind, SyntaxEdge::Leave, name.span)
            }
            JSXElementName::IdentifierReference(name) => {
                self.jsx_name_span(name.span, depth)?;
                self.syntax(
                    SyntaxKind::Component(name.name.as_str()),
                    SyntaxEdge::Leaf,
                    name.span,
                )
            }
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
        self.syntax(SyntaxKind::Member, SyntaxEdge::Enter, member.span)?;
        let next = depth + 1;
        match &member.object {
            JSXMemberExpressionObject::IdentifierReference(name) if opening => {
                let kind = SyntaxKind::Component(name.name.as_str());
                self.syntax(kind, SyntaxEdge::Enter, name.span)?;
                self.identifier(name, Usage::Read, false)?;
                self.syntax(kind, SyntaxEdge::Leave, name.span)?;
            }
            JSXMemberExpressionObject::IdentifierReference(name) => {
                self.jsx_name_span(name.span, next)?;
                self.syntax(
                    SyntaxKind::Component(name.name.as_str()),
                    SyntaxEdge::Leaf,
                    name.span,
                )?;
            }
            JSXMemberExpressionObject::MemberExpression(object) => {
                self.jsx_member(object, next, opening)?;
            }
            object => {
                return Err(self.fail(object.span(), ResolutionErrorKind::UnsupportedSyntax));
            }
        }
        self.jsx_name_span(member.property.span, next)?;
        self.syntax(
            SyntaxKind::Property(member.property.name.as_str()),
            SyntaxEdge::Leaf,
            member.property.span,
        )?;
        self.syntax(SyntaxKind::Member, SyntaxEdge::Leave, member.span)
    }

    fn jsx_attribute(
        &mut self,
        attribute: &JSXAttributeItem<'a>,
        depth: usize,
    ) -> Result<(), ResolutionError> {
        self.visit(attribute.span(), depth)?;
        let next = depth + 1;
        match attribute {
            JSXAttributeItem::SpreadAttribute(value) => {
                self.syntax(SyntaxKind::SpreadAttribute, SyntaxEdge::Enter, value.span)?;
                self.expression(&value.argument, next)?;
                self.syntax(SyntaxKind::SpreadAttribute, SyntaxEdge::Leave, value.span)
            }
            JSXAttributeItem::Attribute(value) => {
                let name = match &value.name {
                    JSXAttributeName::Identifier(name) => {
                        self.jsx_name_span(name.span, next)?;
                        name.name.as_str()
                    }
                    name => {
                        return Err(self.fail(name.span(), ResolutionErrorKind::UnsupportedSyntax));
                    }
                };
                let kind = SyntaxKind::Attribute(name);
                self.syntax(kind, SyntaxEdge::Enter, value.span)?;
                self.syntax(
                    SyntaxKind::AttributeName(name),
                    SyntaxEdge::Leaf,
                    value.name.span(),
                )?;
                match &value.value {
                    None => {}
                    Some(JSXAttributeValue::StringLiteral(value)) => {
                        self.visit(value.span, next)?;
                        self.syntax(
                            SyntaxKind::AttributeString(value.value.as_str()),
                            SyntaxEdge::Leaf,
                            value.span,
                        )?;
                    }
                    Some(JSXAttributeValue::ExpressionContainer(value)) => {
                        self.jsx_container(value, next)?;
                    }
                    Some(JSXAttributeValue::Element(value)) => self.jsx_element(value, next)?,
                    Some(JSXAttributeValue::Fragment(value)) => self.jsx_fragment(value, next)?,
                }
                self.syntax(kind, SyntaxEdge::Leave, value.span)
            }
        }
    }

    fn jsx_container(
        &mut self,
        container: &JSXExpressionContainer<'a>,
        depth: usize,
    ) -> Result<(), ResolutionError> {
        self.visit(container.span, depth)?;
        self.syntax(SyntaxKind::Container, SyntaxEdge::Enter, container.span)?;
        match &container.expression {
            JSXExpression::EmptyExpression(value) => {
                self.visit(value.span, depth + 1)?;
                self.syntax(SyntaxKind::Empty, SyntaxEdge::Leaf, value.span)?;
            }
            value => match value.as_expression() {
                Some(expression) => self.expression(expression, depth + 1)?,
                None => return Err(self.fail(value.span(), ResolutionErrorKind::UnsupportedSyntax)),
            },
        }
        self.syntax(SyntaxKind::Container, SyntaxEdge::Leave, container.span)
    }

    fn jsx_children(
        &mut self,
        children: &[JSXChild<'a>],
        depth: usize,
    ) -> Result<(), ResolutionError> {
        for child in children {
            self.visit(child.span(), depth)?;
            match child {
                JSXChild::Text(value) => {
                    self.syntax(
                        SyntaxKind::Text(value.value.as_str()),
                        SyntaxEdge::Leaf,
                        value.span,
                    )?;
                }
                JSXChild::Element(value) => self.jsx_element(value, depth + 1)?,
                JSXChild::Fragment(value) => self.jsx_fragment(value, depth + 1)?,
                JSXChild::ExpressionContainer(value) => self.jsx_container(value, depth + 1)?,
                JSXChild::Spread(value) => {
                    self.syntax(SyntaxKind::SpreadChild, SyntaxEdge::Enter, value.span)?;
                    self.expression(&value.expression, depth + 1)?;
                    self.syntax(SyntaxKind::SpreadChild, SyntaxEdge::Leave, value.span)?;
                }
            }
        }
        Ok(())
    }
}
