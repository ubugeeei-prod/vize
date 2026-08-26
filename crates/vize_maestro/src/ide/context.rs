//! Per-request IDE context shared by every language feature service.

use tower_lsp::lsp_types::Url;

use super::standalone_html_block_at_offset;
use crate::server::ServerState;
use crate::utils::is_standalone_html_path;
use crate::virtual_code::{
    ArtCursorPosition, BlockType, VirtualDocuments, find_art_block_at_offset, find_block_at_offset,
};

/// Context for IDE operations.
pub struct IdeContext<'a> {
    /// Server state
    pub state: &'a ServerState,
    /// Document URI
    pub uri: &'a Url,
    /// Document content
    pub content: String,
    /// Cursor offset in the document
    pub offset: usize,
    /// Which block the cursor is in
    pub block_type: Option<BlockType>,
    /// Virtual documents for this file. An owned snapshot, never a `DashMap` shard guard: `&IdeContext` crosses the `.await` points of hover, completion, definition, references and rename, where a live guard hangs the server — see [`ServerState::get_virtual_docs`] (#3377).
    pub virtual_docs: Option<std::sync::Arc<VirtualDocuments>>,
}

impl<'a> IdeContext<'a> {
    /// Create a new IDE context.
    ///
    /// This re-fetches the document from the store and materializes its content.
    /// Callers that have already materialized the document content should prefer
    /// [`IdeContext::with_content`] to avoid a redundant document lookup and a
    /// second full Rope→String allocation.
    pub fn new(state: &'a ServerState, uri: &'a Url, offset: usize) -> Option<Self> {
        let content = state.documents.text(uri)?;
        Some(Self::with_content(state, uri, offset, content))
    }

    /// Create a new IDE context from already-materialized document content.
    ///
    /// Reuses the provided `content` instead of re-reading the document from the
    /// store, avoiding a redundant `DashMap` lookup and a second full
    /// Rope→String allocation per request.
    pub fn with_content(
        state: &'a ServerState,
        uri: &'a Url,
        offset: usize,
        content: String,
    ) -> Self {
        // Determine block type
        let block_type = if uri.path().ends_with(".art.vue") {
            // For art files, use art-specific block detection
            find_art_block_at_offset(&content, offset)
        } else if is_standalone_html_path(uri.path()) {
            Some(standalone_html_block_at_offset(&content, offset))
        } else {
            // Parse SFC to determine block type
            let options = vize_atelier_sfc::SfcParseOptions {
                filename: uri.path().to_string().into(),
                ..Default::default()
            };
            if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&content, options) {
                find_block_at_offset(&descriptor, offset)
            } else {
                None
            }
        };

        let virtual_docs = state.get_virtual_docs(uri);

        Self {
            state,
            uri,
            content,
            offset,
            block_type,
            virtual_docs,
        }
    }

    /// Effective Vue dialect for this document.
    ///
    /// Delegates to [`ServerState::document_dialect`]: an explicit `dialect`
    /// config key wins, otherwise the structural petite-vue detection memoized
    /// on the open document is used (no per-request re-scan).
    #[inline]
    pub fn dialect(&self) -> vize_s0::dialect::VueDialect {
        self.state.document_dialect(self.uri, &self.content)
    }

    /// Check if cursor is in template block.
    #[inline]
    pub fn is_in_template(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Template))
    }

    /// Check if cursor is in script block.
    #[inline]
    pub fn is_in_script(&self) -> bool {
        matches!(
            self.block_type,
            Some(BlockType::Script) | Some(BlockType::ScriptSetup)
        )
    }

    /// Check if cursor is in style block.
    #[inline]
    pub fn is_in_style(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Style(_)))
    }

    /// Check if cursor is in an art custom block.
    #[inline]
    pub fn is_in_art(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Art(_)))
    }

    /// Check if cursor is in an art variant template.
    #[inline]
    pub fn is_in_art_variant_template(&self) -> bool {
        matches!(
            self.block_type,
            Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
        )
    }
}
