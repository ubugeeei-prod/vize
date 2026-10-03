use super::{BuildResult, JsxDecisionKind as Decision, JsxIssue, JsxIssueKind as Issue};
use super::{NativeJsxAnalysis, RejectedJsxAnalysis};
use alloc::{boxed::Box, vec::Vec};
use vize_l2::file::{Namespace, ReferenceTarget};
use vize_l2::lang::js::{JsxFile, JsxNode};
use vize_l2::resolution::{SyntaxKind as Syntax, Usage};

/// Project only the genuine completed owner's same-walk scalar records.
/// Unsupported constructs refuse the whole view while retaining its custody.
pub fn build_jsx_decisions(owner: JsxFile<'_>) -> BuildResult<'_> {
    let mut decisions = Vec::new();
    let mut issues = Vec::new();
    for node in owner.nodes() {
        match classify(node) {
            Ok(kind) => decisions.push(kind),
            Err(kind) => {
                let span = node.span().unwrap_or(vize_l0::Span::new(0, 0));
                issues.push(JsxIssue { kind, span });
            }
        }
    }
    if !issues.is_empty() {
        return Err(Box::new(RejectedJsxAnalysis { owner, issues }));
    }
    Ok(NativeJsxAnalysis { owner, decisions })
}

fn classify<'a>(node: JsxNode<'_, 'a>) -> Result<Decision<'a>, Issue> {
    Ok(match node.kind().ok_or(Issue::InvalidRecord)? {
        Syntax::Expression => {
            let mut children = node.children();
            if node.parent().is_some()
                || children.next().and_then(JsxNode::kind) != Some(Syntax::Element)
                || children.next().is_some()
            {
                return Err(Issue::Expression);
            }
            Decision::Root
        }
        Syntax::IdentifierExpression(name) => {
            scalar_container_child(node)?;
            let Some([reference]) = node.references() else {
                return Err(Issue::Expression);
            };
            if !read_is_resolved(node, name) {
                return Err(Issue::Expression);
            }
            let ReferenceTarget::Resolved(binding) = reference.target else {
                return Err(Issue::Expression);
            };
            Decision::Read { name, binding }
        }
        Syntax::NumberExpression(value) => literal(node, Decision::Number(value))?,
        Syntax::StringExpression(value) => literal(node, Decision::String(value))?,
        Syntax::BooleanExpression(value) => literal(node, Decision::Boolean(value))?,
        Syntax::NullExpression => literal(node, Decision::Null)?,
        Syntax::Element => Decision::Element,
        Syntax::Opening { self_closing } => Decision::Opening { self_closing },
        Syntax::Closing => Decision::Closing,
        Syntax::Intrinsic(name) => Decision::Intrinsic(name),
        Syntax::Component(name) => {
            if !read_is_resolved(node, name) {
                return Err(Issue::UnresolvedComponent);
            }
            Decision::Component(name)
        }
        Syntax::Member => Decision::Member,
        Syntax::Property(name) => Decision::Property(name),
        Syntax::Attribute(name) => static_attribute(node, name)?,
        Syntax::AttributeName(name) => Decision::AttributeName(name),
        Syntax::AttributeString(value) => Decision::AttributeString(value),
        Syntax::Text(value) => Decision::Text(value),
        Syntax::Container => {
            let mut children = node.children();
            let child = children.next().and_then(JsxNode::kind);
            if children.next().is_some() {
                return Err(Issue::Expression);
            }
            match child {
                Some(Syntax::Empty) => Decision::EmptyContainer,
                Some(
                    Syntax::IdentifierExpression(_)
                    | Syntax::NumberExpression(_)
                    | Syntax::StringExpression(_)
                    | Syntax::BooleanExpression(_)
                    | Syntax::NullExpression,
                ) => Decision::ExpressionContainer,
                _ => return Err(Issue::Expression),
            }
        }
        Syntax::Empty => Decision::Empty,
        Syntax::Fragment | Syntax::FragmentOpening | Syntax::FragmentClosing => {
            return Err(Issue::Fragment);
        }
        Syntax::SpreadAttribute => return Err(Issue::SpreadAttribute),
        Syntax::SpreadChild => return Err(Issue::SpreadChild),
    })
}

fn scalar_container_child(node: JsxNode<'_, '_>) -> Result<(), Issue> {
    if node.parent().and_then(JsxNode::kind) != Some(Syntax::Container)
        || node.children().next().is_some()
    {
        return Err(Issue::Expression);
    }
    Ok(())
}

fn literal<'a>(node: JsxNode<'_, 'a>, value: Decision<'a>) -> Result<Decision<'a>, Issue> {
    scalar_container_child(node)?;
    if node.references().is_none_or(|rows| !rows.is_empty()) {
        return Err(Issue::Expression);
    }
    Ok(value)
}

fn read_is_resolved(node: JsxNode<'_, '_>, name: &str) -> bool {
    let Some(rows) = node.references() else {
        return false;
    };
    if rows.is_empty() {
        // A closing name has no evaluation and therefore no File reference.
        let mut parent = node.parent();
        while let Some(node) = parent {
            match node.kind() {
                Some(Syntax::Closing) => return true,
                Some(Syntax::Member) => parent = node.parent(),
                _ => return false,
            }
        }
        return false;
    }
    let [reference] = rows else {
        return false;
    };
    reference.name.as_str() == name
        && reference.usage == Usage::Read
        && node
            .owner()
            .file()
            .lookup(reference.scope, name, Namespace::Value)
            .is_some_and(|binding| reference.target == ReferenceTarget::Resolved(binding.id()))
}

fn static_attribute<'a>(node: JsxNode<'_, 'a>, name: &'a str) -> Result<Decision<'a>, Issue> {
    if name.starts_with("v-") || name.starts_with("vModel") || name.starts_with("vSlots") {
        return Err(Issue::DirectiveOrSlot);
    }
    if !matches!(name, "id" | "class" | "title" | "disabled")
        && !name.starts_with("data-")
        && !name.starts_with("aria-")
    {
        return Err(Issue::Attribute);
    }
    let mut children = node.children();
    if children.next().and_then(JsxNode::kind) != Some(Syntax::AttributeName(name)) {
        return Err(Issue::InvalidRecord);
    }
    let value = match children.next().and_then(JsxNode::kind) {
        None => None,
        Some(Syntax::AttributeString(value)) => Some(value),
        _ => return Err(Issue::Attribute),
    };
    if children.next().is_some() {
        return Err(Issue::Attribute);
    }
    Ok(Decision::StaticAttribute { name, value })
}
