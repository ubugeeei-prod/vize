//! Virtual document generation and caching.

use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
use tower_lsp::lsp_types::Url;

use crate::utils::is_standalone_html_path;
use crate::virtual_code::VirtualDocuments;

use super::ServerState;

mod art;

use art::{
    add_inline_art_template_virtual_docs, art_script_setup_isolated,
    generate_art_script_setup_virtual_doc,
};

#[cfg(test)]
mod tests;

impl ServerState {
    /// Generate and cache virtual documents for a document.
    pub fn update_virtual_docs(&self, uri: &Url, content: &str) {
        self.open_imports.update(uri, content);
        if uri.path().ends_with(".art.vue") {
            self.update_art_virtual_docs(uri, content);
            return;
        }

        if is_standalone_html_path(uri.path()) {
            self.update_standalone_html_virtual_docs(uri, content);
            return;
        }

        if crate::utils::is_jsx_path(uri.path()) {
            self.update_jsx_virtual_docs(uri, content);
            return;
        }

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            self.remove_virtual_docs(uri);
            return;
        };

        let base_uri = uri.path();
        let mut virtual_docs = self.virtual_gen.write().generate(&descriptor, base_uri);
        add_inline_art_template_virtual_docs(&mut virtual_docs, &descriptor, base_uri);
        super::art_template_context::attach(
            &mut virtual_docs,
            descriptor.script_setup.as_ref(),
            false,
        );
        self.virtual_docs_cache
            .insert(uri.clone(), Arc::new(virtual_docs));
    }

    /// Generate and cache virtual documents for standalone HTML files.
    fn update_standalone_html_virtual_docs(&self, uri: &Url, content: &str) {
        use crate::virtual_code::{TemplateCodeGenerator, VirtualDocuments};

        let allocator = vize_s0::Allocator::new();
        let (ast, _errors) = vize_armature::parse(&allocator, content);
        let base_uri = uri.path();

        let mut template_gen = TemplateCodeGenerator::new();
        template_gen.set_block_offset(0);
        let mut template_doc = template_gen.generate(&ast, content);
        template_doc.uri = vize_s0::cstr!("{base_uri}.__template.ts").to_string();

        let mut docs = VirtualDocuments::new();
        docs.template = Some(template_doc);
        self.virtual_docs_cache.insert(uri.clone(), Arc::new(docs));
    }

    /// Generate and cache virtual documents for a `.jsx`/`.tsx` document.
    ///
    /// JSX/TSX components are not SFCs, so the only embedded-language virtual
    /// documents they produce are the CSS blocks of any `<style scoped>` (#1495,
    /// #1498). The type-aware features build their own per-request virtual TS
    /// (see [`crate::ide::JsxService`]); this cache only needs to expose the
    /// scoped CSS so the editor's CSS service gets diagnostics + source mapping,
    /// mirroring the SFC style virtual-document path.
    fn update_jsx_virtual_docs(&self, uri: &Url, content: &str) {
        let styles = crate::ide::JsxScopedStyleService::virtual_css_documents(content, uri);
        if styles.is_empty() {
            self.virtual_docs_cache.remove(uri);
            return;
        }
        let mut docs = VirtualDocuments::new();
        docs.styles = styles;
        self.virtual_docs_cache.insert(uri.clone(), Arc::new(docs));
    }

    /// Generate and cache virtual documents for an art file (*.art.vue).
    ///
    /// Uses the default variant's template as the synthetic template block,
    /// and generates virtual docs for script_setup if present.
    fn update_art_virtual_docs(&self, uri: &Url, content: &str) {
        use crate::virtual_code::{ScriptCodeGenerator, TemplateCodeGenerator, VirtualDocuments};

        let allocator = vize_s0::Allocator::new();
        let Ok(art_desc) =
            vize_musea::parse_art(&allocator, content, vize_musea::ArtParseOptions::default())
        else {
            self.remove_virtual_docs(uri);
            return;
        };

        let base_uri = uri.path();
        let mut docs = VirtualDocuments::new();

        // Generate one virtual template per variant so editor features remain correct even when
        // the cursor is inside a non-default variant.
        docs.art_templates.resize(art_desc.variants.len(), None);

        for (index, variant) in art_desc.variants.iter().enumerate() {
            let template_content = variant.template;
            if template_content.trim().is_empty() {
                continue;
            }

            let template_allocator = vize_s0::Allocator::new();
            let (ast, _errors) = vize_armature::parse(&template_allocator, template_content);

            let template_ptr = template_content.as_ptr() as usize;
            let source_ptr = content.as_ptr() as usize;
            let block_offset = (template_ptr - source_ptr) as u32;

            let mut template_gen = TemplateCodeGenerator::new();
            template_gen.set_block_offset(block_offset);
            let mut template_doc = template_gen.generate(&ast, template_content);
            template_doc.uri =
                vize_s0::cstr!("{base_uri}.art_variant_{index}.template.ts").to_string();

            if variant.is_default || docs.template.is_none() {
                docs.template = Some(template_doc.clone());
            }

            docs.art_templates[index] = Some(template_doc);
        }

        // Generate script_setup virtual doc using SFC parser
        // (SFC parser handles script blocks even in art files)
        let sfc_options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };
        if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, sfc_options) {
            if let Some(ref script_setup) = descriptor.script_setup {
                let isolate = art_script_setup_isolated(script_setup);
                let mut script_doc = generate_art_script_setup_virtual_doc(
                    base_uri,
                    script_setup.content.as_ref(),
                    script_setup.loc.start,
                    art_desc.variants.len(),
                    isolate,
                );
                script_doc.uri = vize_s0::cstr!("{base_uri}.__script_setup.ts").to_string();
                docs.script_setup = Some(script_doc);
            }
            if let Some(ref script) = descriptor.script {
                let mut script_gen = ScriptCodeGenerator::new();
                let mut script_doc = script_gen.generate(script, false);
                script_doc.uri = vize_s0::cstr!("{base_uri}.__script.ts").to_string();
                docs.script = Some(script_doc);
            }
            super::art_template_context::attach(
                &mut docs,
                descriptor.script_setup.as_ref(),
                descriptor
                    .script_setup
                    .as_ref()
                    .is_some_and(art_script_setup_isolated),
            );
        }

        self.virtual_docs_cache.insert(uri.clone(), Arc::new(docs));
    }

    /// Owned snapshot of a document's cached virtual documents: an `Arc` clone,
    /// never a `DashMap` shard guard, so nothing stays locked after it returns.
    ///
    /// Load-bearing, not a style choice (#3377). `vize lsp` drives tower-lsp on
    /// one `block_on` thread while `Server::serve` polls up to four queued
    /// messages concurrently, so a handler suspended at an `.await` and a
    /// `didOpen`/`didChange`/`didClose` write share that thread. A suspended
    /// handler still holding a shard read guard — as [`crate::ide::IdeContext`]
    /// used to across the hover, completion, definition, references and rename
    /// awaits — parks [`Self::update_virtual_docs`] in `parking_lot` on the
    /// shard write lock, and the only thread that could poll the reader into
    /// releasing it is the parked one: a permanent, silent server hang. #3373
    /// removed the same shape from the open-document store, see
    /// [`crate::document::DocumentStore::text`]. The `Arc` keeps the cost at a
    /// refcount bump instead of cloning every virtual document per request.
    pub fn get_virtual_docs(&self, uri: &Url) -> Option<Arc<VirtualDocuments>> {
        self.virtual_docs_cache
            .get(uri)
            .map(|entry| Arc::clone(entry.value()))
    }

    /// Remove cached virtual documents when a document is closed.
    pub fn remove_virtual_docs(&self, uri: &Url) {
        self.open_imports.remove(uri);
        self.virtual_docs_cache.remove(uri);
    }

    /// Clear all cached virtual documents.
    pub fn clear_virtual_docs(&self) {
        self.open_imports.clear();
        self.virtual_docs_cache.clear();
    }

    /// Cache of parsed imported-component metadata, keyed by resolved path.
    /// Used by template completion to avoid re-parsing imported components on
    /// every keystroke. Callers handle staleness via the entry's file stamp.
    pub(crate) fn component_metadata_cache(
        &self,
    ) -> &DashMap<PathBuf, crate::ide::completion::template::CachedComponentMetadata> {
        &self.component_metadata_cache
    }
}
