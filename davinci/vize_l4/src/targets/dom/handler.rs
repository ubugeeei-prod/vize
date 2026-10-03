//! Whole original bounded click bodies, never fabricated ExprRef payloads.

use super::{DomError, DomErrorKind};
use crate::write::{LinkSink, Writer};
use vize_l0::{Span, id::NodeId};
use vize_l2::{file::FileArtifact, op::OnOp};
use vize_l3::decision::dom::{DomFacts, PropertyRole};

pub(super) fn write<L: LinkSink>(
    writer: &mut Writer<L>,
    file: &FileArtifact<'_>,
    facts: Option<&DomFacts<'_, '_>>,
    node: NodeId,
    on: &OnOp<'_>,
) -> Result<(), DomError> {
    let fail = |kind| DomError {
        node: Some(node),
        span: on.span,
        kind,
    };
    let facts = facts.ok_or_else(|| fail(DomErrorKind::MissingAnalysis))?;
    let row = facts
        .file_handler(node)
        .ok_or_else(|| fail(DomErrorKind::MissingBinding))?;
    if !core::ptr::eq(row.handler().file(), file)
        || row.handler().id().node() != node
        || !core::ptr::eq(row.on(), on)
        || !facts
            .binding(node)
            .is_some_and(|binding| binding.role == PropertyRole::Event)
    {
        return Err(fail(DomErrorKind::FileOwnerMismatch));
    }
    let source = row.resolution().input().operand().syntax().source();
    if !core::ptr::eq(source.authored_root(), file.artifact().source()) {
        return Err(fail(DomErrorKind::FileOwnerMismatch));
    }
    // Semantic syntax/access classification was sealed at the actual L3 On
    // event. This appends its complete unchanged decoded window only.
    writer.push("$event => {");
    let mut references = row.resolution().references().iter().peekable();
    let mut segment = |decoded: Span, authored: Span| -> Result<(), DomError> {
        let mut at = decoded.start;
        while let Some(reference) = references.peek().copied() {
            if reference.span.start >= decoded.end {
                break;
            }
            if reference.span.start < at
                || reference.span.end > decoded.end
                || decoded.end - decoded.start != authored.end - authored.start
            {
                return Err(fail(DomErrorKind::UncertifiedExpressionSpelling));
            }
            let prefix = source
                .text()
                .get(at as usize..reference.span.start as usize)
                .ok_or_else(|| fail(DomErrorKind::UncertifiedExpressionSpelling))?;
            if !prefix.is_empty() {
                writer.push_linked(
                    prefix,
                    Span::new(
                        authored.start + (at - decoded.start),
                        authored.start + (reference.span.start - decoded.start),
                    ),
                );
            }
            let text = source
                .text()
                .get(reference.span.start as usize..reference.span.end as usize)
                .ok_or_else(|| fail(DomErrorKind::UncertifiedExpressionSpelling))?;
            writer.push_named(
                text,
                Span::new(
                    authored.start + (reference.span.start - decoded.start),
                    authored.start + (reference.span.end - decoded.start),
                ),
                reference.name,
            );
            at = reference.span.end;
            references.next();
        }
        let text = source
            .text()
            .get(at as usize..decoded.end as usize)
            .ok_or_else(|| fail(DomErrorKind::UncertifiedExpressionSpelling))?;
        if !text.is_empty() {
            let authored = if at == decoded.start {
                authored
            } else {
                Span::new(authored.start + (at - decoded.start), authored.end)
            };
            writer.push_linked(text, authored);
        }
        Ok(())
    };
    if let Some(map) = source.decode_map() {
        for original in map.segments() {
            segment(original.decoded(), original.authored())?;
        }
    } else {
        let end =
            u32::try_from(source.text().len()).map_err(|_| fail(DomErrorKind::OutputTooLarge))?;
        segment(Span::new(0, end), source.span())?;
    }
    if references.next().is_some() {
        return Err(fail(DomErrorKind::UncertifiedExpressionSpelling));
    }
    writer.push("}");
    Ok(())
}
