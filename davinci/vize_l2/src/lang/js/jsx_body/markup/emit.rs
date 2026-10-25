use super::prepare::children;
use crate::{
    artifact::{ArtifactError, RegionBuilder},
    lang::js::jsx_body::records::Record,
    op::{Attribute, Namespace},
    resolution::SyntaxKind as Syntax,
};
use vize_l0::Vec;

pub(super) fn element<'a>(
    records: &[Record<'a>],
    index: usize,
    region: &mut RegionBuilder<'_, 'a>,
) -> Result<(), ArtifactError> {
    let row = records.get(index).ok_or_else(invalid)?;
    let mut body = children(records, index);
    let opening = body.next().ok_or_else(invalid)?;
    let mut header = children(records, opening);
    let name = records
        .get(header.next().ok_or_else(invalid)?)
        .ok_or_else(invalid)?;
    let Syntax::Intrinsic(tag) = name.kind else {
        return Err(invalid());
    };
    let mut attributes = Vec::new_in(&region.allocator());
    for attribute in header {
        let row = records.get(attribute).ok_or_else(invalid)?;
        let Syntax::Attribute(name) = row.kind else {
            return Err(invalid());
        };
        let value = children(records, attribute)
            .nth(1)
            .and_then(|index| records.get(index))
            .and_then(|row| match row.kind {
                Syntax::AttributeString(value) => Some(value),
                _ => None,
            });
        attributes.push(Attribute {
            name,
            value,
            span: row.span,
        });
    }
    let mut error = None;
    region.element(tag, Namespace::Html, attributes, row.span, |region, _| {
        for child in body {
            if error.is_some() {
                break;
            }
            let Some(row) = records.get(child) else {
                error = Some(invalid());
                break;
            };
            error = match row.kind {
                Syntax::Element => element(records, child, region).err(),
                Syntax::Text(text) => region.text(text, row.span).err(),
                // Original JSX comments are retained in ProgramObservation;
                // they do not create rendered markup nodes.
                Syntax::Container | Syntax::Closing => None,
                _ => Some(invalid()),
            };
        }
    })?;
    error.map_or(Ok(()), Err)
}

fn invalid() -> ArtifactError {
    ArtifactError::InvalidSpan {
        node: None,
        span: vize_l0::Span::new(0, 0),
    }
}
