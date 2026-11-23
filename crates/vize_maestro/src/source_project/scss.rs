//! Variable definitions projected from the same original SCSS that produces CSS.
use super::{SnapshotRefusal, SourceSnapshot};
use crate::{
    document::DocumentStore,
    utils::{offset_to_position, position_to_offset},
};
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range};
use vize_atelier_sfc::scss::{ScssProducts, ScssRefusal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScssDefinitionError {
    Host(SnapshotRefusal),
    ForeignSource,
    Style(ScssRefusal),
}

impl SourceSnapshot {
    /// Project a source-owned stylesheet's variable use to a UTF-16 LSP
    /// location under the current document revision. Both source identity and
    /// the actual editor revision must match before returning a response.
    pub fn scss_definition(
        &self,
        documents: &DocumentStore,
        products: &ScssProducts<'_>,
        position: Position,
    ) -> Result<Option<GotoDefinitionResponse>, ScssDefinitionError> {
        if !core::ptr::eq(products.block().root_source(), self.source()) {
            return Err(ScssDefinitionError::ForeignSource);
        }
        self.with_current(documents, |document| {
            let program = products.program().map_err(ScssDefinitionError::Style)?;
            let text = document.position_text();
            let location = position_to_offset(text, position)
                .and_then(|offset| program.definition_at(u32::try_from(offset).ok()?))
                .and_then(|declaration| {
                    let start = offset_to_position(text, declaration.span.start as usize)?;
                    let end = offset_to_position(text, declaration.span.end as usize)?;
                    Some(GotoDefinitionResponse::Scalar(Location::new(
                        self.uri().clone(),
                        Range::new(start, end),
                    )))
                });
            Ok(location)
        })
        .map_err(ScssDefinitionError::Host)?
    }
}

#[cfg(test)]
mod tests;
