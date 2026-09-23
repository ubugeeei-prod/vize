//! Per-request IDE context shared by every language feature service.

use std::sync::OnceLock;

use tower_lsp::lsp_types::Url;
use vize_resident::SharedDescriptor;

use super::standalone_html_block_at_offset;
use crate::server::ServerState;
use crate::utils::is_standalone_html_path;
use crate::virtual_code::{
    ArtCursorPosition, BlockType, VirtualDocuments, find_art_block_at_completion_offset,
    find_art_block_at_offset, find_block_at_completion_offset, find_block_at_offset,
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
    /// This document's SFC descriptor, read once from the resident tier
    /// (P5-6a) — see [`IdeContext::descriptor`].
    descriptor: OnceLock<Option<SharedDescriptor>>,
}

impl<'a> IdeContext<'a> {
    /// Create a context for an open document.
    ///
    /// The text comes from the document store. `None` when `uri` is not open.
    pub fn new(state: &'a ServerState, uri: &'a Url, offset: usize) -> Option<Self> {
        Self::read(state, uri, offset, false)
    }

    /// Completion context for an open document. Script-body ends stay insertion points.
    pub(crate) fn at_completion(
        state: &'a ServerState,
        uri: &'a Url,
        offset: usize,
    ) -> Option<Self> {
        Self::read(state, uri, offset, true)
    }

    /// A buffer that is not an open document (another file's text, or a test
    /// fixture). Request handlers use [`Self::new`].
    #[cfg(feature = "native")]
    pub(crate) fn for_unopened(
        state: &'a ServerState,
        uri: &'a Url,
        offset: usize,
        content: String,
    ) -> Self {
        Self::assemble(state, uri, offset, content, false)
    }

    /// Publish `content` as the open document and read the context back.
    /// Tests use this so the store, not a side string, is the source of text.
    pub fn testing(state: &'a ServerState, uri: &'a Url, offset: usize, content: String) -> Self {
        state.documents.open(uri.clone(), content, 0, "vue".into());
        let content = state.documents.text(uri).unwrap_or_default();
        Self::assemble(state, uri, offset, content, false)
    }

    /// [`Self::testing`] for a completion cursor.
    pub fn testing_completion(
        state: &'a ServerState,
        uri: &'a Url,
        offset: usize,
        content: String,
    ) -> Self {
        state.documents.open(uri.clone(), content, 0, "vue".into());
        let content = state.documents.text(uri).unwrap_or_default();
        Self::assemble(state, uri, offset, content, true)
    }

    fn read(state: &'a ServerState, uri: &'a Url, offset: usize, completion: bool) -> Option<Self> {
        let content = state.documents.text(uri)?;
        Some(Self::assemble(state, uri, offset, content, completion))
    }

    fn assemble(
        state: &'a ServerState,
        uri: &'a Url,
        offset: usize,
        content: String,
        completion: bool,
    ) -> Self {
        let descriptor = OnceLock::new();
        // Determine block type
        let block_type = if uri.path().ends_with(".art.vue") {
            // For art files, use art-specific block detection
            if completion {
                find_art_block_at_completion_offset(&content, offset)
            } else {
                find_art_block_at_offset(&content, offset)
            }
        } else if is_standalone_html_path(uri.path()) {
            Some(standalone_html_block_at_offset(&content, offset))
        } else {
            // The resident descriptor determines the block type.
            descriptor
                .get_or_init(|| state.sfc_descriptor(uri, &content))
                .as_deref()
                .and_then(|descriptor| {
                    if completion {
                        find_block_at_completion_offset(descriptor, offset)
                    } else {
                        find_block_at_offset(descriptor, offset)
                    }
                    .or_else(|| {
                        (state.patterned_template_enabled()
                            && root_match_subject_at(descriptor, offset, completion))
                        .then_some(BlockType::Template)
                    })
                })
        };

        let virtual_docs = state.get_virtual_docs(uri);

        Self {
            state,
            uri,
            content,
            offset,
            block_type,
            virtual_docs,
            descriptor,
        }
    }

    /// This document's SFC descriptor, served by the resident tier: one parse
    /// per buffer revision shared by every request, instead of `parse_sfc`
    /// per request (P5-6a). `None` when the parser rejects the content.
    pub fn descriptor(&self) -> Option<&SharedDescriptor> {
        self.descriptor
            .get_or_init(|| self.state.sfc_descriptor(self.uri, &self.content))
            .as_ref()
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

fn root_match_subject_at(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    offset: usize,
    completion: bool,
) -> bool {
    use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};
    let Some(template) = descriptor.template.as_ref() else {
        return false;
    };
    if offset < template.loc.tag_start
        || offset >= template.loc.start
        || !template.has_root_match()
        || template.src.is_some()
        || template.lang.as_deref().is_some_and(|lang| lang != "html")
    {
        return false;
    }
    let Some(header) = descriptor
        .source
        .get(template.loc.tag_start..template.loc.start)
    else {
        return false;
    };
    let allocator = vize_s0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, header);
    let Some(TemplateChildNode::Element(element)) = root.children.first() else {
        return false;
    };
    element.props.iter().any(|prop| {
        let PropNode::Directive(dir) = prop else {
            return false;
        };
        let Some(ExpressionNode::Simple(expression)) = &dir.exp else {
            return false;
        };
        dir.name == "match"
            && dir.arg.is_none()
            && dir.modifiers.is_empty()
            && offset >= template.loc.tag_start + expression.loc.span.start as usize
            && (offset < template.loc.tag_start + expression.loc.span.end as usize
                || (completion
                    && offset == template.loc.tag_start + expression.loc.span.end as usize))
    })
}

#[cfg(test)]
mod tests;
