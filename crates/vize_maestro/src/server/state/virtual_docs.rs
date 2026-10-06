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

fn script_projection<'a>(
    descriptor: &'a vize_atelier_sfc::SfcDescriptor<'a>,
) -> (Option<&'a str>, bool, u32) {
    if let Some(block) = descriptor.script_setup.as_ref() {
        (Some(block.content.as_ref()), true, block.loc.start as u32)
    } else if let Some(block) = descriptor.script.as_ref() {
        (Some(block.content.as_ref()), false, block.loc.start as u32)
    } else {
        (None, false, 0)
    }
}

#[cfg(test)]
mod tests;

impl ServerState {
    /// Generate and cache virtual documents for a document.
    pub fn update_virtual_docs(&self, uri: &Url, content: &str) {
        self.open_imports.update(uri, content);
        self.binding_occurrences.remove(uri);
        let config = self.occurrence_config();
        let revision = self.occurrence_source_revision(uri, content);
        if uri.path().ends_with(".art.vue") {
            self.update_art_virtual_docs(uri, content);
            return;
        }

        if is_standalone_html_path(uri.path()) {
            self.publish_legacy_occurrences(uri, content, revision, config);
            self.update_standalone_html_virtual_docs(uri, content);
            return;
        }

        if crate::utils::is_plain_script_path(uri.path()) {
            self.publish_legacy_occurrences(uri, content, revision, config);
            self.virtual_docs_cache.remove(uri);
            return;
        }

        if crate::utils::is_jsx_path(uri.path()) {
            self.publish_legacy_occurrences(uri, content, revision, config);
            self.update_jsx_virtual_docs(uri, content);
            return;
        }

        let Some(descriptor) = self.sfc_descriptor(uri, content) else {
            self.remove_virtual_docs(uri);
            return;
        };

        let base_uri = uri.path();
        let features = config.features;
        let authored_domain = !config.legacy_vue2
            && crate::virtual_code::PhysicalOccurrences::supports_sfc_descriptor(&descriptor);
        let (mut virtual_docs, facts) =
            if revision.is_some() && authored_domain && (features.references || features.code_lens)
            {
                self.virtual_gen
                    .write()
                    .generate_with_occurrences(&descriptor, base_uri, content)
            } else {
                (
                    self.virtual_gen.write().generate(&descriptor, base_uri),
                    None,
                )
            };
        add_inline_art_template_virtual_docs(&mut virtual_docs, &descriptor, base_uri);
        super::art_template_context::attach(&mut virtual_docs);
        self.virtual_docs_cache
            .insert(uri.clone(), Arc::new(virtual_docs));
        if authored_domain {
            self.publish_occurrences(uri, content, revision, config, facts);
        } else {
            self.publish_legacy_occurrences(uri, content, revision, config);
        }
    }

    /// Generate and cache virtual documents for standalone HTML files.
    fn update_standalone_html_virtual_docs(&self, uri: &Url, content: &str) {
        use crate::virtual_code::{VirtualDocuments, project_template_fragment};

        let allocator = vize_l0::Allocator::new();
        let (ast, _errors) = vize_armature::parse(&allocator, content);
        let base_uri = uri.path();
        let template_doc = project_template_fragment(
            None,
            false,
            0,
            &ast,
            0,
            vize_l0::cstr!("{base_uri}.__template.ts").to_string(),
        );

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
        use crate::virtual_code::{
            PhysicalOccurrences, ScriptCodeGenerator, VirtualDocuments,
            project_template_fragment_with_occurrences,
        };

        let allocator = vize_l0::Allocator::new();
        let Ok(art_desc) =
            vize_musea::parse_art(&allocator, content, vize_musea::ArtParseOptions::default())
        else {
            self.remove_virtual_docs(uri);
            return;
        };

        let base_uri = uri.path();
        let config = self.occurrence_config();
        let features = config.features;
        let revision = self.occurrence_source_revision(uri, content);
        let capture = revision.is_some()
            && !config.legacy_vue2
            && (features.references || features.code_lens);
        let mut docs = VirtualDocuments::new();
        let descriptor = self.sfc_descriptor(uri, content);
        // This existing fragment analysis owns only its selected script and
        // variant templates. Do not call a partial Art relation complete.
        let complete_fragment = descriptor.as_ref().is_some_and(|descriptor| {
            descriptor.styles.is_empty()
                && !(descriptor.script.is_some() && descriptor.script_setup.is_some())
        });
        let authored_domain = !config.legacy_vue2
            && complete_fragment
            && art_desc
                .variants
                .iter()
                .any(|variant| !variant.template.trim().is_empty());
        let capture = capture && authored_domain;
        let mut facts = capture.then(PhysicalOccurrences::default);
        let (script, script_setup, script_offset) = descriptor
            .as_deref()
            .map(script_projection)
            .unwrap_or((None, false, 0));

        // One checker document per variant, so a non-default variant keeps its own mappings.
        docs.art_templates.resize(art_desc.variants.len(), None);

        for (index, variant) in art_desc.variants.iter().enumerate() {
            let template_content = variant.template;
            if template_content.trim().is_empty() {
                continue;
            }

            let template_allocator = vize_l0::Allocator::new();
            let (ast, errors) = vize_armature::parse(&template_allocator, template_content);

            let template_ptr = template_content.as_ptr() as usize;
            let source_ptr = content.as_ptr() as usize;
            let block_offset = (template_ptr - source_ptr) as u32;

            let (template_doc, packet) = project_template_fragment_with_occurrences(
                script,
                script_setup,
                script_offset,
                &ast,
                block_offset,
                vize_l0::cstr!("{base_uri}.art_variant_{index}.template.ts").to_string(),
                (capture && errors.is_empty()).then_some(content),
            );
            match (facts.as_mut(), packet) {
                (Some(facts), Some(packet)) => facts.merge(packet),
                _ => facts = None,
            }

            if variant.is_default || docs.template.is_none() {
                docs.template = Some(template_doc.clone());
            }

            if let Some(slot) = docs.art_templates.get_mut(index) {
                *slot = Some(template_doc);
            }
        }

        // Generate script_setup virtual doc using SFC parser
        // (SFC parser handles script blocks even in art files)
        if let Some(descriptor) = descriptor.as_ref() {
            if let Some(ref script_setup) = descriptor.script_setup {
                let isolate = art_script_setup_isolated(script_setup);
                let mut script_doc = generate_art_script_setup_virtual_doc(
                    base_uri,
                    script_setup.content.as_ref(),
                    script_setup.loc.start,
                    art_desc.variants.len(),
                    isolate,
                );
                script_doc.uri = vize_l0::cstr!("{base_uri}.__script_setup.ts").to_string();
                docs.script_setup = Some(script_doc);
            }
            if let Some(ref script) = descriptor.script {
                let mut script_gen = ScriptCodeGenerator::new();
                let mut script_doc = script_gen.generate(script, false);
                script_doc.uri = vize_l0::cstr!("{base_uri}.__script.ts").to_string();
                docs.script = Some(script_doc);
            }
        }

        super::art_template_context::attach(&mut docs);

        self.virtual_docs_cache.insert(uri.clone(), Arc::new(docs));
        if authored_domain {
            self.publish_occurrences(uri, content, revision, config, facts);
        } else {
            self.publish_legacy_occurrences(uri, content, revision, config);
        }
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
        self.binding_occurrences.remove(uri);
    }

    /// Clear all cached virtual documents.
    pub fn clear_virtual_docs(&self) {
        self.open_imports.clear();
        self.virtual_docs_cache.clear();
        self.binding_occurrences.clear();
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
