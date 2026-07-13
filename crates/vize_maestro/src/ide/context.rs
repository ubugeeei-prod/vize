//! Per-request IDE context backed by persistent Atlas products.

use dashmap::mapref::one::Ref;
use tower_lsp::lsp_types::Url;
use vize_atlas::Shared;

use super::standalone_html_block_at_offset;
use crate::server::ServerState;
use crate::utils::is_standalone_html_path;
use crate::virtual_code::{
    ArtCursorPosition, BlockType, VirtualDocuments, find_art_block_at_offset_with_descriptor,
    find_block_at_offset,
};

pub struct IdeContext<'a> {
    pub state: &'a ServerState,
    pub uri: &'a Url,
    pub content: String,
    pub offset: usize,
    pub block_type: Option<BlockType>,
    pub sfc_artifact: Option<Shared<vize_atelier_sfc::SfcDescriptorArtifact>>,
    relief_artifact: Option<Shared<Option<vize_relief::ReliefArtifact>>>,
    pub virtual_docs: Option<Ref<'a, Url, VirtualDocuments>>,
}

impl<'a> IdeContext<'a> {
    pub fn new(state: &'a ServerState, uri: &'a Url, offset: usize) -> Option<Self> {
        let content = state.documents.get(uri)?.text();
        Some(Self::with_content(state, uri, offset, content))
    }

    pub fn with_content(
        state: &'a ServerState,
        uri: &'a Url,
        offset: usize,
        content: String,
    ) -> Self {
        let is_sfc = uri.path().ends_with(".vue");
        let is_standalone_html = is_standalone_html_path(uri.path());
        if is_standalone_html {
            state.ensure_artifact_source(uri, &content);
        }
        let sfc_artifact = is_sfc
            .then(|| {
                state.ensure_artifact_source(uri, &content);
                state.sfc_descriptor(uri)
            })
            .flatten();
        let relief_artifact = (is_sfc && !uri.path().ends_with(".art.vue"))
            .then(|| state.sfc_relief(uri))
            .flatten();
        let block_type = if uri.path().ends_with(".art.vue") {
            find_art_block_at_offset_with_descriptor(
                &content,
                offset,
                sfc_artifact
                    .as_ref()
                    .and_then(|artifact| artifact.descriptor()),
            )
        } else if is_standalone_html {
            Some(standalone_html_block_at_offset(&content, offset))
        } else {
            sfc_artifact
                .as_ref()
                .and_then(|artifact| artifact.descriptor())
                .and_then(|descriptor| find_block_at_offset(descriptor, offset))
        };
        let virtual_docs = state.get_virtual_docs(uri);
        Self {
            state,
            uri,
            content,
            offset,
            block_type,
            sfc_artifact,
            relief_artifact,
            virtual_docs,
        }
    }

    pub fn sfc_descriptor(&self) -> Option<&vize_atelier_sfc::SfcDescriptor<'static>> {
        self.sfc_artifact.as_ref()?.descriptor()
    }

    pub fn sfc_croquis(&self) -> Option<Shared<vize_croquis::CroquisDocument>> {
        self.state.sfc_croquis(self.uri)
    }

    pub fn relief_snapshot(&self) -> Option<&vize_relief::ReliefSnapshot> {
        self.relief_artifact
            .as_ref()?
            .as_ref()
            .as_ref()
            .map(|syntax| syntax.snapshot())
    }

    pub fn dialect(&self) -> vize_carton::dialect::VueDialect {
        self.state.document_dialect(self.uri, &self.content)
    }

    pub fn is_in_template(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Template))
    }

    pub fn is_in_script(&self) -> bool {
        matches!(
            self.block_type,
            Some(BlockType::Script | BlockType::ScriptSetup)
        )
    }

    pub fn is_in_style(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Style(_)))
    }

    pub fn is_in_art(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Art(_)))
    }

    pub fn is_in_art_variant_template(&self) -> bool {
        matches!(
            self.block_type,
            Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
        )
    }
}
