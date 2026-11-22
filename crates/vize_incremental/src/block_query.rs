//! Borrow stage results from one revision of an open document. Requests for
//! positions, source blocks, markup and canonical pages share
//! the resident block partition. Each accessor asks for only its own stage.
use crate::{
    BlockKind, BlockSource, DescriptorParseError, PageArtifact, ParsedSfc, ResidentDatabase,
    SourceFile, SurfaceArtifact,
    db::{Block, l1_block, l2_page, sfc_blocks},
    descriptor::sfc_descriptor,
};
use vize_l0::Span;

#[cfg(feature = "ir-graph")]
pub mod graph;

#[cfg(feature = "open-block-queries")]
pub(crate) mod open;
#[cfg(feature = "open-block-queries")]
pub use open::ParsedBlocks;

/// A file-absolute offset cannot select a UTF-8 position in this revision.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockPositionError {
    /// The offset exceeds the document length in bytes.
    OutOfBounds,
    /// The offset lies inside the encoding of a single Unicode scalar.
    NotCharBoundary,
}

/// The accepted block partition and the exact document revision it indexes.
pub struct DocumentBlocks<'db> {
    db: &'db ResidentDatabase,
    blocks: &'db [Block<'db>],
    source: &'db str,
}

impl ResidentDatabase {
    /// Read an input opened in this database. A rejected document exposes its
    /// original descriptor error. A read holds the current immutable revision.
    ///
    /// # Errors
    /// Returns the descriptor's rejection for this file and revision.
    #[inline]
    pub fn document_blocks(
        &self,
        file: SourceFile,
    ) -> Result<DocumentBlocks<'_>, &DescriptorParseError> {
        let blocks = sfc_blocks(self, file);
        // The existing partition represents both rejected and empty files as
        // zero blocks. Only that case needs the original descriptor outcome.
        if blocks.is_empty()
            && let ParsedSfc::Failed(error) = sfc_descriptor(self, file)
        {
            return Err(error);
        }
        Ok(DocumentBlocks {
            db: self,
            blocks,
            source: file.text(self).as_str(),
        })
    }
}

impl<'db> DocumentBlocks<'db> {
    /// All accepted blocks in authored order, including custom and empty blocks.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = BlockView<'db>> + DoubleEndedIterator + '_ {
        self.blocks
            .iter()
            .map(|&block| BlockView { db: self.db, block })
    }

    /// The block content at a file-absolute UTF-8 boundary. Headers, gaps,
    /// empty blocks and EOF have no containing content.
    ///
    /// # Errors
    /// Returns [`BlockPositionError::OutOfBounds`] past EOF and
    /// [`BlockPositionError::NotCharBoundary`] inside a multi-byte scalar.
    #[inline]
    pub fn at(&self, byte: u32) -> Result<Option<BlockView<'db>>, BlockPositionError> {
        if byte as usize > self.source.len() {
            return Err(BlockPositionError::OutOfBounds);
        }
        if !self.source.is_char_boundary(byte as usize) {
            return Err(BlockPositionError::NotCharBoundary);
        }
        // A typical SFC has just a script, template and style. The first
        // matching content is cheaper to read directly on short partitions.
        if self.blocks.len() <= 6 {
            for &block in self.blocks {
                let start = block.start(self.db);
                let Some(relative) = byte.checked_sub(start) else {
                    break;
                };
                if (relative as usize) < block.source(self.db).text.len() {
                    return Ok(Some(BlockView { db: self.db, block }));
                }
            }
            return Ok(None);
        }
        let end = self
            .blocks
            .partition_point(|block| block.start(self.db) <= byte);
        let Some(&block) = end.checked_sub(1).and_then(|index| self.blocks.get(index)) else {
            return Ok(None);
        };
        let view = BlockView { db: self.db, block };
        let relative = byte - block.start(self.db);
        Ok(((relative as usize) < view.source().text.len()).then_some(view))
    }
}

/// A genuine block of the revision, shared by all requests for its artifacts.
/// Stages keep their block-relative coordinates when an earlier block moves.
#[derive(Clone, Copy)]
pub struct BlockView<'db> {
    db: &'db ResidentDatabase,
    block: Block<'db>,
}

impl<'db> BlockView<'db> {
    /// The descriptor role, including the authored tag of a custom block.
    #[must_use]
    #[inline]
    pub fn kind(self) -> &'db BlockKind {
        self.block.kind(self.db)
    }
    /// The zero-based ordinal among blocks of this kind in this document.
    #[must_use]
    #[inline]
    pub fn ordinal(self) -> u32 {
        self.block.ordinal(self.db)
    }
    /// The file-absolute UTF-8 byte offset of the first content byte.
    #[must_use]
    #[inline]
    pub fn start(self) -> u32 {
        self.block.start(self.db)
    }
    /// The original content, sorted header attributes and L0 source key.
    #[must_use]
    #[inline]
    pub fn source(self) -> &'db BlockSource {
        self.block.source(self.db)
    }
    /// The L1 surface of an inline HTML template. Other blocks return `None`.
    #[must_use]
    #[inline]
    pub fn surface(self) -> Option<&'db SurfaceArtifact> {
        l1_block(self.db, self.block).as_ref()
    }
    /// The L2 page for an inline HTML template or a style block. Other blocks return `None`.
    #[must_use]
    #[inline]
    pub fn page(self) -> Option<&'db PageArtifact> {
        l2_page(self.db, self.block).as_ref()
    }

    /// Map a complete UTF-8 span in this block to the current document.
    #[must_use]
    #[inline]
    pub fn authored_span(self, relative: Span) -> Option<Span> {
        self.source()
            .text
            .get(relative.start as usize..relative.end as usize)?;
        let start = self.start();
        Some(Span::new(
            start.checked_add(relative.start)?,
            start.checked_add(relative.end)?,
        ))
    }

    /// The module binding selected by the original style header.
    #[must_use]
    #[inline]
    pub fn style_module(self) -> Option<&'db str> {
        if self.kind() != &BlockKind::Style {
            return None;
        }
        self.source()
            .attr("module")
            .map(|name| if name.is_empty() { "$style" } else { name })
    }
}

#[cfg(all(test, not(feature = "seeded-stale-cache")))]
mod tests;

#[cfg(all(test, not(feature = "seeded-stale-cache")))]
mod corpus;
