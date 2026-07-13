use super::*;

pub(super) fn snapshot_children(node: &JsxSyntaxNode) -> &[JsxSyntaxNode] {
    match node {
        JsxSyntaxNode::Element(element) => &element.children,
        JsxSyntaxNode::Fragment { children, .. } => children,
        _ => &[],
    }
}

pub(super) fn walk_snapshot_nodes<'a>(
    nodes: &'a [JsxSyntaxNode],
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    for node in nodes {
        match node {
            JsxSyntaxNode::Element(_) | JsxSyntaxNode::Fragment { .. } => {
                let element = MarkupElement::from_jsx_snapshot(node);
                enter(element);
                walk_snapshot_nodes(snapshot_children(node), enter, exit);
                exit(element);
            }
            JsxSyntaxNode::If { branches, .. } => {
                for branch in branches {
                    walk_snapshot_nodes(&branch.body, enter, exit);
                }
            }
            JsxSyntaxNode::For { body, .. } => walk_snapshot_nodes(body, enter, exit),
            _ => {}
        }
    }
}

pub(super) fn snapshot_attribute_name(attribute: &JsxSyntaxAttribute) -> Option<&str> {
    match attribute {
        JsxSyntaxAttribute::Attribute { name, .. } => Some(name),
        _ => None,
    }
}

pub(super) fn snapshot_attribute_span(attribute: &JsxSyntaxAttribute) -> JsxSyntaxSpan {
    match attribute {
        JsxSyntaxAttribute::Attribute { span, .. } | JsxSyntaxAttribute::Spread { span, .. } => {
            *span
        }
    }
}

pub(super) fn snapshot_attribute_dynamic(attribute: &JsxSyntaxAttribute) -> bool {
    matches!(
        attribute,
        JsxSyntaxAttribute::Attribute {
            value: JsxSyntaxAttributeValue::Expression(_),
            ..
        } | JsxSyntaxAttribute::Spread { .. }
    )
}

pub(super) fn snapshot_attribute_kind(attribute: &JsxSyntaxAttribute) -> Option<MarkupBindingKind> {
    let name = snapshot_attribute_name(attribute)?;
    Some(if is_jsx_event_handler_name(name) {
        MarkupBindingKind::On
    } else if snapshot_attribute_dynamic(attribute) {
        MarkupBindingKind::Bind
    } else {
        MarkupBindingKind::Attribute
    })
}

pub(super) fn snapshot_attribute_arg(attribute: &JsxSyntaxAttribute) -> Option<&str> {
    let name = snapshot_attribute_name(attribute)?;
    if snapshot_attribute_kind(attribute) == Some(MarkupBindingKind::On) {
        name.strip_prefix("on")
    } else {
        Some(name)
    }
}

#[inline]
pub(super) const fn snapshot_range(span: JsxSyntaxSpan) -> ByteRange {
    ByteRange::new(span.start, span.end)
}
