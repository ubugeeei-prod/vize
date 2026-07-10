use super::model::{ReliefElementKind, ReliefExpressionRef, ReliefModifiers, ReliefSpan};
use vize_relief::{
    AttributeNode, DirectiveNode, ElementNode, ForNode, IfBranchNode, IfNode, PropNode,
    TemplateChildNode, TextCallContent, TextNode,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ReliefRenderOp<'a> {
    Element {
        tag: &'a str,
        kind: ReliefElementKind,
        span: ReliefSpan,
    },
    Attribute {
        name: &'a str,
        name_span: ReliefSpan,
        value: Option<&'a str>,
        value_span: Option<ReliefSpan>,
        span: ReliefSpan,
    },
    Directive {
        name: &'a str,
        arg: Option<ReliefExpressionRef<'a>>,
        exp: Option<ReliefExpressionRef<'a>>,
        modifiers: ReliefModifiers<'a>,
        span: ReliefSpan,
    },
    Text {
        content: &'a str,
        span: ReliefSpan,
    },
    Comment {
        content: &'a str,
        is_directive: bool,
        span: ReliefSpan,
    },
    Interpolation {
        expression: ReliefExpressionRef<'a>,
        raw: bool,
        span: ReliefSpan,
    },
    If {
        span: ReliefSpan,
    },
    IfBranch {
        condition: Option<ReliefExpressionRef<'a>>,
        span: ReliefSpan,
    },
    For {
        source: ReliefExpressionRef<'a>,
        value: Option<ReliefExpressionRef<'a>>,
        key: Option<ReliefExpressionRef<'a>>,
        index: Option<ReliefExpressionRef<'a>>,
        span: ReliefSpan,
    },
    TextCall {
        content: Option<ReliefExpressionRef<'a>>,
        span: ReliefSpan,
    },
    CompoundExpression {
        expression: ReliefExpressionRef<'a>,
        span: ReliefSpan,
    },
    HoistRef {
        index: usize,
    },
}

impl<'a> ReliefRenderOp<'a> {
    pub fn from_prop(prop: &'a PropNode<'a>) -> Self {
        match prop {
            PropNode::Attribute(attribute) => Self::from_attribute(attribute),
            PropNode::Directive(directive) => Self::from_directive(directive),
        }
    }

    pub fn from_element(element: &'a ElementNode<'a>) -> Self {
        Self::Element {
            tag: element.tag.as_str(),
            kind: element.tag_type.into(),
            span: ReliefSpan::from_location(&element.loc),
        }
    }

    pub fn from_text(text: &'a TextNode) -> Self {
        Self::Text {
            content: text.content.as_str(),
            span: ReliefSpan::from_location(&text.loc),
        }
    }

    pub fn from_if(node: &'a IfNode<'a>) -> Self {
        Self::If {
            span: ReliefSpan::from_location(&node.loc),
        }
    }

    pub fn from_if_branch(branch: &'a IfBranchNode<'a>) -> Self {
        Self::IfBranch {
            condition: branch
                .condition
                .as_ref()
                .map(ReliefExpressionRef::from_expression),
            span: ReliefSpan::from_location(&branch.loc),
        }
    }

    pub fn from_for(node: &'a ForNode<'a>) -> Self {
        Self::For {
            source: ReliefExpressionRef::from_expression(&node.source),
            value: node
                .value_alias
                .as_ref()
                .map(ReliefExpressionRef::from_expression),
            key: node
                .key_alias
                .as_ref()
                .map(ReliefExpressionRef::from_expression),
            index: node
                .object_index_alias
                .as_ref()
                .map(ReliefExpressionRef::from_expression),
            span: ReliefSpan::from_location(&node.loc),
        }
    }

    fn from_attribute(attribute: &'a AttributeNode) -> Self {
        Self::Attribute {
            name: attribute.name.as_str(),
            name_span: ReliefSpan::from_location(&attribute.name_loc),
            value: attribute.value.as_ref().map(|value| value.content.as_str()),
            value_span: attribute
                .value
                .as_ref()
                .map(|value| ReliefSpan::from_location(&value.loc)),
            span: ReliefSpan::from_location(&attribute.loc),
        }
    }

    pub fn from_directive(directive: &'a DirectiveNode<'a>) -> Self {
        Self::Directive {
            name: directive.name.as_str(),
            arg: directive
                .arg
                .as_ref()
                .map(ReliefExpressionRef::from_expression),
            exp: directive
                .exp
                .as_ref()
                .map(ReliefExpressionRef::from_expression),
            modifiers: ReliefModifiers::new(directive.modifiers.as_slice()),
            span: ReliefSpan::from_location(&directive.loc),
        }
    }

    pub fn from_template_child(child: &'a TemplateChildNode<'a>) -> Self {
        match child {
            TemplateChildNode::Element(element) => Self::from_element(element),
            TemplateChildNode::Text(text) => Self::from_text(text),
            TemplateChildNode::Comment(comment) => Self::Comment {
                content: comment.content.as_str(),
                is_directive: comment.directive.is_some(),
                span: ReliefSpan::from_location(&comment.loc),
            },
            TemplateChildNode::Interpolation(interpolation) => Self::Interpolation {
                expression: ReliefExpressionRef::from_expression(&interpolation.content),
                #[cfg(feature = "legacy")]
                raw: interpolation.raw,
                #[cfg(not(feature = "legacy"))]
                raw: false,
                span: ReliefSpan::from_location(&interpolation.loc),
            },
            TemplateChildNode::If(node) => Self::from_if(node),
            TemplateChildNode::IfBranch(branch) => Self::from_if_branch(branch),
            TemplateChildNode::For(node) => Self::from_for(node),
            TemplateChildNode::TextCall(node) => Self::TextCall {
                content: match &node.content {
                    TextCallContent::Text(_) => None,
                    TextCallContent::Interpolation(interpolation) => {
                        Some(ReliefExpressionRef::from_expression(&interpolation.content))
                    }
                    TextCallContent::Compound(compound) => {
                        Some(ReliefExpressionRef::Source(compound.loc.source.as_str()))
                    }
                },
                span: ReliefSpan::from_location(&node.loc),
            },
            TemplateChildNode::CompoundExpression(expression) => Self::CompoundExpression {
                expression: ReliefExpressionRef::Source(expression.loc.source.as_str()),
                span: ReliefSpan::from_location(&expression.loc),
            },
            TemplateChildNode::Hoisted(index) => Self::HoistRef { index: *index },
        }
    }
}
