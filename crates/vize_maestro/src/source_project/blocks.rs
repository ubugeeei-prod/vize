//! Stage-selective reads against the editor's shared open-document database.
use super::{SnapshotRefusal, SourceSnapshot};
use crate::server::ServerState;
use vize_incremental::{
    BlockKind, DescriptorParseError, artifact::SurfaceFinding, block_query::BlockPositionError,
};
use vize_l0::{Span, diag::Diagnostic, key::ArtifactKey};

/// The most detailed stage requested by the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockStage {
    Source,
    Surface,
    Page,
}

/// Only the requested findings are copied. Each finding has the original
/// stage's block-relative coordinates, based at `OpenBlock::content.start`.
#[derive(Debug, PartialEq, Eq)]
pub enum BlockFindings {
    SourceOnly,
    Unavailable,
    Surface {
        key: ArtifactKey,
        findings: Vec<SurfaceFinding>,
    },
    Page {
        key: ArtifactKey,
        diagnostics: Vec<Diagnostic>,
    },
}

/// The selected content extent and identity in this revision of the editor file.
#[derive(Debug, PartialEq, Eq)]
pub struct OpenBlock {
    pub kind: BlockKind,
    pub ordinal: u32,
    pub content: Span,
    pub source_key: ArtifactKey,
    pub findings: BlockFindings,
}

/// A missing or superseded document, inapplicable language or invalid input.
#[derive(Debug, PartialEq, Eq)]
pub enum OpenBlockError {
    Host(SnapshotRefusal),
    Language,
    Descriptor(DescriptorParseError),
    Position(BlockPositionError),
    ContentExtent,
}

impl SourceSnapshot {
    /// Locate a block at an authored UTF-8 byte offset under the current host
    /// revision. Gaps and EOF return `None`. A stage unavailable for this kind
    /// retains its source location. `SourceQueryResult::publish` can recheck
    /// the document after an asynchronous request uses the result.
    pub fn open_block_at(
        &self,
        state: &ServerState,
        byte: u32,
        stage: BlockStage,
    ) -> Result<Option<OpenBlock>, OpenBlockError> {
        self.with_current(&state.documents, |_| {
            if self.language_id() != "vue"
                || crate::utils::is_standalone_html_path(self.uri().path())
            {
                return Err(OpenBlockError::Language);
            }
            state
                .resident
                .open_blocks(self.uri(), self.source(), |blocks| {
                    let blocks = blocks
                        .map_err(|e| OpenBlockError::Descriptor(e.clone()))?
                        .blocks;
                    let Some(block) = blocks.at(byte).map_err(OpenBlockError::Position)? else {
                        return Ok(None);
                    };
                    let source = block.source();
                    let len = u32::try_from(source.text.len())
                        .map_err(|_| OpenBlockError::ContentExtent)?;
                    let content = block
                        .authored_span(Span::new(0, len))
                        .ok_or(OpenBlockError::ContentExtent)?;
                    let findings = match stage {
                        BlockStage::Source => BlockFindings::SourceOnly,
                        BlockStage::Surface => {
                            block
                                .surface()
                                .map_or(BlockFindings::Unavailable, |surface| {
                                    BlockFindings::Surface {
                                        key: surface.key,
                                        findings: surface.findings.clone(),
                                    }
                                })
                        }
                        BlockStage::Page => {
                            block.page().map_or(BlockFindings::Unavailable, |page| {
                                BlockFindings::Page {
                                    key: page.key,
                                    diagnostics: page.diagnostics.clone(),
                                }
                            })
                        }
                    };
                    Ok(Some(OpenBlock {
                        kind: block.kind().clone(),
                        ordinal: block.ordinal(),
                        content,
                        source_key: source.key,
                        findings,
                    }))
                })
        })
        .map_err(OpenBlockError::Host)?
    }
}

#[cfg(test)]
mod tests;
