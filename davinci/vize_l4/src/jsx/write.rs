use vize_l0::Span;
use vize_l2::lang::js::JsxNode;
use vize_l3::jsx::{JsxDecisionKind as Kind, NativeJsxAnalysis};

use super::helpers::SelectedHelpers;
use super::{JsxEmitError, JsxEmitErrorKind, invalid, kind};
use crate::write::line_endings::line_break_len;
use crate::write::{LinkSink, Writer};

/// Original spans get a link at each line, including comments and Unicode.
pub(super) fn original<L: LinkSink>(
    writer: &mut Writer<L>,
    source: &str,
    span: Span,
) -> Result<(), JsxEmitError> {
    let failure = || JsxEmitError {
        kind: JsxEmitErrorKind::SourceWindow,
        span,
    };
    let text = source
        .get(span.start as usize..span.end as usize)
        .ok_or_else(failure)?;
    if !L::RECORDING {
        writer.push(text);
        return Ok(());
    }
    let mut start = 0;
    let mut cursor = 0;
    while cursor < text.len() {
        let width = line_break_len(text, cursor);
        cursor += width.max(1);
        if width > 0 {
            writer.push_linked(
                text.get(start..cursor).ok_or_else(failure)?,
                Span::new(span.start + start as u32, span.start + cursor as u32),
            );
            start = cursor;
        }
    }
    if start < text.len() {
        writer.push_linked(
            text.get(start..).ok_or_else(failure)?,
            Span::new(span.start + start as u32, span.end),
        );
    }
    Ok(())
}

pub(super) fn quoted<L: LinkSink>(writer: &mut Writer<L>, value: &str, span: Span) {
    // JSON string syntax is valid JavaScript, including quotes/control bytes.
    writer.push_linked(&serde_json::to_string(value).unwrap_or_default(), span);
}

pub(super) fn element<L: LinkSink>(
    writer: &mut Writer<L>,
    analysis: &NativeJsxAnalysis<'_>,
    node: JsxNode<'_, '_>,
    helper: &SelectedHelpers,
) -> Result<(), JsxEmitError> {
    if kind(analysis, node)? != Kind::Element {
        return Err(invalid(node));
    }
    let mut children = node.children();
    let opening = children.next().ok_or_else(|| invalid(node))?;
    let mut header = opening.children().peekable();
    let name = header.next().ok_or_else(|| invalid(opening))?;
    writer.use_helper(helper.node.id);
    writer.anchor(node.span().ok_or_else(|| invalid(node))?.start);
    writer.push(helper.node.alias.as_str());
    writer.push("(");
    match kind(analysis, name)? {
        Kind::Intrinsic(value) => quoted(writer, value, name.span().ok_or_else(|| invalid(name))?),
        Kind::Component(value) => writer.push_named(
            name.source().ok_or_else(|| invalid(name))?,
            name.span().ok_or_else(|| invalid(name))?,
            value,
        ),
        _ => return Err(invalid(name)),
    }
    writer.push(", ");
    if header.peek().is_none() {
        writer.push("null");
    } else {
        writer.push("{ ");
        let mut separator = "";
        for attribute in header {
            let Kind::StaticAttribute { name, value } = kind(analysis, attribute)? else {
                return Err(invalid(attribute));
            };
            writer.push(separator);
            separator = ", ";
            quoted(
                writer,
                name,
                attribute.span().ok_or_else(|| invalid(attribute))?,
            );
            writer.push(": ");
            if let Some(value) = value {
                let value_node = attribute
                    .children()
                    .nth(1)
                    .ok_or_else(|| invalid(attribute))?;
                quoted(
                    writer,
                    super::whitespace::normalized(value)
                        .as_ref()
                        .map_or(value, |value| value.as_str()),
                    value_node.span().ok_or_else(|| invalid(value_node))?,
                );
            } else {
                writer.push("true");
            }
        }
        writer.push(" }");
    }
    writer.push(", ");
    let has_children = node
        .children()
        .skip(1)
        .any(|child| match kind(analysis, child) {
            Ok(Kind::Closing | Kind::EmptyContainer) => false,
            Ok(Kind::Text(value)) => super::whitespace::has_text(value),
            _ => true,
        });
    writer.push(if has_children { "[" } else { "null" });
    let mut separator = "";
    for child in children {
        match kind(analysis, child)? {
            Kind::Closing => {}
            Kind::EmptyContainer => container(writer, analysis, child)?,
            Kind::Text(value) if super::whitespace::has_text(value) => {
                writer.push(separator);
                separator = ", ";
                let text_helper = helper.text.as_ref().ok_or_else(|| invalid(child))?;
                writer.use_helper(text_helper.id);
                writer.push(text_helper.alias.as_str());
                writer.push("(");
                quoted(
                    writer,
                    super::whitespace::normalized(value)
                        .as_ref()
                        .map_or(value, |value| value.as_str()),
                    child.span().ok_or_else(|| invalid(child))?,
                );
                writer.push(")");
            }
            Kind::Text(_) => {}
            Kind::Element => {
                writer.push(separator);
                separator = ", ";
                element(writer, analysis, child, helper)?;
            }
            Kind::ExpressionContainer => {
                writer.push(separator);
                separator = ", ";
                container(writer, analysis, child)?;
            }
            _ => return Err(invalid(child)),
        }
    }
    if has_children {
        writer.push("]");
    }
    writer.push(")");
    Ok(())
}

fn container<L: LinkSink>(
    writer: &mut Writer<L>,
    analysis: &NativeJsxAnalysis<'_>,
    node: JsxNode<'_, '_>,
) -> Result<(), JsxEmitError> {
    let span = node.span().ok_or_else(|| invalid(node))?;
    let child = node.children().next().ok_or_else(|| invalid(node))?;
    let source = analysis.owner().file().artifact().source();
    let inner = Span::new(span.start + 1, span.end - 1);
    let child_span = child.span().ok_or_else(|| invalid(child))?;
    if kind(analysis, child)? == Kind::Empty {
        return original(writer, source, inner);
    }
    original(writer, source, Span::new(inner.start, child_span.start))?;
    let text = child.source().ok_or_else(|| invalid(child))?;
    match kind(analysis, child)? {
        Kind::Read { name, .. } => writer.push_named(text, child_span, name),
        Kind::Number(_) | Kind::String(_) | Kind::Boolean(_) | Kind::Null => {
            writer.push_linked(text, child_span);
        }
        _ => return Err(invalid(child)),
    }
    original(writer, source, Span::new(child_span.end, inner.end))
}
